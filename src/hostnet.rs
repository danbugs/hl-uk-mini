//! Host networking — individual `net_*` host functions backed by [`socket2`].
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

use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};

use hyperlight_host::func::Registerable;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use crate::net_policy::{self, ListenPorts, NetworkPolicy};

#[cfg(windows)]
use std::os::windows::io::AsRawSocket;

const MAX_SOCKETS: usize = 1024;

// ── SocketTable ──────────────────────────────────────────────────────

struct SocketTable {
    sockets: HashMap<i32, Socket>,
    next_fd: i32,
}

impl SocketTable {
    fn new() -> Self {
        Self {
            sockets: HashMap::new(),
            next_fd: 3, // skip stdin/stdout/stderr
        }
    }

    fn insert(&mut self, socket: Socket) -> Result<i32, i32> {
        if self.sockets.len() >= MAX_SOCKETS {
            return Err(libc::EMFILE);
        }
        let fd = self.next_fd;
        self.next_fd = self.next_fd.wrapping_add(1);
        self.sockets.insert(fd, socket);
        Ok(fd)
    }

    fn get(&self, fd: i32) -> Option<&Socket> {
        self.sockets.get(&fd)
    }

    fn remove(&mut self, fd: i32) -> Option<Socket> {
        self.sockets.remove(&fd)
    }
}

type Table = Arc<Mutex<SocketTable>>;

fn lock(t: &Table) -> std::sync::MutexGuard<'_, SocketTable> {
    t.lock().unwrap()
}

// ── Windows socket FFI ──────────────────────────────────────────────

/// WSAPOLLFD — mirrors the Winsock struct for [`WSAPoll`].
#[cfg(windows)]
#[repr(C)]
struct WsaPollFd {
    fd: usize, // SOCKET (UINT_PTR)
    events: i16,
    revents: i16,
}

#[cfg(windows)]
const INVALID_SOCKET: usize = !0;

#[cfg(windows)]
unsafe extern "system" {
    fn WSAPoll(fdarray: *mut WsaPollFd, nfds: u32, timeout: i32) -> i32;
    fn WSAGetLastError() -> i32;
    #[link_name = "getsockopt"]
    fn ws2_getsockopt(s: usize, level: i32, optname: i32, optval: *mut u8, optlen: *mut i32)
    -> i32;
    #[link_name = "setsockopt"]
    fn ws2_setsockopt(s: usize, level: i32, optname: i32, optval: *const u8, optlen: i32) -> i32;
}

/// Translate a Winsock error code to a Linux errno value.
///
/// The guest is a Linux unikernel (Unikraft) and expects POSIX errno
/// semantics.  Windows CRT errno values (< 200) use the same numbering
/// as POSIX; Winsock error codes (10000+) need manual translation.
#[cfg(windows)]
fn winsock_to_posix(code: i32) -> i32 {
    if code < 200 {
        return code; // CRT errno — same numbering as POSIX
    }
    match code {
        10004 => 4,   // WSAEINTR        -> EINTR
        10009 => 9,   // WSAEBADF        -> EBADF
        10013 => 13,  // WSAEACCES       -> EACCES
        10014 => 14,  // WSAEFAULT       -> EFAULT
        10022 => 22,  // WSAEINVAL       -> EINVAL
        10024 => 24,  // WSAEMFILE       -> EMFILE
        10035 => 11,  // WSAEWOULDBLOCK  -> EAGAIN
        10036 => 115, // WSAEINPROGRESS  -> EINPROGRESS
        10037 => 114, // WSAEALREADY     -> EALREADY
        10038 => 88,  // WSAENOTSOCK     -> ENOTSOCK
        10048 => 98,  // WSAEADDRINUSE   -> EADDRINUSE
        10049 => 99,  // WSAEADDRNOTAVAIL-> EADDRNOTAVAIL
        10050 => 100, // WSAENETDOWN     -> ENETDOWN
        10051 => 101, // WSAENETUNREACH  -> ENETUNREACH
        10053 => 103, // WSAECONNABORTED -> ECONNABORTED
        10054 => 104, // WSAECONNRESET   -> ECONNRESET
        10055 => 105, // WSAENOBUFS      -> ENOBUFS
        10056 => 106, // WSAEISCONN      -> EISCONN
        10057 => 107, // WSAENOTCONN     -> ENOTCONN
        10060 => 110, // WSAETIMEDOUT    -> ETIMEDOUT
        10061 => 111, // WSAECONNREFUSED -> ECONNREFUSED
        10065 => 113, // WSAEHOSTUNREACH -> EHOSTUNREACH
        _ => 5,       // EIO
    }
}

