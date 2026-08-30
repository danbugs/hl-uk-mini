# Networking

Unikraft guests on Hyperlight can access host networking through a
**hostsock** driver that forwards POSIX socket calls to the host via
hypercalls.  The host manages real sockets; the guest only holds a
virtual file descriptor.

## Architecture

```
Guest (Unikraft)              Host (Rust)
─────────────────             ──────────────
Python/Node app               hostnet.rs
      │                            │
  libc socket()               SocketTable
      │                       (HashMap<i32, Socket>)
  hostsock.c                       │
  (POSIX socket driver)       socket2::Socket
      │                       (real OS socket)
  hl_hcall_int / vecbytes ──→ net_* host functions
```

### How it works

1. The guest calls `socket()`, `connect()`, `send()`, etc. through
   the standard POSIX socket API.
2. Unikraft's socket layer dispatches to **hostsock** — a kernel
   driver registered for `AF_INET` and `AF_INET6`.
3. hostsock serialises the call into `hl_param` structs and makes a
   synchronous host call (`hl_hcall_int` for integer results,
   `hl_hcall_vecbytes` for variable-length data).
4. The host's **hostnet** module looks up the virtual fd in a
   `SocketTable`, performs the operation on a real `socket2::Socket`,
   and returns the result.

### Blocking model and intra-guest networking

A host function call is a synchronous VM exit — the guest vCPU is
fully paused until the call returns.  If a blocking host call
(e.g. `accept()`, `recv()`) were allowed to block indefinitely, the
entire VM would freeze and no other guest thread could run.

To avoid this, hostsock uses a **check-ready pattern**: before any
potentially-blocking call, it polls the socket with `net_poll(timeout=0)`.
If the socket isn't ready, it returns `EAGAIN` instead of calling the
host.  Unikraft's POSIX socket layer then calls `uk_file_poll()`,
which blocks *the current thread* (not the vCPU) and yields to the
cooperative scheduler.  Other guest threads can now run.

When all threads are blocked, the scheduler's idle thread enters
`time_block_until()`, which periodically calls
`hostsock_rescan_events()` (~every 1 ms) to poll tracked sockets.
When a socket becomes ready, `posix_sock_event_set()` wakes the
waiting thread, putting it back on the run queue.

This enables intra-guest networking — for example, a server and
client can run in two threads inside the same guest
(see [`examples/python/tcp_echo.py`](../examples/python/tcp_echo.py)).

## Enabling networking

Pass a `NetworkPolicy` when creating a sandbox:

```rust
use hyperlight_unikraft::{NetworkPolicy, AllowList, BlockList, ListenPorts};

// Full network access (all outbound destinations permitted):
let (usandbox, _cfg) = create_sandbox(
    &Some(rootfs), &None, 256, Vec::new(),
    Some(NetworkPolicy::AllowAll), None,
)?;
```

When `None` (the default), no `net_*` host functions are registered
and guest socket calls fail.

## Host functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `net_socket` | `(family, type, proto) → fd` | Create a socket (IPv4/IPv6, TCP/UDP only) |
| `net_bind` | `(fd, family, addr, port) → 0` | Bind to an address |
| `net_listen` | `(fd, backlog) → 0` | Start listening |
| `net_accept` | `(fd) → [new_fd, addr]` | Accept a connection |
| `net_connect` | `(fd, family, addr, port) → 0` | Connect to a remote address |
| `net_send` | `(fd, data) → bytes_sent` | Send data |
| `net_sendto` | `(fd, data, family, addr, port) → bytes_sent` | Send to a specific address |
| `net_recvfrom` | `(fd, len) → [bytes, addr, data]` | Receive data |
| `net_shutdown` | `(fd, how) → 0` | Shut down part of a connection |
| `net_close` | `(fd) → 0` | Close a socket |
| `net_getpeername` | `(fd) → addr` | Get peer address |
| `net_getsockname` | `(fd) → addr` | Get local address |
| `net_getsockopt` | `(fd, level, optname) → value` | Get socket option |
| `net_setsockopt` | `(fd, level, optname, value) → 0` | Set socket option |
| `net_poll` | `(pollfds, timeout_ms) → [count, revents]` | Poll for I/O events |
| `net_resolve` | `(hostname) → "ip1,ip2,..."` | DNS resolution |
| `host_nanosleep` | `(ns) → 0` | Sleep (capped at 30 s) |

