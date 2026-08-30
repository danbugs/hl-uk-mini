//! Integration tests for hyperlight-unikraft.
//!
//! These require pre-built rootfs CPIOs in `build-elfloader/`.
//! Run `just build-rootfs python` and `just build-rootfs node` first.
//! Tests that need a missing rootfs are skipped automatically.

use std::path::PathBuf;
use std::sync::Arc;

use hyperlight_unikraft::{
    AllowList, BlockList, Exec, ListenPorts, Mount, NetworkPolicy, SNAPSHOT_TAG,
    create_sandbox, init, restore, run,
    OciTag, Snapshot,
};

// ── Helpers ────────────────────────────────────────────────────────

fn rootfs(runtime: &str) -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("build-elfloader/{runtime}-rootfs.cpio"));
    if path.exists() { Some(path) } else { None }
}

fn snapshot_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "hluk-test-{label}-{}",
        std::process::id(),
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

macro_rules! require_rootfs {
    ($runtime:expr) => {
        match rootfs($runtime) {
            Some(p) => p,
            None => {
                eprintln!(
                    "SKIP: {}-rootfs.cpio not found (run `just build-rootfs {}`)",
                    $runtime, $runtime,
                );
                return;
            }
        }
    };
}

// ── Python tests ───────────────────────────────────────────────────

#[test]
fn python_inline_code() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "print('hluk-test-ok')").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("hluk-test-ok"),
        "expected guest to print 'hluk-test-ok', got: {output:?}",
    );
}

#[test]
fn python_exec_file() {
    let rootfs = require_rootfs!("python");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/hello.py");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello"),
        "expected hello.py to produce output containing 'Hello', got: {output:?}",
    );
}

#[test]
fn python_snapshot_round_trip() {
    let rootfs = require_rootfs!("python");
    let snap_dir = snapshot_dir("py-snap");

    // Save
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore + run
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, Vec::new(), None, None).unwrap();
    run(&mut sandbox, "print('restored-ok')").unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("restored-ok"),
        "expected restored guest to print 'restored-ok', got: {output:?}",
    );

    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn python_multiple_runs() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "x = 1 + 1").unwrap();
    run(&mut sandbox, "print(f'x={x}')").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("x=2"),
        "expected 'x=2' after multiple runs, got: {output:?}",
    );
    run(&mut sandbox, "import sys; print(sys.version)").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("3.12"),
        "expected sys.version to contain '3.12', got: {output:?}",
    );
}

// ── Node.js tests ──────────────────────────────────────────────────

#[test]
fn node_exec_file() {
    let rootfs = require_rootfs!("node");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/node/hello.js");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 512, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello"),
        "expected hello.js to produce output containing 'Hello', got: {output:?}",
    );
}

#[test]
fn node_inline_code() {
    let rootfs = require_rootfs!("node");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 512, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "console.log('hluk-node-ok')").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("hluk-node-ok"),
        "expected guest to print 'hluk-node-ok', got: {output:?}",
    );
}

// ── Hostfs tests ──────────────────────────────────────────────────

