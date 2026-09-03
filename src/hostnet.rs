//! Host networking — individual `net_*` host functions backed by [`rustix`].
//!
//! Note: A Hyperlight host function call is a synchronous VM exit — the guest
//! is fully paused until the call returns.  This means blocking sockets
//! are correct: `recv()` blocks until data, `poll()` blocks until events
//! or timeout.
//!
//! ## Return conventions
//!
//! - **`i32` returns**: non-negative on success (value is op-specific),
//!   negative = `-errno` on error.
//! - **`Vec<u8>` returns**: first 4 bytes are `i32` status, followed by
//!   packed data on success.
//!
//! ## Cross-platform strategy
//!
//! All socket operations go through `rustix`, which internally dispatches
//! to the correct platform syscall (Linux `poll`/`setsockopt`/… vs Winsock
//! `WSAPoll`/`setsockopt`/…).  The only remaining platform-specific code
//! is [`errno_to_linux`], which maps `rustix::io::Errno` to Linux errno values
//! for the guest (a Linux unikernel).

use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use hyperlight_host::func::Registerable;
use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::fd::OwnedFd;
use rustix::io::Errno;
#[cfg(unix)]
use rustix::net::SocketFlags;
use rustix::net::{self as rnet, AddressFamily, RecvFlags, SendFlags, SocketType, sockopt};

use crate::net_policy::{self, ListenPorts, NetworkPolicy};

const MAX_SOCKETS: usize = 1024;

// ── Linux errno values ──────────────────────────────────────────────
//
// The guest is a Linux unikernel (Unikraft) and expects POSIX errno
// values.  On Linux hosts `Errno::raw_os_error()` already returns
// these, but on Windows it returns Winsock error codes (10000+).
// Mapping through `Errno` variants gives us platform-independent
// values the guest understands.

/// Map a `rustix::io::Errno` to the corresponding Linux errno integer.
///
/// `rustix::io::Errno` uses the same variant names on every platform
/// (e.g. `Errno::CONNREFUSED`), so the match works identically on
/// Linux and Windows — the difference is hidden inside rustix.
fn errno_to_linux(e: Errno) -> i32 {
    match e {
        Errno::INTR => 4,
        #[cfg(not(windows))]
        Errno::IO => 5,
        Errno::BADF => 9,
        Errno::AGAIN => 11,
        Errno::ACCESS => 13,
        Errno::FAULT => 14,
        Errno::INVAL => 22,
        Errno::MFILE => 24,
        Errno::NOTSOCK => 88,
        Errno::PROTONOSUPPORT => 93,
        Errno::AFNOSUPPORT => 97,
        Errno::ADDRINUSE => 98,
        Errno::ADDRNOTAVAIL => 99,
        Errno::NETDOWN => 100,
        Errno::NETUNREACH => 101,
        Errno::CONNABORTED => 103,
        Errno::CONNRESET => 104,
        Errno::NOBUFS => 105,
        Errno::ISCONN => 106,
        Errno::NOTCONN => 107,
        Errno::TIMEDOUT => 110,
        Errno::CONNREFUSED => 111,
        Errno::HOSTUNREACH => 113,
        Errno::ALREADY => 114,
        Errno::INPROGRESS => 115,
        _ => 5, // EIO
    }
}

// ── SocketTable ──────────────────────────────────────────────────────

struct SocketTable {
    sockets: HashMap<i32, OwnedFd>,
    next_fd: i32,
}

impl SocketTable {
    fn new() -> Self {
        Self {
            sockets: HashMap::new(),
            next_fd: 3, // skip stdin/stdout/stderr
        }
    }

    fn insert(&mut self, socket: OwnedFd) -> Result<i32, i32> {
        if self.sockets.len() >= MAX_SOCKETS {
            return Err(24); // EMFILE
        }
        let fd = self.next_fd;
        self.next_fd = self.next_fd.wrapping_add(1);
        self.sockets.insert(fd, socket);
        Ok(fd)
    }

    fn get(&self, fd: i32) -> Option<&OwnedFd> {
        self.sockets.get(&fd)
    }

    fn remove(&mut self, fd: i32) -> Option<OwnedFd> {
        self.sockets.remove(&fd)
    }
}

type Table = Arc<Mutex<SocketTable>>;

fn lock(t: &Table) -> std::sync::MutexGuard<'_, SocketTable> {
    t.lock().unwrap()
}

// ── Registration ─────────────────────────────────────────────────────