/// Translate Linux poll event bits to Winsock WSAPOLLFD event bits.
///
/// The guest sends Linux `<poll.h>` constants; Winsock defines different
/// values for the same concepts.
#[cfg(windows)]
fn poll_events_to_win(linux: i16) -> i16 {
    let mut win: i16 = 0;
    // POLLIN  (Linux 0x0001) → POLLRDNORM|POLLRDBAND (Win 0x0300)
    if linux & 0x0001 != 0 {
        win |= 0x0300;
    }
    // POLLPRI (Linux 0x0002) → POLLPRI (Win 0x0400)
    if linux & 0x0002 != 0 {
        win |= 0x0400;
    }
    // POLLOUT (Linux 0x0004) → POLLWRNORM (Win 0x0010)
    if linux & 0x0004 != 0 {
        win |= 0x0010;
    }
    win
}

/// Translate Winsock WSAPOLLFD revents back to Linux constants.
#[cfg(windows)]
fn poll_revents_to_linux(win: i16) -> i16 {
    let mut linux: i16 = 0;
    // POLLRDNORM|POLLRDBAND (Win 0x0300) → POLLIN (Linux 0x0001)
    if win & 0x0300 != 0 {
        linux |= 0x0001;
    }
    // POLLPRI (Win 0x0400) → POLLPRI (Linux 0x0002)
    if win & 0x0400 != 0 {
        linux |= 0x0002;
    }
    // POLLWRNORM|POLLWRBAND (Win 0x0030) → POLLOUT (Linux 0x0004)
    if win & 0x0030 != 0 {
        linux |= 0x0004;
    }
    // POLLERR (Win 0x0001) → POLLERR (Linux 0x0008)
    if win & 0x0001 != 0 {
        linux |= 0x0008;
    }
    // POLLHUP (Win 0x0002) → POLLHUP (Linux 0x0010)
    if win & 0x0002 != 0 {
        linux |= 0x0010;
    }
    // POLLNVAL (Win 0x0004) → POLLNVAL (Linux 0x0020)
    if win & 0x0004 != 0 {
        linux |= 0x0020;
    }
    linux
}