#[test]
fn python_fs_ops() {
    let rootfs = require_rootfs!("python");
    let mount_dir = std::env::temp_dir().join(format!(
        "hluk-fs-ops-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&mount_dir).unwrap();

    let mounts = vec![Mount::rw(&mount_dir, "/mnt/host")];
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, mounts, None, None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/fs_ops.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Cleanup done"),
        "expected fs_ops.py to print 'Cleanup done', got: {output:?}",
    );

    // The script writes sentinel.txt with known content, then cleans up
    // everything else.  Verify the sentinel is visible from the host.
    let sentinel = mount_dir.join("sentinel.txt");
    let content = std::fs::read_to_string(&sentinel)
        .expect("sentinel.txt should exist on the host after guest write");
    assert_eq!(
        content, "guest-was-here\n",
        "sentinel content mismatch — guest write not visible on host",
    );

    // Only sentinel.txt should remain (the script cleaned up the rest).
    let entries: Vec<_> = std::fs::read_dir(&mount_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        entries, vec!["sentinel.txt"],
        "expected only sentinel.txt, found: {:?}", entries,
    );

    let _ = std::fs::remove_dir_all(&mount_dir);
}

/// Writes and reads back a 96 KB file (three 32 KB chunks) through
/// hostfs to exercise multi-chunk I/O.  Catches undersized PEB I/O
/// stacks that silently truncate large transfers.
#[test]
fn python_fs_large_file() {
    let rootfs = require_rootfs!("python");
    let mount_dir = std::env::temp_dir().join(format!(
        "hluk-fs-large-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&mount_dir).unwrap();

    let mounts = vec![Mount::rw(&mount_dir, "/mnt/host")];
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, mounts, None, None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/fs_large.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("large file round-trip OK"),
        "expected fs_large.py to print 'large file round-trip OK', got: {output:?}",
    );

    // The script cleans up large.bin — verify nothing is left.
    let entries: Vec<_> = std::fs::read_dir(&mount_dir)
        .unwrap()
        .collect();
    assert!(
        entries.is_empty(),
        "fs_large.py should clean up, but {} entries remain",
        entries.len(),
    );

    let _ = std::fs::remove_dir_all(&mount_dir);
}

// ── Guest filesystem tests ────────────────────────────────────────

#[test]
fn python_guest_fs_ops() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/guest_fs.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Guest filesystem tests passed"),
        "expected guest_fs.py to print 'Guest filesystem tests passed', got: {output:?}",
    );
}

// ── Threading tests ───────────────────────────────────────────────

/// Spawns worker threads that compute partial sums and join.
#[test]
fn python_threading() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, Vec::new(), None, None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/threading_demo.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Threading demo passed"),
        "expected threading_demo.py to print 'Threading demo passed', got: {output:?}",
    );
}

// ── Subprocess tests ──────────────────────────────────────────────

/// Python spawning Python via subprocess.run() — exercises the kernel's
/// vfork+execve path.  No os.fork() needed (Python 3.12 uses posix_spawn
/// internally for subprocess).
#[test]
fn python_subprocess() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, Vec::new(), None, None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/subprocess_demo.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Subprocess demo passed"),
        "expected subprocess_demo.py to print 'Subprocess demo passed', got: {output:?}",
    );
}

// ── Hostnet tests ─────────────────────────────────────────────────

/// Runs a TCP echo server and client inside the guest using threads.
///
/// The hostsock driver's check_ready pattern returns EAGAIN on blocking
/// calls when the socket isn't ready, letting the cooperative scheduler
/// yield between the server and client threads.
#[test]
fn python_tcp_echo() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, Vec::new(), Some(NetworkPolicy::AllowAll), None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/tcp_echo.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("TCP echo test passed"),
        "expected tcp_echo.py to print 'TCP echo test passed', got: {output:?}",
    );
}

/// Bidirectional TCP — server and client exchange 128 KB each.
/// Stresses the send() POLLOUT guard: without it, the server's
/// sendall() would block the VM when the send buffer fills up.
#[test]
fn python_tcp_bidir() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, Vec::new(), Some(NetworkPolicy::AllowAll), None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/tcp_bidir.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Bidirectional TCP test passed"),
        "expected tcp_bidir.py to print 'Bidirectional TCP test passed', got: {output:?}",
    );
}

/// HTTP server+client inside the guest — exercises the threading+socket
/// pattern from CPython test_httplib.  The send() POLLOUT guard prevents
/// the deadlock that previously hung these tests.
#[test]
fn python_http_server_client() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, Vec::new(), Some(NetworkPolicy::AllowAll), None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/http_server_client.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("HTTP server+client test passed"),
        "expected http_server_client.py to print 'HTTP server+client test passed', got: {output:?}",
    );
}

/// Exercises outbound HTTP via urllib — the script handles connection
/// failures gracefully (try/except), so this test passes even without
/// real internet.  What it validates: the networking stack is wired up
/// well enough that Python's urllib can attempt a connection without
/// crashing the guest.
#[test]
fn python_http_get() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, Vec::new(), Some(NetworkPolicy::AllowAll), None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/http_get.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("HTTP GET test done"),
        "expected http_get.py to print 'HTTP GET test done', got: {output:?}",
    );
}