/// Register all `net_*` host functions plus `net_resolve` for DNS.
///
/// `policy` controls which outbound destinations are allowed.
/// `listen_ports` controls which ports `net_bind` accepts.
pub(crate) fn register(
    target: &mut impl Registerable,
    policy: Option<NetworkPolicy>,
    listen_ports: Option<ListenPorts>,
) -> hyperlight_host::Result<()> {
    // On Windows, Winsock must be initialized before any socket calls.
    // rustix doesn't do this automatically (unlike the old socket2 dep).
    // Binding a std::net socket triggers WSAStartup as a side-effect.
    #[cfg(windows)]
    {
        use std::sync::Once;
        static WINSOCK_INIT: Once = Once::new();
        WINSOCK_INIT.call_once(|| {
            let _ = std::net::UdpSocket::bind("127.0.0.1:0");
        });
    }

    let table: Table = Arc::new(Mutex::new(SocketTable::new()));
    let policy = policy.map(Arc::new);
    let listen_ports = listen_ports.map(Arc::new);

    reg_socket(target, &table)?;
    reg_bind(target, &table, &listen_ports)?;
    reg_listen(target, &table)?;
    reg_accept(target, &table)?;
    reg_connect(target, &table, &policy)?;
    reg_send(target, &table)?;
    reg_sendto(target, &table, &policy)?;
    reg_recvfrom(target, &table, &policy)?;
    reg_shutdown(target, &table)?;
    reg_close(target, &table)?;
    reg_getpeername(target, &table)?;
    reg_getsockname(target, &table)?;
    reg_getsockopt(target, &table)?;
    reg_setsockopt(target, &table)?;
    reg_poll(target, &table)?;
    reg_resolve(target)?;
    reg_nanosleep(target)?;

    Ok(())
}

// ── Individual handlers ──────────────────────────────────────────────

/// `net_socket(family, type, protocol) -> fd or -errno`
fn reg_socket(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_socket",
        move |family: i32, ty: i32, proto: i32| -> hyperlight_host::Result<i32> {
            let domain = match family {
                2 => AddressFamily::INET,
                10 => AddressFamily::INET6,
                _ => return Ok(-97), // EAFNOSUPPORT
            };
            let sock_type = match ty & 0xFF {
                1 => SocketType::STREAM,
                2 => SocketType::DGRAM,
                _ => return Ok(-93), // EPROTONOSUPPORT
            };
            let protocol = match proto {
                0 => None,
                6 => Some(rustix::net::ipproto::TCP),
                17 => Some(rustix::net::ipproto::UDP),
                _ => return Ok(-93), // EPROTONOSUPPORT
            };

            // Create the socket.  On Unix we pass CLOEXEC and, if the
            // guest requested it, NONBLOCK directly via socket_with().
            // On Windows these flags don't exist; sockets are inherently
            // non-inheritable and NONBLOCK is set via ioctlsocket later.
            #[cfg(unix)]
            let sock_result = {
                let mut flags = SocketFlags::CLOEXEC;
                if ty & 0x800 != 0 {
                    flags |= SocketFlags::NONBLOCK;
                }
                rnet::socket_with(domain, sock_type, flags, protocol)
            };
            #[cfg(not(unix))]
            let sock_result = rnet::socket(domain, sock_type, protocol);
            match sock_result {
                Ok(sock) => {
                    // Increase socket buffer sizes for TCP to match Linux
                    // auto-tuning defaults.  256 KB per direction avoids
                    // deadlocks when large payloads fill small default buffers
                    // (e.g. 8 KB send on Windows) during cooperative threading.
                    if sock_type == SocketType::STREAM {
                        let _ = sockopt::set_socket_send_buffer_size(&sock, 256 * 1024);
                        let _ = sockopt::set_socket_recv_buffer_size(&sock, 256 * 1024);
                    }

                    let mut tbl = lock(&tbl);
                    match tbl.insert(sock) {
                        Ok(fd) => Ok(fd),
                        Err(e) => Ok(-e),
                    }
                }
                Err(e) => Ok(-errno_to_linux(e)),
            }
        },
    )
}

/// `net_bind(fd, family, addr, port) -> 0 or -errno`
fn reg_bind(
    t: &mut impl Registerable,
    table: &Table,
    listen_ports: &Option<Arc<ListenPorts>>,
) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    let lp = listen_ports.clone();
    t.register_host_function(
        "net_bind",
        move |fd: i32, family: i32, addr: String, port: i32| -> hyperlight_host::Result<i32> {
            let sa = match parse_addr(family, &addr, port) {
                Some(a) => a,
                None => return Ok(-22), // EINVAL
            };
            // Enforce listen-port allowlist (skip for port 0 = ephemeral).
            if sa.port() != 0
                && let Some(ref lp) = lp
                && lp.check(sa.port()).is_err()
            {
                return Ok(-13); // EACCES
            }
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(match rnet::bind(sock, &sa) {
                    Ok(()) => 0,
                    Err(e) => -errno_to_linux(e),
                }),
                None => Ok(-9), // EBADF
            }
        },
    )
}

/// `net_listen(fd, backlog) -> 0 or -errno`
fn reg_listen(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_listen",
        move |fd: i32, backlog: i32| -> hyperlight_host::Result<i32> {
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(match rnet::listen(sock, backlog) {
                    Ok(()) => 0,
                    Err(e) => -errno_to_linux(e),
                }),
                None => Ok(-9), // EBADF
            }
        },
    )
}