/// Translate Linux socket option (level, optname) to Windows equivalents.
///
/// The guest sends Linux constants; Winsock uses different values for
/// `SOL_SOCKET` options.  `IPPROTO_TCP` options are the same on both.
///
/// For unrecognised `SOL_SOCKET` options the level is still translated
/// to `0xFFFF` — leaving it as `1` (Linux `SOL_SOCKET`) is an invalid
/// protocol level on Windows and causes `WSAEINVAL`.
#[cfg(windows)]
fn translate_sockopt(level: i32, optname: i32) -> (i32, i32) {
    const LINUX_SOL_SOCKET: i32 = 1;
    const WIN_SOL_SOCKET: i32 = 0xFFFF_i32;
    match (level, optname) {
        (LINUX_SOL_SOCKET, 1) => (WIN_SOL_SOCKET, 1), // SO_DEBUG
        (LINUX_SOL_SOCKET, 2) => (WIN_SOL_SOCKET, 4), // SO_REUSEADDR
        (LINUX_SOL_SOCKET, 3) => (WIN_SOL_SOCKET, 0x1008), // SO_TYPE
        (LINUX_SOL_SOCKET, 4) => (WIN_SOL_SOCKET, 0x1007), // SO_ERROR
        (LINUX_SOL_SOCKET, 5) => (WIN_SOL_SOCKET, 0x0010), // SO_DONTROUTE
        (LINUX_SOL_SOCKET, 6) => (WIN_SOL_SOCKET, 0x0020), // SO_BROADCAST
        (LINUX_SOL_SOCKET, 7) => (WIN_SOL_SOCKET, 0x1001), // SO_SNDBUF
        (LINUX_SOL_SOCKET, 8) => (WIN_SOL_SOCKET, 0x1002), // SO_RCVBUF
        (LINUX_SOL_SOCKET, 9) => (WIN_SOL_SOCKET, 8), // SO_KEEPALIVE
        (LINUX_SOL_SOCKET, 10) => (WIN_SOL_SOCKET, 0x0100), // SO_OOBINLINE
        (LINUX_SOL_SOCKET, 13) => (WIN_SOL_SOCKET, 0x0080), // SO_LINGER
        (LINUX_SOL_SOCKET, 15) => (WIN_SOL_SOCKET, 4), // SO_REUSEPORT -> SO_REUSEADDR
        (LINUX_SOL_SOCKET, 20) => (WIN_SOL_SOCKET, 0x1006), // SO_RCVTIMEO
        (LINUX_SOL_SOCKET, 21) => (WIN_SOL_SOCKET, 0x1005), // SO_SNDTIMEO
        // Unrecognised SOL_SOCKET option — still translate the level.
        (LINUX_SOL_SOCKET, _) => (WIN_SOL_SOCKET, optname),
        // IPPROTO_TCP, IPPROTO_IPV6, etc. share the same level/optname
        // numbering on both platforms.
        _ => (level, optname),
    }
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
                2 => Domain::IPV4,
                10 => Domain::IPV6,
                _ => return Ok(-libc::EAFNOSUPPORT),
            };
            let sock_type = match ty & 0xFF {
                1 => Type::STREAM,
                2 => Type::DGRAM,
                _ => return Ok(-libc::EPROTONOSUPPORT),
            };
            let protocol = if proto == 0 {
                None
            } else {
                Some(Protocol::from(proto))
            };

            match Socket::new(domain, sock_type, protocol) {
                Ok(sock) => {
                    // Windows TCP buffers default to 8 KB send / 64 KB recv.
                    // The guest uses cooperative threading, so a blocking
                    // send() pauses the entire VM — if the combined buffer
                    // space is smaller than the payload, the receiver thread
                    // never runs and the send deadlocks.  256 KB per buffer
                    // matches Linux auto-tuning defaults and avoids this.
                    #[cfg(windows)]
                    {
                        let _ = sock.set_send_buffer_size(256 * 1024);
                        let _ = sock.set_recv_buffer_size(256 * 1024);
                    }
                    let mut tbl = lock(&tbl);
                    match tbl.insert(sock) {
                        Ok(fd) => Ok(fd),
                        Err(e) => Ok(-e),
                    }
                }
                Err(e) => Ok(neg_errno(e)),
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
                None => return Ok(-libc::EINVAL),
            };
            // Enforce listen-port allowlist (skip for port 0 = ephemeral).
            if sa.port() != 0
                && let Some(ref lp) = lp
                && lp.check(sa.port()).is_err()
            {
                return Ok(-libc::EACCES);
            }
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(match sock.bind(&SockAddr::from(sa)) {
                    Ok(()) => 0,
                    Err(e) => neg_errno(e),
                }),
                None => Ok(-libc::EBADF),
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
                Some(sock) => Ok(match sock.listen(backlog) {
                    Ok(()) => 0,
                    Err(e) => neg_errno(e),
                }),
                None => Ok(-libc::EBADF),
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
                        Err(e) => return Ok(errno_vec(e)),
                    },
                    None => return Ok({ -libc::EBADF }.to_le_bytes().to_vec()),
                }
            };
            // Lock is released — safe to block on accept.
            let (new_sock, peer) = match listener.accept() {
                Ok(pair) => pair,
                Err(e) => return Ok(errno_vec(e)),
            };
            let new_fd = {
                let mut tbl_guard = lock(&tbl);
                match tbl_guard.insert(new_sock) {
                    Ok(fd) => fd,
                    Err(e) => return Ok({ -e }.to_le_bytes().to_vec()),
                }
            };
            let mut buf = Vec::with_capacity(32);
            buf.extend(new_fd.to_le_bytes());
            if let Some(sa) = peer.as_socket() {
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
                None => return Ok(-libc::EINVAL),
            };
            // Enforce network policy.
            if let Some(ref pol) = pol
                && pol.check(&sa).is_err()
            {
                return Ok(-libc::EACCES);
            }
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(match sock.connect(&SockAddr::from(sa)) {
                    Ok(()) => 0,
                    // Non-blocking connect in progress — treat as success.
                    #[cfg(unix)]
                    Err(e) if e.raw_os_error() == Some(libc::EINPROGRESS) => 0,
                    #[cfg(windows)]
                    Err(e) if e.raw_os_error() == Some(10035) => 0, // WSAEWOULDBLOCK
                    #[cfg(windows)]
                    Err(e) if e.raw_os_error() == Some(10036) => 0, // WSAEINPROGRESS
                    Err(e) => neg_errno(e),
                }),
                None => Ok(-libc::EBADF),
            }
        },
    )
}