All functions return negative `-errno` values on error.

## Limits

- **Max sockets:** 1024 open sockets per sandbox (host-enforced).
- **Recv buffer:** 64 KiB per `recvfrom` call.
- **Send buffer:** 64 KiB per `write`/`sendmsg` call (larger writes
  are truncated with a kernel warning).
- **Tracked sockets:** 64 sockets can be tracked for poll rescan
  (guest-enforced; a warning is logged if the limit is reached).
- **Protocols:** Only `AF_INET`/`AF_INET6` and `SOCK_STREAM`/`SOCK_DGRAM`.
  No raw sockets, no `AF_UNIX`.

## Network policy

`NetworkPolicy` controls which outbound destinations a guest can reach.
The host enforces the policy on `connect()` and `sendto()` — before
the data leaves the VM.

### Variants

| Variant | Behaviour |
|---------|-----------|
| `AllowAll` | All destinations permitted (except link-local). |
| `AllowList(AllowList)` | Only listed IPs/hostnames are reachable. |
| `BlockList(BlockList)` | All destinations permitted *except* listed ones. |

### Always-blocked addresses

Regardless of which variant is active:

- **Link-local** (`169.254.0.0/16`) — blocked for all variants.
  Prevents the guest from reaching cloud metadata services
  (e.g. Azure IMDS at `169.254.169.254`).

### Loopback handling

- **AllowAll** — permits loopback (`127.0.0.0/8`).  In the hostsock
  model all guest sockets are real host sockets, so blocking loopback
  would break intra-guest server+client patterns (e.g. `tcp_echo.py`).
- **AllowList / BlockList** — blocks loopback.  Defense in depth:
  a restricted guest shouldn't reach host services on `127.0.0.1`.

### AllowList and DNS

`AllowList::from_hosts()` accepts a mix of IPs and hostnames:

```rust
let al = AllowList::from_hosts(&["example.com", "10.0.0.5"])?;
```

Hostnames are resolved at construction time.  At check time, hostnames
are re-resolved so that CDN IP rotation doesn't cause false positives.

When using an AllowList, well-known DNS resolver IPs (`8.8.8.8`,
`8.8.4.4`, `1.1.1.1`, `1.0.0.1`, plus any servers in
`/etc/resolv.conf`) are automatically exempted on port 53 — otherwise
the guest couldn't resolve the hostnames in the allowlist.

The host also learns IPs dynamically: when a `recvfrom` on port 53
returns a DNS response, the policy engine parses the A/AAAA records
and adds the resolved IPs to the allowlist.

### ListenPorts

`ListenPorts` is orthogonal to the outbound policy — it controls which
ports the guest may `bind()` for inbound connections:

```rust
let ports = ListenPorts::from([80, 443]);
let (usandbox, _cfg) = create_sandbox(
    &Some(rootfs), &None, 256, Vec::new(),
    Some(NetworkPolicy::AllowAll), Some(ports),
)?;
```

Ephemeral binds (port 0 — "assign any port") are always allowed.

### CLI usage

```sh
# AllowAll — full access:
hluk run --net ...

# AllowList — only reach these hosts:
hluk run --net --net-allow example.com --net-allow 10.0.0.5 ...

# BlockList — block these hosts:
hluk run --net --net-block evil.com --net-block 1.2.3.4 ...

# Restrict inbound listen ports:
hluk run --net --port 80 --port 443 ...
```

## Examples

- [`examples/python/tcp_echo.py`](../examples/python/tcp_echo.py) —
  TCP echo server and client running in two threads inside the guest
  (requires `NetworkPolicy::AllowAll` — uses loopback).
- [`examples/python/net_policy_probe.py`](../examples/python/net_policy_probe.py) —
  UDP sendto probe for integration-testing policy enforcement.