/// `net_accept(fd) -> Vec<u8>`
///
/// Returns:
///   [0..4]  i32  new_fd (>= 0) or -errno
///   [4..8]  i32  family (2=IPv4, 10=IPv6)
///   [8..10] u16  port
///   [10]    u8   addr_len (4 or 16)
///   [11..]  addr bytes
fn reg_accept(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_accept",
        move |fd: i32| -> hyperlight_host::Result<Vec<u8>> {
            // Clone the listener so we can drop the lock before blocking.
            let listener = {
                let tbl_guard = lock(&tbl);
                match tbl_guard.get(fd) {
                    Some(sock) => match sock.try_clone() {
                        Ok(s) => s,
                        Err(e) => return Ok(errno_vec_io(e)),
                    },
                    None => return Ok((-9i32).to_le_bytes().to_vec()), // EBADF
                }
            };
            // Lock is released — safe to block on accept.
            let (new_sock, peer) = match rnet::acceptfrom(&listener) {
                Ok(pair) => pair,
                Err(e) => return Ok(errno_vec(e)),
            };
            let new_fd = {
                let mut tbl_guard = lock(&tbl);
                match tbl_guard.insert(new_sock) {
                    Ok(fd) => fd,
                    Err(e) => return Ok((-e).to_le_bytes().to_vec()),
                }
            };
            let mut buf = Vec::with_capacity(32);
            buf.extend(new_fd.to_le_bytes());
            if let Some(addr) = peer
                && let Ok(sa) = SocketAddr::try_from(addr)
            {
                pack_addr(&mut buf, &sa);
            }
            Ok(buf)
        },
    )
}

/// `net_connect(fd, family, addr, port) -> 0 or -errno`
fn reg_connect(
    t: &mut impl Registerable,
    table: &Table,
    policy: &Option<Arc<NetworkPolicy>>,
) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    let pol = policy.clone();
    t.register_host_function(
        "net_connect",
        move |fd: i32, family: i32, addr: String, port: i32| -> hyperlight_host::Result<i32> {
            let sa = match parse_addr(family, &addr, port) {
                Some(a) => a,
                None => return Ok(-22), // EINVAL
            };
            // Enforce network policy.
            if let Some(ref pol) = pol
                && pol.check(&sa).is_err()
            {
                return Ok(-13); // EACCES
            }
            // Clone the socket so we can drop the lock before blocking.
            let sock_clone = {
                let tbl = lock(&tbl);
                match tbl.get(fd) {
                    Some(sock) => match sock.try_clone() {
                        Ok(s) => s,
                        Err(_) => return Ok(-5), // EIO
                    },
                    None => return Ok(-9), // EBADF
                }
            };
            // Cap connect duration: the guest vCPU is frozen during this
            // blocking call, so an indefinite connect hangs the whole VM.
            // SO_SNDTIMEO limits the connect timeout on Linux; Windows
            // uses its own ~21 s TCP retransmission timeout regardless.
            let _ = sockopt::set_socket_timeout(
                &sock_clone,
                sockopt::Timeout::Send,
                Some(Duration::from_secs(30)),
            );
            // Lock is released — safe to block on connect.
            Ok(match rnet::connect(&sock_clone, &sa) {
                Ok(()) => 0,
                // Non-blocking connect in progress — treat as success.
                Err(e)
                    if e == Errno::INPROGRESS || e == Errno::WOULDBLOCK || e == Errno::ALREADY =>
                {
                    0
                }
                Err(e) => -errno_to_linux(e),
            })
        },
    )
}

/// `net_send(fd, data) -> bytes_sent or -errno`
fn reg_send(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_send",
        move |fd: i32, data: Vec<u8>| -> hyperlight_host::Result<i32> {
            // Clone the socket so we can drop the lock before blocking.
            let sock_clone = {
                let tbl = lock(&tbl);
                match tbl.get(fd) {
                    Some(sock) => match sock.try_clone() {
                        Ok(s) => s,
                        Err(_) => return Ok(-5), // EIO
                    },
                    None => return Ok(-9), // EBADF
                }
            };
            // Lock is released — safe to block on send.
            Ok(match rnet::send(&sock_clone, &data, SendFlags::empty()) {
                Ok(n) => n as i32,
                Err(e) => -errno_to_linux(e),
            })
        },
    )
}

/// `net_sendto(fd, data, family, addr, port) -> bytes_sent or -errno`
fn reg_sendto(
    t: &mut impl Registerable,
    table: &Table,
    policy: &Option<Arc<NetworkPolicy>>,
) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    let pol = policy.clone();
    t.register_host_function(
        "net_sendto",
        move |fd: i32,
              data: Vec<u8>,
              family: i32,
              addr: String,
              port: i32|
              -> hyperlight_host::Result<i32> {
            let sa = match parse_addr(family, &addr, port) {
                Some(a) => a,
                None => return Ok(-22), // EINVAL
            };
            // Enforce network policy.
            if let Some(ref pol) = pol
                && pol.check(&sa).is_err()
            {
                return Ok(-13); // EACCES
            }
            // Clone the socket so we can drop the lock before blocking.
            let sock_clone = {
                let tbl = lock(&tbl);
                match tbl.get(fd) {
                    Some(sock) => match sock.try_clone() {
                        Ok(s) => s,
                        Err(_) => return Ok(-5), // EIO
                    },
                    None => return Ok(-9), // EBADF
                }
            };
            // Lock is released — safe to block on sendto.
            Ok(
                match rnet::sendto(&sock_clone, &data, SendFlags::empty(), &sa) {
                    Ok(n) => n as i32,
                    Err(e) => -errno_to_linux(e),
                },
            )
        },
    )
}

