//! Integration tests for hyperlight-unikraft.
//!
//! These require pre-built rootfs CPIOs in `build-elfloader/`.
//! Run `just build-rootfs python` and `just build-rootfs node` first.
//! Tests that need a missing rootfs are skipped automatically.

use std::path::PathBuf;
use std::sync::Arc;

use hyperlight_unikraft::{
    Exec, Mount, SNAPSHOT_TAG,
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
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), false).unwrap();
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
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), false).unwrap();
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
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), false).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore + run
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, Vec::new(), false).unwrap();
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
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), false).unwrap();
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
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 512, Vec::new(), false).unwrap();
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
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 512, Vec::new(), false).unwrap();
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
        &Some(rootfs), &None, 256, mounts, false,
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
        &Some(rootfs), &None, 256, mounts, false,
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
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), false).unwrap();
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
        &Some(rootfs), &None, 256, Vec::new(), false,
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
        &Some(rootfs), &None, 256, Vec::new(), true,
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
        &Some(rootfs), &None, 256, Vec::new(), true,
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
        &Some(rootfs), &None, 256, Vec::new(), true,
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
        &Some(rootfs), &None, 256, Vec::new(), true,
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

// ── Error path tests ───────────────────────────────────────────────

#[test]
fn exec_file_not_found() {
    let rootfs = require_rootfs!("python");
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), false).unwrap();
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
        false,
    );
    assert!(result.is_err());
}

#[test]
fn create_sandbox_no_initrd() {
    // Should succeed — just no rootfs mapped.
    let (usandbox, cfg) = create_sandbox(&None, &None, 256, Vec::new(), false).unwrap();
    assert_eq!(cfg.initrd_base, 0);
    assert_eq!(cfg.initrd_size, 0);
    // Don't evolve — no driver to run without an initrd.
    drop(usandbox);
}
