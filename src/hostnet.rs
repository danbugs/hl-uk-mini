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

// ── Registration ─────────────────────────────────────────────────────

/// Register all `net_*` host functions plus `net_resolve` for DNS.
pub(crate) fn register(target: &mut impl Registerable) -> hyperlight_host::Result<()> {
    let table: Table = Arc::new(Mutex::new(SocketTable::new()));

    reg_socket(target, &table)?;
    reg_bind(target, &table)?;
    reg_listen(target, &table)?;
    reg_accept(target, &table)?;
    reg_connect(target, &table)?;
    reg_send(target, &table)?;
    reg_sendto(target, &table)?;
    reg_recvfrom(target, &table)?;
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
fn reg_bind(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_bind",
        move |fd: i32,
              family: i32,
              addr: String,
              port: i32|
              -> hyperlight_host::Result<i32> {
            let sa = match parse_addr(family, &addr, port) {
                Some(a) => a,
                None => return Ok(-libc::EINVAL),
            };
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
                    None => return Ok((-libc::EBADF as i32).to_le_bytes().to_vec()),
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
                    Err(e) => return Ok((-e as i32).to_le_bytes().to_vec()),
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
fn reg_connect(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
    t.register_host_function(
        "net_connect",
        move |fd: i32,
              family: i32,
              addr: String,
              port: i32|
              -> hyperlight_host::Result<i32> {
            let sa = match parse_addr(family, &addr, port) {
                Some(a) => a,
                None => return Ok(-libc::EINVAL),
            };
            let tbl = lock(&tbl);
            match tbl.get(fd) {
                Some(sock) => Ok(match sock.connect(&SockAddr::from(sa)) {
                    Ok(()) => 0,
                    #[cfg(unix)]
                    Err(e) if e.raw_os_error() == Some(libc::EINPROGRESS) => 0,
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
fn reg_sendto(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
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
fn reg_recvfrom(t: &mut impl Registerable, table: &Table) -> hyperlight_host::Result<()> {
    let tbl = table.clone();
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
                    None => return Ok((-libc::EBADF as i32).to_le_bytes().to_vec()),
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
                None => Ok((-libc::EBADF as i32).to_le_bytes().to_vec()),
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
                None => Ok((-libc::EBADF as i32).to_le_bytes().to_vec()),
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
                    #[cfg(not(unix))]
                    {
                        let _ = (sock, level, optname);
                        Ok(0)
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
                    #[cfg(not(unix))]
                    {
                        let _ = (sock, level, optname, value);
                        Ok(0)
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

                let ret = unsafe {
                    libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout_ms)
                };

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

            #[cfg(not(unix))]
            {
                let _ = (tbl, pollfds_raw, timeout_ms, nfds);
                Ok(0i32.to_le_bytes().to_vec())
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
        None => (-libc::EAFNOSUPPORT as i32).to_le_bytes().to_vec(),
    }
}

/// Encode an I/O error as a 4-byte `-errno` Vec.
fn errno_vec(e: std::io::Error) -> Vec<u8> {
    neg_errno(e).to_le_bytes().to_vec()
}

/// Convert an I/O error to `-errno` as i32.
fn neg_errno(e: std::io::Error) -> i32 {
    -(e.raw_os_error().unwrap_or(libc::EIO))
}