/// `net_recvfrom(fd, len) -> Vec<u8>`
///
/// Returns:
///   [0..4]  i32  bytes_received (>= 0) or -errno
///   [4..8]  i32  family
///   [8..10] u16  port
///   [10]    u8   addr_len
///   [11..11+addr_len] addr bytes
///   [11+addr_len..]   received data
fn reg_recvfrom(
    t: &mut impl Registerable,
    table: &Table,
    policy: &Option<Arc<NetworkPolicy>>,
) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    let pol = policy.clone();
    t.register_host_function(
        "net_recvfrom",
        move |fd: i32, len: i32| -> hyperlight_host::Result<Vec<u8>> {
            let len = (len as usize).min(65536);
            // Clone the socket so we can drop the lock before blocking.
            let sock_clone = {
                let tbl = lock(&tbl);
                match tbl.get(fd) {
                    Some(sock) => match sock.try_clone() {
                        Ok(s) => s,
                        Err(e) => return Ok(errno_vec_io(e)),
                    },
                    None => return Ok((-9i32).to_le_bytes().to_vec()), // EBADF
                }
            };
            // Lock is released — safe to block on recv.
            //
            // On Windows, recvfrom() can return WSAEINVAL on connected
            // TCP sockets in some configurations.  Use recv() as the
            // primary path on Windows — the source address is only
            // meaningful for UDP, and the guest can use getpeername()
            // when it needs the peer address.
            #[cfg(windows)]
            let recv_result: Result<
                (Vec<u8>, usize, Option<std::net::SocketAddr>),
                Errno,
            > = {
                let mut buf = vec![MaybeUninit::uninit(); len];
                match rnet::recv(&sock_clone, &mut buf[..], RecvFlags::empty()) {
                    Ok(((init_data, _), n)) => {
                        let n = n.min(len);
                        Ok((init_data[..n].to_vec(), n, None))
                    }
                    Err(e) => Err(e),
                }
            };

            #[cfg(not(windows))]
            let recv_result: Result<
                (Vec<u8>, usize, Option<std::net::SocketAddr>),
                Errno,
            > = {
                let mut buf = vec![MaybeUninit::uninit(); len];
                match rnet::recvfrom(&sock_clone, &mut buf[..], RecvFlags::empty()) {
                    Ok(((init_data, _), n, src_addr)) => {
                        let n = n.min(len);
                        let sa = src_addr.and_then(|a| SocketAddr::try_from(a).ok());
                        Ok((init_data[..n].to_vec(), n, sa))
                    }
                    Err(e) => Err(e),
                }
            };

            match recv_result {
                Ok((data, n, src_addr)) => {
                    // Learn IPs from DNS responses when using AllowList.
                    if let Some(ref pol) = pol
                        && let NetworkPolicy::AllowList(ref al) = **pol
                        && let Some(ref sa) = src_addr
                        && sa.port() == 53
                    {
                        net_policy::learn_ips_from_dns_response(&data, al);
                    }

                    let mut buf = Vec::with_capacity(16 + n);
                    buf.extend((n as i32).to_le_bytes());
                    if let Some(sa) = src_addr {
                        pack_addr(&mut buf, &sa);
                    } else {
                        pack_zero_addr(&mut buf);
                    }
                    buf.extend_from_slice(&data);
                    Ok(buf)
                }
                Err(e) => Ok(errno_vec(e)),
            }
        },
    )
}

/// `net_shutdown(fd, how) -> 0 or -errno`
fn reg_shutdown(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_shutdown",
        move |fd: i32, how: i32| -> hyperlight_host::Result<i32> {
            let shut = match how {
                0 => rustix::net::Shutdown::Read,
                1 => rustix::net::Shutdown::Write,
                _ => rustix::net::Shutdown::Both,
            };
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(match rnet::shutdown(sock, shut) {
                    Ok(()) => 0,
                    Err(e) => -errno_to_linux(e),
                }),
                None => Ok(-9), // EBADF
            }
        },
    )
}

/// `net_close(fd) -> 0 or -errno`
fn reg_close(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_close",
        move |fd: i32| -> hyperlight_host::Result<i32> {
            let mut tbl = lock(&tbl);
            tbl.remove(fd); // OwnedFd::drop closes the fd
            Ok(0)
        },
    )
}

