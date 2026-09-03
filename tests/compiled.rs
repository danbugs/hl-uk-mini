//! Tests for compiled-language runtimes: C, C++, Rust, Go, .NET AOT.
//!
//! The guest binaries are Linux ELF executables built from `examples/`
//! by `just build-test-bins` into `build-elfloader/bins/<runtime>/` —
//! on Linux, the same way rootfs images are.  Each test mounts that
//! directory into the guest via hostfs and dispatches the guest path,
//! so the tests need no host toolchain and run on every host OS.

mod common;

use std::sync::Arc;

use common::{BIN_MOUNT, require_bins, require_rootfs, snapshot_dir};
use hyperlight_unikraft::{
    Mount, OciTag, SNAPSHOT_TAG, Snapshot, create_sandbox, init, restore, run,
};

/// Environment handed to the `env_vars` examples.
const ENV: &[(&str, &str)] = &[
    ("MY_VAR", "hello_world"),
    ("DEBUG", "1"),
    ("GREETING", "hi there"),
];

/// Boot `runtime` with its prebuilt binaries at [`BIN_MOUNT`], run each
/// guest `path` in turn, and return the captured output.
fn run_bins(runtime: &str, scratch_mb: usize, env: &[(&str, &str)], paths: &[&str]) -> String {
    let rootfs = require_rootfs(runtime);
    let mounts = vec![Mount::rw(require_bins(runtime), BIN_MOUNT)];
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, scratch_mb, mounts, None, None).unwrap();
    if !env.is_empty() {
        cfg.set_env_vars(env).unwrap();
    }
    let mut sandbox = init(usandbox).unwrap();
    for path in paths {
        run(&mut sandbox, *path).unwrap();
    }
    cfg.drain_output()
}

/// Snapshot `runtime` with an empty [`BIN_MOUNT`], restore it with the
/// prebuilt binaries mounted, run `path`, and return the output.
fn run_bin_from_snapshot(runtime: &str, scratch_mb: usize, path: &str) -> String {
    let rootfs = require_rootfs(runtime);
    let snap_dir = snapshot_dir(&format!("{runtime}-snap"));
    let empty_mount = snapshot_dir(&format!("{runtime}-snap-mount"));

    let mounts_save = vec![Mount::rw(&empty_mount, BIN_MOUNT)];
    let (usandbox, _cfg) =
        create_sandbox(&Some(rootfs), &None, scratch_mb, mounts_save, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    let mounts_run = vec![Mount::rw(require_bins(runtime), BIN_MOUNT)];
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, mounts_run, None, None).unwrap();
    run(&mut sandbox, path).unwrap();
    let output = cfg2.drain_output();

    let _ = std::fs::remove_dir_all(&empty_mount);
    let _ = std::fs::remove_dir_all(&snap_dir);
    output
}

fn assert_env(output: &str) {
    assert!(
        output.contains("MY_VAR=hello_world"),
        "expected MY_VAR=hello_world, got: {output:?}"
    );
    assert!(
        output.contains("DEBUG=1"),
        "expected DEBUG=1, got: {output:?}"
    );
    assert!(
        output.contains("GREETING=hi there"),
        "expected GREETING=hi there, got: {output:?}"
    );
}

// ── C ────────────────────────────────────────────────────────────

#[test]
fn c_hello() {
    let output = run_bins("c", 64, &[], &["/mnt/bin/hello"]);
    assert!(
        output.contains("Hello from C on Hyperlight"),
        "expected C hello output, got: {output:?}"
    );
}

#[test]
fn c_snapshot_round_trip() {
    let output = run_bin_from_snapshot("c", 64, "/mnt/bin/hello");
    assert!(
        output.contains("Hello from C on Hyperlight"),
        "expected C hello from snapshot, got: {output:?}"
    );
}

#[test]
fn c_multi_binary_mount() {
    let output = run_bins("c", 64, &[], &["/mnt/bin/hello", "/mnt/bin/goodbye"]);
    assert!(
        output.contains("Hello from C on Hyperlight"),
        "expected hello output, got: {output:?}"
    );
    assert!(
        output.contains("Goodbye from C on Hyperlight"),
        "expected goodbye output, got: {output:?}"
    );
}

#[test]
fn cpp_hello() {
    let output = run_bins("c", 64, &[], &["/mnt/bin/hello_cpp"]);
    assert!(
        output.contains("Hello from C++ on Hyperlight"),
        "expected C++ hello output, got: {output:?}"
    );
}

#[test]
fn c_env_vars() {
    assert_env(&run_bins("c", 64, ENV, &["/mnt/bin/env_vars"]));
}

// ── Rust ─────────────────────────────────────────────────────────

#[test]
fn rust_hello() {
    let output = run_bins("rust", 64, &[], &["/mnt/bin/hello"]);
    assert!(
        output.contains("Hello from Rust on Hyperlight"),
        "expected Rust hello output, got: {output:?}"
    );
}

#[test]
fn rust_snapshot_round_trip() {
    let output = run_bin_from_snapshot("rust", 64, "/mnt/bin/hello");
    assert!(
        output.contains("Hello from Rust on Hyperlight"),
        "expected Rust hello from snapshot, got: {output:?}"
    );
}

#[test]
fn rust_env_vars() {
    assert_env(&run_bins("rust", 64, ENV, &["/mnt/bin/env_vars"]));
}

// ── Go ───────────────────────────────────────────────────────────

#[test]
fn go_hello() {
    let output = run_bins("go", 128, &[], &["/mnt/bin/hello"]);
    assert!(
        output.contains("Hello from Go on Hyperlight"),
        "expected Go hello output, got: {output:?}"
    );
}

#[test]
fn go_snapshot_round_trip() {
    let output = run_bin_from_snapshot("go", 128, "/mnt/bin/hello");
    assert!(
        output.contains("Hello from Go on Hyperlight"),
        "expected Go hello from snapshot, got: {output:?}"
    );
}

#[test]
fn go_env_vars() {
    assert_env(&run_bins("go", 128, ENV, &["/mnt/bin/env_vars"]));
}

// ── .NET AOT ─────────────────────────────────────────────────────

#[test]
fn dotnet_aot_hello() {
    let output = run_bins("dotnet-aot", 256, &[], &["/mnt/bin/Hello"]);
    assert!(
        output.contains("Hello from .NET AOT on Hyperlight"),
        "expected .NET AOT hello output, got: {output:?}"
    );
}

#[test]
fn dotnet_aot_snapshot_round_trip() {
    let output = run_bin_from_snapshot("dotnet-aot", 256, "/mnt/bin/Hello");
    assert!(
        output.contains("Hello from .NET AOT on Hyperlight"),
        "expected .NET AOT hello from snapshot, got: {output:?}"
    );
}

#[test]
fn dotnet_aot_env_vars() {
    assert_env(&run_bins("dotnet-aot", 256, ENV, &["/mnt/bin/EnvVars"]));
}