/// `net_send(fd, data) -> bytes_sent or -errno`
fn reg_send(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_send",
        move |fd: i32, data: Vec<u8>| -> hyperlight_host::Result<i32> {
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(match sock.send(&data) {
                    Ok(n) => n as i32,
                    Err(e) => neg_errno(e),
                }),
                None => Ok(-libc::EBADF),
            }
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
                None => return Ok(-libc::EINVAL),
            };
            // Enforce network policy.
            if let Some(ref pol) = pol
                && pol.check(&sa).is_err()
            {
                return Ok(-libc::EACCES);
            }
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(match sock.send_to(&data, &SockAddr::from(sa)) {
                    Ok(n) => n as i32,
                    Err(e) => neg_errno(e),
                }),
                None => Ok(-libc::EBADF),
            }
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
                        Err(e) => return Ok(errno_vec(e)),
                    },
                    None => return Ok({ -libc::EBADF }.to_le_bytes().to_vec()),
                }
            };
            // Lock is released — safe to block on recv.
            let mut recv_buf = vec![MaybeUninit::uninit(); len];
            match sock_clone.recv_from(&mut recv_buf) {
                Ok((n, src_addr)) => {
                    let data: Vec<u8> = recv_buf[..n]
                        .iter()
                        .map(|b| unsafe { b.assume_init() })
                        .collect();

                    // Learn IPs from DNS responses when using AllowList.
                    if let Some(ref pol) = pol
                        && let NetworkPolicy::AllowList(ref al) = **pol
                        && let Some(sa) = src_addr.as_socket()
                        && sa.port() == 53
                    {
                        net_policy::learn_ips_from_dns_response(&data, al);
                    }

                    let mut buf = Vec::with_capacity(16 + n);
                    buf.extend((n as i32).to_le_bytes());
                    if let Some(sa) = src_addr.as_socket() {
                        pack_addr(&mut buf, &sa);
                    } else {
                        buf.extend(0i32.to_le_bytes()); // family
                        buf.extend(0u16.to_le_bytes()); // port
                        buf.push(0); // addr_len
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
                0 => std::net::Shutdown::Read,
                1 => std::net::Shutdown::Write,
                _ => std::net::Shutdown::Both,
            };
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(match sock.shutdown(shut) {
                    Ok(()) => 0,
                    Err(e) => neg_errno(e),
                }),
                None => Ok(-libc::EBADF),
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
            tbl.remove(fd); // Socket::drop closes the fd
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
                Some(sock) => match sock.peer_addr() {
                    Ok(a) => Ok(addr_result(&a)),
                    Err(e) => Ok(errno_vec(e)),
                },
                None => Ok({ -libc::EBADF }.to_le_bytes().to_vec()),
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
                Some(sock) => match sock.local_addr() {
                    Ok(a) => Ok(addr_result(&a)),
                    Err(e) => Ok(errno_vec(e)),
                },
                None => Ok({ -libc::EBADF }.to_le_bytes().to_vec()),
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
                Some(sock) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::io::AsRawFd;
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
                            Ok(neg_errno(std::io::Error::last_os_error()))
                        } else {
                            Ok(val)
                        }
                    }
                    #[cfg(windows)]
                    {
                        let raw_sock = sock.as_raw_socket() as usize;
                        let (win_level, win_optname) = translate_sockopt(level, optname);
                        let mut val: i32 = 0;
                        let mut len: i32 = std::mem::size_of::<i32>() as i32;
                        let ret = unsafe {
                            ws2_getsockopt(
                                raw_sock,
                                win_level,
                                win_optname,
                                &mut val as *mut i32 as *mut u8,
                                &mut len,
                            )
                        };
                        if ret != 0 {
                            let err = unsafe { WSAGetLastError() };
                            Ok(-winsock_to_posix(err))
                        } else {
                            // SO_ERROR returns a Winsock error code — translate
                            // to Linux errno for the guest.
                            if level == 1 && optname == 4 {
                                val = winsock_to_posix(val);
                            }
                            Ok(val)
                        }
                    }
                }
                None => Ok(-libc::EBADF),
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
                Some(sock) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::io::AsRawFd;
                        let ret = unsafe {
                            libc::setsockopt(
                                sock.as_raw_fd(),
                                level,
                                optname,
                                &value as *const i32 as *const libc::c_void,
                                std::mem::size_of::<i32>() as libc::socklen_t,
                            )
                        };
                        if ret < 0 {
                            Ok(neg_errno(std::io::Error::last_os_error()))
                        } else {
                            Ok(0)
                        }
                    }
                    #[cfg(windows)]
                    {
                        let raw_sock = sock.as_raw_socket() as usize;
                        let (win_level, win_optname) = translate_sockopt(level, optname);
                        let ret = unsafe {
                            ws2_setsockopt(
                                raw_sock,
                                win_level,
                                win_optname,
                                &value as *const i32 as *const u8,
                                std::mem::size_of::<i32>() as i32,
                            )
                        };
                        if ret != 0 {
                            let err = unsafe { WSAGetLastError() };
                            Ok(-winsock_to_posix(err))
                        } else {
                            Ok(0)
                        }
                    }
                }
                None => Ok(-libc::EBADF),
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

            #[cfg(unix)]
            {
                use std::os::unix::io::AsRawFd;
                let tbl = lock(&tbl);
                let mut fds: Vec<libc::pollfd> = (0..nfds)
                    .map(|i| {
                        let off = i * 8;
                        let guest_fd =
                            i32::from_le_bytes(pollfds_raw[off..off + 4].try_into().unwrap());
                        let events =
                            i16::from_le_bytes(pollfds_raw[off + 4..off + 6].try_into().unwrap());
                        let raw_fd = tbl.get(guest_fd).map(|s| s.as_raw_fd()).unwrap_or(-1);
                        libc::pollfd {
                            fd: raw_fd,
                            events,
                            revents: 0,
                        }
                    })
                    .collect();
                drop(tbl); // release lock during blocking poll

                let ret =
                    unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout_ms) };

                let mut buf = Vec::with_capacity(4 + nfds * 2);
                if ret < 0 {
                    buf.extend(neg_errno(std::io::Error::last_os_error()).to_le_bytes());
                } else {
                    buf.extend((ret as i32).to_le_bytes());
                }
                for pfd in &fds {
                    buf.extend(pfd.revents.to_le_bytes());
                }
                Ok(buf)
            }

            #[cfg(windows)]
            {
                let tbl = lock(&tbl);
                let mut fds: Vec<WsaPollFd> = (0..nfds)
                    .map(|i| {
                        let off = i * 8;
                        let guest_fd =
                            i32::from_le_bytes(pollfds_raw[off..off + 4].try_into().unwrap());
                        let events =
                            i16::from_le_bytes(pollfds_raw[off + 4..off + 6].try_into().unwrap());
                        let raw_sock = tbl
                            .get(guest_fd)
                            .map(|s| s.as_raw_socket() as usize)
                            .unwrap_or(INVALID_SOCKET);
                        WsaPollFd {
                            fd: raw_sock,
                            events: poll_events_to_win(events),
                            revents: 0,
                        }
                    })
                    .collect();
                drop(tbl); // release lock during blocking poll

                let ret = unsafe { WSAPoll(fds.as_mut_ptr(), fds.len() as u32, timeout_ms) };

                let mut buf = Vec::with_capacity(4 + nfds * 2);
                if ret < 0 {
                    let err = unsafe { WSAGetLastError() };
                    buf.extend((-winsock_to_posix(err)).to_le_bytes());
                } else {
                    buf.extend((ret as i32).to_le_bytes());
                }
                for pfd in &fds {
                    buf.extend(poll_revents_to_linux(pfd.revents).to_le_bytes());
                }
                Ok(buf)
            }
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

/// Build a successful address result: status=0 + packed addr.
fn addr_result(sa: &SockAddr) -> Vec<u8> {
    match sa.as_socket() {
        Some(addr) => {
            let mut buf = Vec::with_capacity(24);
            buf.extend(0i32.to_le_bytes());
            pack_addr(&mut buf, &addr);
            buf
        }
        None => { -libc::EAFNOSUPPORT }.to_le_bytes().to_vec(),
    }
}

/// Encode an I/O error as a 4-byte `-errno` Vec.
fn errno_vec(e: std::io::Error) -> Vec<u8> {
    neg_errno(e).to_le_bytes().to_vec()
}

/// Convert an I/O error to `-errno` as i32.
///
/// On Unix, `raw_os_error()` returns POSIX errno values directly.
/// On Windows, it returns Winsock/Win32 error codes that must be
/// translated to POSIX values for the Unikraft guest.
fn neg_errno(e: std::io::Error) -> i32 {
    match e.raw_os_error() {
        #[cfg(unix)]
        Some(code) => -code,
        #[cfg(windows)]
        Some(code) => -winsock_to_posix(code),
        None => -libc::EIO,
    }
}