/// `net_getpeername(fd) -> Vec<u8>`
///
/// Returns:
///   [0..4]  i32  status (0 or -errno)
///   [4..]   packed addr (family, port, addr_len, addr_bytes)
fn reg_getpeername(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_getpeername",
        move |fd: i32| -> hyperlight_host::Result<Vec<u8>> {
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => match rnet::getpeername(sock) {
                    Ok(Some(any)) => match SocketAddr::try_from(any) {
                        Ok(sa) => Ok(addr_result(&sa)),
                        Err(_) => Ok((-97i32).to_le_bytes().to_vec()), // EAFNOSUPPORT
                    },
                    Ok(None) => Ok((-107i32).to_le_bytes().to_vec()), // ENOTCONN
                    Err(e) => Ok(errno_vec(e)),
                },
                None => Ok((-9i32).to_le_bytes().to_vec()), // EBADF
            }
        },
    )
}

/// `net_getsockname(fd) -> Vec<u8>`
fn reg_getsockname(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_getsockname",
        move |fd: i32| -> hyperlight_host::Result<Vec<u8>> {
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => match rnet::getsockname(sock) {
                    Ok(any) => match SocketAddr::try_from(any) {
                        Ok(sa) => Ok(addr_result(&sa)),
                        Err(_) => Ok((-97i32).to_le_bytes().to_vec()), // EAFNOSUPPORT
                    },
                    Err(e) => Ok(errno_vec(e)),
                },
                None => Ok((-9i32).to_le_bytes().to_vec()), // EBADF
            }
        },
    )
}

/// `net_getsockopt(fd, level, optname) -> value or -errno`
fn reg_getsockopt(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_getsockopt",
        move |fd: i32, level: i32, optname: i32| -> hyperlight_host::Result<i32> {
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(get_sockopt(sock, level, optname)),
                None => Ok(-9), // EBADF
            }
        },
    )
}

/// `net_setsockopt(fd, level, optname, value) -> 0 or -errno`
fn reg_setsockopt(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_setsockopt",
        move |fd: i32, level: i32, optname: i32, value: i32| -> hyperlight_host::Result<i32> {
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(set_sockopt(sock, level, optname, value)),
                None => Ok(-9), // EBADF
            }
        },
    )
}

/// `net_poll(pollfds: Vec<u8>, timeout_ms) -> Vec<u8>`
///
/// Blocks in the host's `poll()` for up to `timeout_ms` milliseconds.
///
/// Input Vec<u8> (8 bytes per fd):
///   i32 fd, i16 events, i16 pad
///
/// Returns Vec<u8>:
///   [0..4]  i32  retval (ready count, 0=timeout, <0=-errno)
///   [4..]   i16  revents per fd
fn reg_poll(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_poll",
        move |pollfds_raw: Vec<u8>, timeout_ms: i32| -> hyperlight_host::Result<Vec<u8>> {
            let nfds = pollfds_raw.len() / 8;

            // Translate guest event flags (Linux POLLIN/POLLOUT/POLLPRI
            // constants) to rustix PollFlags (platform-independent).
            let tbl = lock(&tbl);
            let guest_fds: Vec<i32> = (0..nfds)
                .map(|i| {
                    let off = i * 8;
                    i32::from_le_bytes(pollfds_raw[off..off + 4].try_into().unwrap())
                })
                .collect();
            let guest_events: Vec<i16> = (0..nfds)
                .map(|i| {
                    let off = i * 8;
                    i16::from_le_bytes(pollfds_raw[off + 4..off + 6].try_into().unwrap())
                })
                .collect();

            // Build PollFd array.  PollFd borrows the socket fds, so
            // the table lock must be held during the poll call.
            let mut fds: Vec<PollFd<'_>> = Vec::with_capacity(nfds);
            let mut valid = vec![false; nfds];
            for i in 0..nfds {
                let events = linux_events_to_pollflags(guest_events[i]);
                if let Some(sock) = tbl.get(guest_fds[i]) {
                    fds.push(PollFd::new(sock, events));
                    valid[i] = true;
                } else {
                    // Invalid fd — create a dummy PollFd for position.
                    // We still need the right count for the output.
                    // Use any valid fd with empty events; it won't trigger.
                    // Actually, we'll handle NVAL in the output.
                    if let Some(any_sock) = tbl.sockets.values().next() {
                        fds.push(PollFd::new(any_sock, PollFlags::empty()));
                    }
                }
            }

            // If we couldn't fill all positions (no valid sockets at all),
            // just return NVAL for everything.
            if fds.len() < nfds {
                let mut buf = Vec::with_capacity(4 + nfds * 2);
                buf.extend((-(22i32)).to_le_bytes()); // EINVAL
                for _ in 0..nfds {
                    buf.extend(0x0020i16.to_le_bytes()); // POLLNVAL
                }
                return Ok(buf);
            }

            // Convert timeout: negative = infinite, 0 = immediate, positive = ms.
            let timeout = if timeout_ms < 0 {
                None
            } else {
                Some(Timespec {
                    tv_sec: (timeout_ms / 1000) as _,
                    tv_nsec: {
                        // Nsecs is i64 on Linux, c_long (i32) on Windows.
                        let ms_frac = (timeout_ms % 1000) as i64;
                        (ms_frac * 1_000_000) as _
                    },
                })
            };

            let ret = poll(&mut fds, timeout.as_ref());

            let mut buf = Vec::with_capacity(4 + nfds * 2);
            match ret {
                Ok(n) => {
                    buf.extend((n as i32).to_le_bytes());
                    for (i, pfd) in fds.iter().enumerate() {
                        if valid[i] {
                            buf.extend(pollflags_to_linux_revents(pfd.revents()).to_le_bytes());
                        } else {
                            buf.extend(0x0020i16.to_le_bytes()); // POLLNVAL
                        }
                    }
                }
                Err(e) => {
                    buf.extend((-errno_to_linux(e)).to_le_bytes());
                    for _ in 0..nfds {
                        buf.extend(0i16.to_le_bytes());
                    }
                }
            }
            Ok(buf)
        },
    )
}

