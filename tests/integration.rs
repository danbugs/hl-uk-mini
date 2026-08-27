//! Integration tests for hyperlight-unikraft.
//!
//! These require pre-built rootfs CPIOs in `build-elfloader/`.
//! Run `just build-rootfs python` and `just build-rootfs node` first.
//! Tests that need a missing rootfs are skipped automatically.

use std::path::PathBuf;
use std::sync::Arc;

use hyperlight_unikraft::{
    Exec, SNAPSHOT_TAG,
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
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "print('hluk-test-ok')").unwrap();
}

#[test]
fn python_exec_file() {
    let rootfs = require_rootfs!("python");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/python/hello.py");
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
}

#[test]
fn python_snapshot_round_trip() {
    let rootfs = require_rootfs!("python");
    let snap_dir = snapshot_dir("py-snap");

    // Save
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore + run
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let mut sandbox = restore(snap).unwrap();
    run(&mut sandbox, "print('restored-ok')").unwrap();

    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn python_multiple_runs() {
    let rootfs = require_rootfs!("python");
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "x = 1 + 1").unwrap();
    run(&mut sandbox, "print(f'x={x}')").unwrap();
    run(&mut sandbox, "import sys; print(sys.version)").unwrap();
}

// ── Node.js tests ──────────────────────────────────────────────────

#[test]
fn node_exec_file() {
    let rootfs = require_rootfs!("node");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/node/hello.js");
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 512).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
}

#[test]
fn node_inline_code() {
    let rootfs = require_rootfs!("node");
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 512).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "console.log('hluk-node-ok')").unwrap();
}

// ── Error path tests ───────────────────────────────────────────────

#[test]
fn exec_file_not_found() {
    let rootfs = require_rootfs!("python");
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256).unwrap();
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
    );
    assert!(result.is_err());
}

#[test]
fn create_sandbox_no_initrd() {
    // Should succeed — just no rootfs mapped.
    let (usandbox, cfg) = create_sandbox(&None, &None, 256).unwrap();
    assert_eq!(cfg.initrd_base, 0);
    assert_eq!(cfg.initrd_size, 0);
    // Don't evolve — no driver to run without an initrd.
    drop(usandbox);
}