/// Server thread blocks in select() with infinite timeout, client
/// connects and exchanges data.  Validates the halt_irq fix: without
/// it, select() with no timeout never wakes because the idle thread
/// doesn't poll hostsock.
#[test]
fn python_threaded_select() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, Vec::new(), Some(NetworkPolicy::AllowAll), None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/threaded_select.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Threaded select() test passed"),
        "expected threaded_select.py to pass, got: {output:?}",
    );
}

// ── Error path tests ───────────────────────────────────────────────

#[test]
fn exec_file_not_found() {
    let rootfs = require_rootfs!("python");
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let result = run(&mut sandbox, Exec::File("/nonexistent/script.py".into()));
    assert!(result.is_err());
}

#[test]
fn create_sandbox_missing_initrd() {
    let result = create_sandbox(
        &Some("/nonexistent/rootfs.cpio".into()),
        &None,
        256,
        Vec::new(),
        None,
        None,
    );
    assert!(result.is_err());
}

#[test]
fn create_sandbox_no_initrd() {
    // Should succeed — just no rootfs mapped.
    let (usandbox, cfg) = create_sandbox(&None, &None, 256, Vec::new(), None, None).unwrap();
    assert_eq!(cfg.initrd_base, 0);
    assert_eq!(cfg.initrd_size, 0);
    // Don't evolve — no driver to run without an initrd.
    drop(usandbox);
}

// ── Network policy tests ──────────────────────────────────────────

/// The host's non-loopback IP.  Connecting to this IP on a non-listening
/// port gives instant ECONNREFUSED (the kernel sends a RST because no
/// socket is bound).  This lets us test "policy permits the connection"
/// without waiting for a TCP timeout to an unreachable remote IP.
///
/// Uses the standard UDP-connect-to-8.8.8.8 trick: the kernel picks the
/// source IP for the default route without sending any packets.
fn host_ip() -> String {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    sock.connect("8.8.8.8:80").unwrap();
    sock.local_addr().unwrap().ip().to_string()
}

/// A high port that nothing listens on.  Connecting to `host_ip():UNUSED_PORT`
/// returns ECONNREFUSED instantly.
const UNUSED_PORT: u16 = 19999;

/// Helper: run the policy probe inside the guest and return the output.
///
/// The probe tests both TCP connect (reg_connect) and UDP sendto
/// (reg_sendto).  Output lines: TCP_OK/TCP_BLOCKED/TCP_REFUSED/TCP_FAIL
/// and UDP_OK/UDP_BLOCKED/UDP_FAIL.
fn net_probe(
    policy: Option<NetworkPolicy>,
    listen_ports: Option<ListenPorts>,
    host: &str,
    port: u16,
) -> String {
    let rootfs = match rootfs("python") {
        Some(p) => p,
        None => return "SKIP".to_string(),
    };
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, Vec::new(), policy, listen_ports,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let code = format!(
        "HOST = {host:?}; PORT = {port}\n{}",
        include_str!("../examples/python/net_policy_probe.py"),
    );
    let _ = run(&mut sandbox, &*code);
    cfg.drain_output()
}

/// Networking disabled by default — guest socket calls fail because
/// net_* host functions aren't registered at all.
#[test]
fn net_policy_disabled_by_default() {
    let rootfs = require_rootfs!("python");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 256, Vec::new(), None, None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    // socket() calls the net_socket host function which isn't registered,
    // causing the guest to abort — run() returns an error.
    let result = run(&mut sandbox, "import socket; s = socket.socket(socket.AF_INET, socket.SOCK_STREAM); print('SOCKET_OK')");
    let output = cfg.drain_output();
    assert!(
        result.is_err() || !output.contains("SOCKET_OK"),
        "expected socket creation to fail when networking is disabled, got result={result:?}, output={output:?}",
    );
}

/// AllowAll blocks link-local addresses (cloud metadata service).
#[test]
fn net_policy_allowall_blocks_link_local() {
    let output = net_probe(
        Some(NetworkPolicy::AllowAll),
        None,
        "169.254.169.254",
        80,
    );
    if output == "SKIP" { return; }
    assert!(
        output.contains("TCP_BLOCKED"),
        "expected link-local 169.254.169.254 to be TCP-blocked even with AllowAll, got: {output:?}",
    );
    assert!(
        output.contains("UDP_BLOCKED"),
        "expected link-local 169.254.169.254 to be UDP-blocked even with AllowAll, got: {output:?}",
    );
}