/// `net_resolve(hostname) -> String`
///
/// Returns comma-separated IP addresses, or "error:reason".
fn reg_resolve(t: &mut impl Registerable) -> hyperlight_host::Result<()> {
    t.register_host_function(
        "net_resolve",
        move |hostname: String| -> hyperlight_host::Result<String> {
            // ToSocketAddrs requires a port — use 0.
            let lookup = format!("{hostname}:0");
            match lookup.to_socket_addrs() {
                Ok(addrs) => {
                    let ips: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
                    if ips.is_empty() {
                        Ok("error:ENOENT".to_string())
                    } else {
                        Ok(ips.join(","))
                    }
                }
                Err(e) => Ok(format!("error:{e}")),
            }
        },
    )
}

/// `host_nanosleep(ns) -> 0`
fn reg_nanosleep(t: &mut impl Registerable) -> hyperlight_host::Result<()> {
    t.register_host_function(
        "host_nanosleep",
        move |ns: u64| -> hyperlight_host::Result<i32> {
            let dur = std::time::Duration::from_nanos(ns.min(30_000_000_000)); // cap 30s
            std::thread::sleep(dur);
            Ok(0)
        },
    )
}

// ── Sockopt dispatch ────────────────────────────────────────────────
//
// The guest sends Linux constants for level/optname.  rustix's typed
// sockopt functions handle platform differences internally, so we just
// need to map Linux constant pairs to the right function call.

/// Linux socket option constants.
const LINUX_SOL_SOCKET: i32 = 1;
const LINUX_IPPROTO_TCP: i32 = 6;

/// Dispatch a Linux getsockopt(level, optname) via rustix's typed API.
fn get_sockopt(sock: &OwnedFd, level: i32, optname: i32) -> i32 {
    match (level, optname) {
        // SOL_SOCKET options
        (LINUX_SOL_SOCKET, 2) => bool_to_i32(sockopt::socket_reuseaddr(sock)), // SO_REUSEADDR
        (LINUX_SOL_SOCKET, 3) => {
            // SO_TYPE
            match sockopt::socket_type(sock) {
                Ok(t) if t == SocketType::STREAM => 1,
                Ok(t) if t == SocketType::DGRAM => 2,
                Ok(_) => 0,
                Err(e) => -errno_to_linux(e),
            }
        }
        (LINUX_SOL_SOCKET, 4) => {
            // SO_ERROR
            match sockopt::socket_error(sock) {
                Ok(Ok(())) => 0,
                Ok(Err(e)) => errno_to_linux(e),
                Err(e) => -errno_to_linux(e),
            }
        }
        (LINUX_SOL_SOCKET, 6) => bool_to_i32(sockopt::socket_broadcast(sock)), // SO_BROADCAST
        (LINUX_SOL_SOCKET, 7) => {
            // SO_SNDBUF
            match sockopt::socket_send_buffer_size(sock) {
                Ok(n) => n as i32,
                Err(e) => -errno_to_linux(e),
            }
        }
        (LINUX_SOL_SOCKET, 8) => {
            // SO_RCVBUF
            match sockopt::socket_recv_buffer_size(sock) {
                Ok(n) => n as i32,
                Err(e) => -errno_to_linux(e),
            }
        }
        (LINUX_SOL_SOCKET, 9) => bool_to_i32(sockopt::socket_keepalive(sock)), // SO_KEEPALIVE
        (LINUX_SOL_SOCKET, 10) => bool_to_i32(sockopt::socket_oobinline(sock)), // SO_OOBINLINE
        (LINUX_SOL_SOCKET, 13) => {
            // SO_LINGER — return l_linger in seconds (0 if off)
            match sockopt::socket_linger(sock) {
                Ok(Some(dur)) => dur.as_secs() as i32,
                Ok(None) => 0,
                Err(e) => -errno_to_linux(e),
            }
        }
        (LINUX_SOL_SOCKET, 20) => {
            // SO_RCVTIMEO — return timeout in microseconds
            match sockopt::socket_timeout(sock, sockopt::Timeout::Recv) {
                Ok(Some(dur)) => dur.as_micros() as i32,
                Ok(None) => 0,
                Err(e) => -errno_to_linux(e),
            }
        }
        (LINUX_SOL_SOCKET, 21) => {
            // SO_SNDTIMEO — return timeout in microseconds
            match sockopt::socket_timeout(sock, sockopt::Timeout::Send) {
                Ok(Some(dur)) => dur.as_micros() as i32,
                Ok(None) => 0,
                Err(e) => -errno_to_linux(e),
            }
        }
        // IPPROTO_TCP options
        (LINUX_IPPROTO_TCP, 1) => bool_to_i32(sockopt::tcp_nodelay(sock)), // TCP_NODELAY
        // Fall through to raw getsockopt for anything we don't wrap.
        _ => {
            #[cfg(unix)]
            {
                use std::os::fd::AsRawFd;
                let mut val: i32 = 0;
                let mut len = std::mem::size_of::<i32>() as libc::socklen_t;
                let ret = unsafe {
                    libc::getsockopt(
                        sock.as_raw_fd(),
                        level,
                        optname,
                        &mut val as *mut i32 as *mut libc::c_void,
                        &mut len,
                    )
                };
                if ret < 0 {
                    let e = std::io::Error::last_os_error();
                    -(e.raw_os_error().unwrap_or(5))
                } else {
                    val
                }
            }
            #[cfg(not(unix))]
            {
                -92 // ENOPROTOOPT on Windows
            }
        }
    }
}

/// Dispatch a Linux setsockopt(level, optname, value) via rustix's typed API.
fn set_sockopt(sock: &OwnedFd, level: i32, optname: i32, value: i32) -> i32 {
    let res = match (level, optname) {
        // SOL_SOCKET options
        (LINUX_SOL_SOCKET, 2) => sockopt::set_socket_reuseaddr(sock, value != 0), // SO_REUSEADDR
        (LINUX_SOL_SOCKET, 6) => sockopt::set_socket_broadcast(sock, value != 0), // SO_BROADCAST
        (LINUX_SOL_SOCKET, 7) => {
            sockopt::set_socket_send_buffer_size(sock, value as usize) // SO_SNDBUF
        }
        (LINUX_SOL_SOCKET, 8) => {
            sockopt::set_socket_recv_buffer_size(sock, value as usize) // SO_RCVBUF
        }
        (LINUX_SOL_SOCKET, 9) => sockopt::set_socket_keepalive(sock, value != 0), // SO_KEEPALIVE
        (LINUX_SOL_SOCKET, 10) => sockopt::set_socket_oobinline(sock, value != 0), // SO_OOBINLINE
        (LINUX_SOL_SOCKET, 13) => {
            // SO_LINGER
            let linger = if value > 0 {
                Some(Duration::from_secs(value as u64))
            } else {
                None
            };
            sockopt::set_socket_linger(sock, linger)
        }
        #[cfg(not(windows))]
        (LINUX_SOL_SOCKET, 15) => sockopt::set_socket_reuseport(sock, value != 0), // SO_REUSEPORT
        #[cfg(windows)]
        (LINUX_SOL_SOCKET, 15) => sockopt::set_socket_reuseaddr(sock, value != 0), // SO_REUSEPORT → REUSEADDR
        (LINUX_SOL_SOCKET, 20) => {
            // SO_RCVTIMEO — value in microseconds from guest
            let timeout = if value > 0 {
                Some(Duration::from_micros(value as u64))
            } else {
                None
            };
            sockopt::set_socket_timeout(sock, sockopt::Timeout::Recv, timeout)
        }
        (LINUX_SOL_SOCKET, 21) => {
            // SO_SNDTIMEO — value in microseconds from guest
            let timeout = if value > 0 {
                Some(Duration::from_micros(value as u64))
            } else {
                None
            };
            sockopt::set_socket_timeout(sock, sockopt::Timeout::Send, timeout)
        }
        // IPPROTO_TCP options
        (LINUX_IPPROTO_TCP, 1) => sockopt::set_tcp_nodelay(sock, value != 0), // TCP_NODELAY
        // Fall through to raw setsockopt for any option we don't have
        // a typed rustix wrapper for.  This keeps compatibility with
        // options like IP_RECVERR that musl's DNS resolver requires.
        _ => {
            #[cfg(unix)]
            {
                use std::os::fd::AsRawFd;
                let ret = unsafe {
                    libc::setsockopt(
                        sock.as_raw_fd(),
                        level,
                        optname,
                        &value as *const i32 as *const libc::c_void,
                        std::mem::size_of::<i32>() as libc::socklen_t,
                    )
                };
                return if ret < 0 {
                    let e = std::io::Error::last_os_error();
                    -(e.raw_os_error().unwrap_or(5))
                } else {
                    0
                };
            }
            #[cfg(not(unix))]
            return 0; // best-effort: ignore unknown options on Windows
        }
    };
    match res {
        Ok(()) => 0,
        Err(e) => -errno_to_linux(e),
    }
}