/// AllowAll permits loopback — needed for intra-guest server+client
/// patterns.  In the hostsock model all guest sockets are host sockets,
/// so blocking loopback would prevent guest-internal networking.
/// AllowList/BlockList still block loopback (defense in depth).
#[test]
fn net_policy_allowall_permits_loopback() {
    let output = net_probe(
        Some(NetworkPolicy::AllowAll),
        None,
        "127.0.0.1",
        UNUSED_PORT,
    );
    if output == "SKIP" { return; }
    // Loopback should pass the policy check.  TCP gives ECONNREFUSED
    // (nothing listening), not EACCES.  UDP sendto succeeds.
    assert!(
        !output.contains("TCP_BLOCKED"),
        "expected loopback 127.0.0.1 to be TCP-permitted by AllowAll, got: {output:?}",
    );
    assert!(
        !output.contains("UDP_BLOCKED"),
        "expected loopback 127.0.0.1 to be UDP-permitted by AllowAll, got: {output:?}",
    );
}

/// AllowList permits connections to a listed IP.
///
/// Uses the host's own IP on a non-listening port — the kernel RSTs
/// immediately (ECONNREFUSED), proving the policy check passed without
/// waiting for a remote TCP handshake.
#[test]
fn net_policy_allowlist_permits() {
    let ip = host_ip();
    let al = AllowList::from_hosts(&[ip.as_str()]).unwrap();
    let output = net_probe(
        Some(NetworkPolicy::AllowList(al)),
        None,
        &ip,
        UNUSED_PORT,
    );
    if output == "SKIP" { return; }
    assert!(
        !output.contains("TCP_BLOCKED"),
        "expected allowlisted IP {ip} to pass TCP policy check, got: {output:?}",
    );
    assert!(
        !output.contains("UDP_BLOCKED"),
        "expected allowlisted IP {ip} to pass UDP policy check, got: {output:?}",
    );
}

/// AllowList blocks connections to an unlisted IP.
#[test]
fn net_policy_allowlist_blocks() {
    let al = AllowList::from_hosts(&["93.184.216.34"]).unwrap();
    let output = net_probe(
        Some(NetworkPolicy::AllowList(al)),
        None,
        "1.2.3.4",
        80,
    );
    if output == "SKIP" { return; }
    assert!(
        output.contains("TCP_BLOCKED"),
        "expected unlisted IP 1.2.3.4 to be TCP-blocked by AllowList, got: {output:?}",
    );
    assert!(
        output.contains("UDP_BLOCKED"),
        "expected unlisted IP 1.2.3.4 to be UDP-blocked by AllowList, got: {output:?}",
    );
}

/// BlockList blocks connections to a listed IP.
#[test]
fn net_policy_blocklist_blocks() {
    let bl = BlockList::from_hosts(&["1.2.3.4"]).unwrap();
    let output = net_probe(
        Some(NetworkPolicy::BlockList(bl)),
        None,
        "1.2.3.4",
        80,
    );
    if output == "SKIP" { return; }
    assert!(
        output.contains("TCP_BLOCKED"),
        "expected listed IP 1.2.3.4 to be TCP-blocked by BlockList, got: {output:?}",
    );
    assert!(
        output.contains("UDP_BLOCKED"),
        "expected listed IP 1.2.3.4 to be UDP-blocked by BlockList, got: {output:?}",
    );
}

/// BlockList permits connections to an unlisted IP.
///
/// Uses the host's own IP (same as allowlist_permits) for instant response.
#[test]
fn net_policy_blocklist_permits() {
    let ip = host_ip();
    let bl = BlockList::from_hosts(&["1.2.3.4"]).unwrap();
    let output = net_probe(
        Some(NetworkPolicy::BlockList(bl)),
        None,
        &ip,
        UNUSED_PORT,
    );
    if output == "SKIP" { return; }
    assert!(
        !output.contains("TCP_BLOCKED"),
        "expected unlisted IP {ip} to pass TCP BlockList policy check, got: {output:?}",
    );
    assert!(
        !output.contains("UDP_BLOCKED"),
        "expected unlisted IP {ip} to pass UDP BlockList policy check, got: {output:?}",
    );
}