/// Convert a `Result<bool>` sockopt getter to an i32 for the guest.
fn bool_to_i32(r: Result<bool, Errno>) -> i32 {
    match r {
        Ok(true) => 1,
        Ok(false) => 0,
        Err(e) => -errno_to_linux(e),
    }
}

// ── Poll flag translation ───────────────────────────────────────────
//
// The guest sends Linux `<poll.h>` constants.  rustix's `PollFlags`
// uses the correct platform values internally — we just need to convert
// between the guest's Linux constants and rustix's portable flags.

/// Linux poll(2) event constants.
const LINUX_POLLIN: i16 = 0x0001;
const LINUX_POLLPRI: i16 = 0x0002;
const LINUX_POLLOUT: i16 = 0x0004;
const LINUX_POLLERR: i16 = 0x0008;
const LINUX_POLLHUP: i16 = 0x0010;
const LINUX_POLLNVAL: i16 = 0x0020;

/// Translate Linux poll event bits to rustix PollFlags.
fn linux_events_to_pollflags(linux: i16) -> PollFlags {
    let mut flags = PollFlags::empty();
    if linux & LINUX_POLLIN != 0 {
        flags |= PollFlags::IN;
    }
    if linux & LINUX_POLLPRI != 0 {
        flags |= PollFlags::PRI;
    }
    if linux & LINUX_POLLOUT != 0 {
        flags |= PollFlags::OUT;
    }
    flags
}

/// Translate rustix PollFlags revents back to Linux constants.
///
/// Uses `intersects` instead of `contains` because Windows defines
/// POLLIN = POLLRDNORM | POLLRDBAND (0x0300).  WSAPoll typically
/// returns only POLLRDNORM (0x0100) for readable data, so a
/// `contains(POLLIN)` check requires both bits and silently fails.
/// `intersects` fires when *any* constituent bit is set, matching
/// Linux behaviour where POLLIN is a single bit (0x0001).
fn pollflags_to_linux_revents(flags: PollFlags) -> i16 {
    let mut linux: i16 = 0;
    if flags.intersects(PollFlags::IN) {
        linux |= LINUX_POLLIN;
    }
    if flags.intersects(PollFlags::PRI) {
        linux |= LINUX_POLLPRI;
    }
    if flags.intersects(PollFlags::OUT) {
        linux |= LINUX_POLLOUT;
    }
    if flags.intersects(PollFlags::ERR) {
        linux |= LINUX_POLLERR;
    }
    if flags.intersects(PollFlags::HUP) {
        linux |= LINUX_POLLHUP;
    }
    if flags.intersects(PollFlags::NVAL) {
        linux |= LINUX_POLLNVAL;
    }
    linux
}

// ── Helpers ──────────────────────────────────────────────────────────

fn parse_addr(family: i32, addr: &str, port: i32) -> Option<SocketAddr> {
    let port = port as u16;
    let ip: IpAddr = match family {
        2 => addr.parse::<Ipv4Addr>().ok()?.into(),
        10 => addr.parse::<Ipv6Addr>().ok()?.into(),
        _ => return None,
    };
    Some(SocketAddr::new(ip, port))
}

/// Pack a socket address into a buffer: family(i32) + port(u16) + addr_len(u8) + addr_bytes.
fn pack_addr(buf: &mut Vec<u8>, addr: &SocketAddr) {
    match addr {
        SocketAddr::V4(v4) => {
            buf.extend(2i32.to_le_bytes()); // AF_INET
            buf.extend(v4.port().to_le_bytes());
            buf.push(4); // addr_len
            buf.extend_from_slice(&v4.ip().octets());
        }
        SocketAddr::V6(v6) => {
            buf.extend(10i32.to_le_bytes()); // AF_INET6
            buf.extend(v6.port().to_le_bytes());
            buf.push(16); // addr_len
            buf.extend_from_slice(&v6.ip().octets());
        }
    }
}

/// Pack a zero address (when no source address is available).
fn pack_zero_addr(buf: &mut Vec<u8>) {
    buf.extend(0i32.to_le_bytes()); // family
    buf.extend(0u16.to_le_bytes()); // port
    buf.push(0); // addr_len
}

/// Build a successful address result: status=0 + packed addr.
fn addr_result(sa: &SocketAddr) -> Vec<u8> {
    let mut buf = Vec::with_capacity(24);
    buf.extend(0i32.to_le_bytes());
    pack_addr(&mut buf, sa);
    buf
}

/// Encode a rustix Errno as a 4-byte `-errno` Vec (Linux errno).
fn errno_vec(e: Errno) -> Vec<u8> {
    (-errno_to_linux(e)).to_le_bytes().to_vec()
}

/// Encode a std::io::Error as a 4-byte `-errno` Vec (Linux errno).
fn errno_vec_io(e: std::io::Error) -> Vec<u8> {
    let code = match e.raw_os_error() {
        Some(code) => {
            // On Windows, raw_os_error() returns Winsock codes.
            // Try to match via Errno for portable translation.
            errno_to_linux(Errno::from_raw_os_error(code))
        }
        None => 5, // EIO
    };
    (-code).to_le_bytes().to_vec()
}
