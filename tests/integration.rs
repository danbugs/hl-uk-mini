//! Integration tests for hyperlight-unikraft.
//!
//! These require pre-built rootfs CPIOs in `build-elfloader/`.
//! Run `just build-rootfs <runtime>` for each runtime under test.
//! Tests panic if the required rootfs is missing.

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
                panic!(
                    "{}-rootfs.cpio not found — run `just build-rootfs {}`",
                    $runtime, $runtime,
                );
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

#[test]
fn node_snapshot_round_trip() {
    let rootfs = require_rootfs!("node");
    let snap_dir = snapshot_dir("node-snap");
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs.clone()), &None, 512, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, Vec::new(), None, None).unwrap();
    run(&mut sandbox, "console.log('restored-node-ok')").unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("restored-node-ok"),
        "expected restored node output, got: {output:?}",
    );
    let _ = std::fs::remove_dir_all(&snap_dir);
}

// ── Bash tests ────────────────────────────────────────────────────

#[test]
fn bash_inline_code() {
    let rootfs = require_rootfs!("bash");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "echo 'hluk-bash-ok'").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("hluk-bash-ok"),
        "expected guest to print 'hluk-bash-ok', got: {output:?}",
    );
}

#[test]
fn bash_exec_file() {
    let rootfs = require_rootfs!("bash");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/bash/hello.sh");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello"),
        "expected hello.sh to produce output containing 'Hello', got: {output:?}",
    );
}

#[test]
fn bash_snapshot_round_trip() {
    let rootfs = require_rootfs!("bash");
    let snap_dir = snapshot_dir("bash-snap");

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
    run(&mut sandbox, "echo 'restored-bash-ok'").unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("restored-bash-ok"),
        "expected restored guest to print 'restored-bash-ok', got: {output:?}",
    );

    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn bash_multiple_runs() {
    let rootfs = require_rootfs!("bash");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "x=42").unwrap();
    run(&mut sandbox, "echo \"x=$x\"").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("x=42"),
        "expected 'x=42' after multiple runs, got: {output:?}",
    );
}

#[test]
fn bash_coreutils() {
    let rootfs = require_rootfs!("bash");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/bash/coreutils.sh");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();

    // Verify a representative subset of coreutils output
    assert!(output.contains("=== cat ==="), "missing cat section: {output:?}");
    assert!(output.contains("alice:admin:login"), "cat didn't print file: {output:?}");
    assert!(output.contains("=== grep admin ==="), "missing grep section: {output:?}");
    assert!(output.contains("=== sort ==="), "missing sort section: {output:?}");
    assert!(output.contains("=== awk table ==="), "missing awk section: {output:?}");
    assert!(output.contains("=== ls ==="), "missing ls section: {output:?}");
    assert!(output.contains("=== find *.txt ==="), "missing find section: {output:?}");
    assert!(output.contains("=== sed s/viewer/readonly/ ==="), "missing sed section: {output:?}");
    assert!(output.contains("=== seq ==="), "missing seq section: {output:?}");
    assert!(output.contains("=== hexdump ==="), "missing hexdump section: {output:?}");
    assert!(output.contains("Done"), "script didn't finish: {output:?}");
}

// ── .NET JIT tests ───────────────────────────────────────────────

#[test]
fn dotnet_jit_inline_code() {
    let rootfs = require_rootfs!("dotnet-jit");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 768, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "Console.WriteLine(\"hluk-dotnet-ok\");").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("hluk-dotnet-ok"),
        "expected guest to print 'hluk-dotnet-ok', got: {output:?}",
    );
}

#[test]
fn dotnet_jit_exec_file() {
    let rootfs = require_rootfs!("dotnet-jit");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/dotnet-jit/Hello.cs");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 768, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello"),
        "expected Hello.cs to produce output containing 'Hello', got: {output:?}",
    );
}

#[test]
fn dotnet_jit_snapshot_round_trip() {
    let rootfs = require_rootfs!("dotnet-jit");
    let snap_dir = snapshot_dir("dotnet-jit-snap");

    // Save
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 768, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore + run
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, Vec::new(), None, None).unwrap();
    run(&mut sandbox, "Console.WriteLine(\"restored-dotnet-ok\");").unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("restored-dotnet-ok"),
        "expected restored guest to print 'restored-dotnet-ok', got: {output:?}",
    );

    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn dotnet_jit_multiple_runs() {
    let rootfs = require_rootfs!("dotnet-jit");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 768, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    // Each dispatch is an independent Roslyn compilation — no shared state.
    run(&mut sandbox, "Console.WriteLine(\"run1-ok\");").unwrap();
    let output1 = cfg.drain_output();
    assert!(
        output1.contains("run1-ok"),
        "expected 'run1-ok' from first dispatch, got: {output1:?}",
    );

    run(&mut sandbox, "Console.WriteLine($\"x={1 + 1}\");").unwrap();
    let output2 = cfg.drain_output();
    assert!(
        output2.contains("x=2"),
        "expected 'x=2' from second dispatch, got: {output2:?}",
    );
}

// ── Tier 3 (compiled language) tests ─────────────────────────────
//
// These compile the example source at test time, mount the build
// directory into the guest via hostfs, and dispatch the guest path —
// the same `--mount` + `--exec` pattern users use on the CLI.

/// Guest mount point for compiled binaries.
const BIN_MOUNT: &str = "/mnt/bin";

/// Compile a source file into the given output path.
fn compile_example(cmd: &str, args: &[&str], out_path: &std::path::Path) {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("{cmd} failed to start: {e}"));
    assert!(
        output.status.success(),
        "{cmd} failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(out_path.exists(), "compiler didn't produce {}", out_path.display());
}

#[test]
fn c_hello() {
    let rootfs = require_rootfs!("c");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/c/hello.c");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-c-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello");
    compile_example("gcc", &[
        "-O2", "-Wall", "-static-pie", "-fPIE",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from C on Hyperlight"),
        "expected C hello output, got: {output:?}",
    );
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn c_snapshot_round_trip() {
    let rootfs = require_rootfs!("c");
    let snap_dir = snapshot_dir("c-snap");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/c/hello.c");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-c-snap-bin-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello");
    compile_example("gcc", &[
        "-O2", "-Wall", "-static-pie", "-fPIE",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    // Save with mount configured (empty dir — binary not needed at save time)
    let empty_mount = std::env::temp_dir().join(format!(
        "hluk-test-c-snap-mount-{}", std::process::id(),
    ));
    std::fs::create_dir_all(&empty_mount).unwrap();
    let mounts_save = vec![Mount::rw(&empty_mount, BIN_MOUNT)];
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts_save, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore — mount points to directory with the binary
    let mounts_run = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, mounts_run, None, None).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("Hello from C on Hyperlight"),
        "expected C hello from snapshot, got: {output:?}",
    );

    let _ = std::fs::remove_dir_all(&build_dir);
    let _ = std::fs::remove_dir_all(&empty_mount);
    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn rust_hello() {
    let rootfs = require_rootfs!("rust");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/rust/hello.rs");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-rust-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello");
    compile_example("rustc", &[
        "-C", "opt-level=2",
        "-C", "target-feature=+crt-static",
        "-C", "relocation-model=pie",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from Rust on Hyperlight"),
        "expected Rust hello output, got: {output:?}",
    );
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn go_hello() {
    let rootfs = require_rootfs!("go");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/go/hello.go");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-go-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello");
    let status = std::process::Command::new("go")
        .args(["build", "-buildmode=pie", "-ldflags=-s -w",
               "-o", out.to_str().unwrap(), src.to_str().unwrap()])
        .env("CGO_ENABLED", "0")
        .status()
        .expect("go build failed to start");
    assert!(status.success(), "go build failed");

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 128, mounts, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from Go on Hyperlight"),
        "expected Go hello output, got: {output:?}",
    );
    let _ = std::fs::remove_dir_all(&build_dir);
}

/// Run `dotnet publish` and return true on success. Suppresses build output.
fn dotnet_publish(proj: &std::path::Path, out_dir: &std::path::Path) -> bool {
    match std::process::Command::new("dotnet")
        .args(["publish", "-c", "Release", "-r", "linux-musl-x64",
               "-v", "q", "--nologo",
               "-o", out_dir.to_str().unwrap()])
        .current_dir(proj)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

#[test]
fn dotnet_aot_hello() {
    let rootfs = require_rootfs!("dotnet-aot");
    let proj = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/dotnet-aot");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-dotnet-aot-{}", std::process::id()));

    if !dotnet_publish(&proj, &build_dir) {
        eprintln!("SKIP: dotnet publish failed");
        return;
    }

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, mounts, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/Hello").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from .NET AOT on Hyperlight"),
        "expected .NET AOT hello output, got: {output:?}",
    );
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn dotnet_aot_snapshot_round_trip() {
    let rootfs = require_rootfs!("dotnet-aot");
    let snap_dir = snapshot_dir("dotnet-aot-snap");
    let proj = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/dotnet-aot");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-dotnet-aot-snap-bin-{}", std::process::id()));

    if !dotnet_publish(&proj, &build_dir) {
        eprintln!("SKIP: dotnet publish failed");
        return;
    }

    // Save with mount configured
    let empty_mount = std::env::temp_dir().join(format!(
        "hluk-test-dotnet-aot-snap-mount-{}", std::process::id(),
    ));
    std::fs::create_dir_all(&empty_mount).unwrap();
    let mounts_save = vec![Mount::rw(&empty_mount, BIN_MOUNT)];
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256, mounts_save, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore — mount points to build directory
    let mounts_run = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, mounts_run, None, None).unwrap();
    run(&mut sandbox, "/mnt/bin/Hello").unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("Hello from .NET AOT on Hyperlight"),
        "expected .NET AOT hello from snapshot, got: {output:?}",
    );

    let _ = std::fs::remove_dir_all(&build_dir);
    let _ = std::fs::remove_dir_all(&empty_mount);
    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn rust_snapshot_round_trip() {
    let rootfs = require_rootfs!("rust");
    let snap_dir = snapshot_dir("rust-snap");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/rust/hello.rs");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-rust-snap-bin-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello");
    compile_example("rustc", &[
        "-C", "opt-level=2",
        "-C", "target-feature=+crt-static",
        "-C", "relocation-model=pie",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    // Save with mount configured
    let empty_mount = std::env::temp_dir().join(format!(
        "hluk-test-rust-snap-mount-{}", std::process::id(),
    ));
    std::fs::create_dir_all(&empty_mount).unwrap();
    let mounts_save = vec![Mount::rw(&empty_mount, BIN_MOUNT)];
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts_save, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore — mount points to build directory
    let mounts_run = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, mounts_run, None, None).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("Hello from Rust on Hyperlight"),
        "expected Rust hello from snapshot, got: {output:?}",
    );

    let _ = std::fs::remove_dir_all(&build_dir);
    let _ = std::fs::remove_dir_all(&empty_mount);
    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn go_snapshot_round_trip() {
    let rootfs = require_rootfs!("go");
    let snap_dir = snapshot_dir("go-snap");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/go/hello.go");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-go-snap-bin-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello");
    let status = std::process::Command::new("go")
        .args(["build", "-buildmode=pie", "-ldflags=-s -w",
               "-o", out.to_str().unwrap(), src.to_str().unwrap()])
        .env("CGO_ENABLED", "0")
        .status()
        .expect("go build failed to start");
    assert!(status.success(), "go build failed");

    // Save with mount configured
    let empty_mount = std::env::temp_dir().join(format!(
        "hluk-test-go-snap-mount-{}", std::process::id(),
    ));
    std::fs::create_dir_all(&empty_mount).unwrap();
    let mounts_save = vec![Mount::rw(&empty_mount, BIN_MOUNT)];
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 128, mounts_save, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore — mount points to build directory
    let mounts_run = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, mounts_run, None, None).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("Hello from Go on Hyperlight"),
        "expected Go hello from snapshot, got: {output:?}",
    );

    let _ = std::fs::remove_dir_all(&build_dir);
    let _ = std::fs::remove_dir_all(&empty_mount);
    let _ = std::fs::remove_dir_all(&snap_dir);
}

/// Mount a directory with two different C binaries, run both from
/// the same sandbox.  Proves the mount is live and multiple binaries
/// can be dispatched without rebooting.
#[test]
fn c_multi_binary_mount() {
    let rootfs = require_rootfs!("c");
    let hello_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/c/hello.c");
    let goodbye_src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/c/goodbye.c");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-c-multi-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();

    let hello_out = build_dir.join("hello");
    compile_example("gcc", &[
        "-O2", "-Wall", "-static-pie", "-fPIE",
        "-o", hello_out.to_str().unwrap(), hello_src.to_str().unwrap(),
    ], &hello_out);

    let goodbye_out = build_dir.join("goodbye");
    compile_example("gcc", &[
        "-O2", "-Wall", "-static-pie", "-fPIE",
        "-o", goodbye_out.to_str().unwrap(), goodbye_src.to_str().unwrap(),
    ], &goodbye_out);

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();

    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    run(&mut sandbox, "/mnt/bin/goodbye").unwrap();

    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from C on Hyperlight"),
        "expected hello output, got: {output:?}",
    );
    assert!(
        output.contains("Goodbye from C on Hyperlight"),
        "expected goodbye output, got: {output:?}",
    );
    let _ = std::fs::remove_dir_all(&build_dir);
}

/// C++ static-pie binary runs on the C rootfs — no separate runtime needed.
#[test]
fn cpp_hello() {
    let rootfs = require_rootfs!("c");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/c/hello_cpp.cpp");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-cpp-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello_cpp");
    compile_example("g++", &[
        "-O2", "-Wall", "-static-pie", "-fPIE",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/hello_cpp").unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from C++ on Hyperlight"),
        "expected C++ hello output, got: {output:?}",
    );
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn powershell_hello() {
    let rootfs = require_rootfs!("powershell");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/powershell/hello.ps1");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 1024, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from PowerShell on Hyperlight"),
        "expected PowerShell hello output, got: {output:?}",
    );
}

#[test]
fn powershell_snapshot_round_trip() {
    let rootfs = require_rootfs!("powershell");
    let snap_dir = snapshot_dir("powershell-snap");
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs.clone()), &None, 1024, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, Vec::new(), None, None).unwrap();
    run(&mut sandbox, "[Console]::WriteLine('restored-pwsh-ok')").unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("restored-pwsh-ok"),
        "expected restored PowerShell output, got: {output:?}",
    );
    let _ = std::fs::remove_dir_all(&snap_dir);
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
        output.contains("Status: 200"),
        "expected http_get.py to get Status: 200, got: {output:?}",
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

// ── Agent tests ───────────────────────────────────────────────────

#[test]
fn agent_hello() {
    let rootfs = require_rootfs!("agent");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/hello.py");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from the Hyperlight agent"),
        "expected agent hello.py output, got: {output:?}",
    );
}

#[test]
fn agent_data_science() {
    let rootfs = require_rootfs!("agent");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/data_science.py");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("slope") && output.contains("sklearn"),
        "expected data_science.py to run, got: {output:?}",
    );
}

#[test]
fn agent_shell_subprocess() {
    let rootfs = require_rootfs!("agent");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/shell_commands.py");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from hush shell!"),
        "expected shell_commands.py to run, got: {output:?}",
    );
}

#[test]
fn agent_ssl_available() {
    let rootfs = require_rootfs!("agent");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, r#"
import ssl
print(f'ssl={ssl.OPENSSL_VERSION}')
import sqlite3
print('sqlite3-ok')
import ctypes
print('ctypes-ok')
"#).unwrap();
    let output = cfg.drain_output();
    assert!(output.contains("ssl="), "SSL module not available: {output:?}");
    assert!(output.contains("sqlite3-ok"), "sqlite3 not available: {output:?}");
    assert!(output.contains("ctypes-ok"), "ctypes not available: {output:?}");
}

#[test]
fn agent_snapshot_round_trip() {
    let rootfs = require_rootfs!("agent");
    let snap_dir = snapshot_dir("agent-snap");

    // Save
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore + run hello.py from snapshot
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, Vec::new(), None, None).unwrap();
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/hello.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("Hello from the Hyperlight agent"),
        "expected agent hello.py to work after snapshot restore, got: {output:?}",
    );
    assert!(
        output.contains("numpy"),
        "expected numpy available after restore, got: {output:?}",
    );

    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn agent_verify_all_packages() {
    let rootfs = require_rootfs!("agent");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/verify_packages.py");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("ALL") && output.contains("packages verified"),
        "expected all packages to verify, got: {output:?}",
    );
    assert!(
        !output.contains("FAILED"),
        "some packages failed to import: {output:?}",
    );
}

#[test]
fn agent_pip_install() {
    let rootfs = require_rootfs!("agent");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/pip_install.py");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs), &None, 1536, Vec::new(),
        Some(NetworkPolicy::AllowAll), None,
    ).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Success!"),
        "expected pip_install.py to install and import six, got: {output:?}",
    );
}

// ── Agent-slim tests ──────────────────────────────────────────────

#[test]
fn agent_slim_hello() {
    let rootfs = require_rootfs!("agent-slim");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/hello.py");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from the Hyperlight agent"),
        "expected agent hello.py to run on slim, got: {output:?}",
    );
}

#[test]
fn agent_slim_ssl_available() {
    let rootfs = require_rootfs!("agent-slim");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, r#"
import ssl
print(f'ssl={ssl.OPENSSL_VERSION}')
import sqlite3
print('sqlite3-ok')
import ctypes
print('ctypes-ok')
"#).unwrap();
    let output = cfg.drain_output();
    assert!(output.contains("ssl="), "SSL not available in agent-slim: {output:?}");
    assert!(output.contains("sqlite3-ok"), "sqlite3 not available in agent-slim: {output:?}");
    assert!(output.contains("ctypes-ok"), "ctypes not available in agent-slim: {output:?}");
}

#[test]
fn agent_slim_shell_subprocess() {
    let rootfs = require_rootfs!("agent-slim");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/shell_commands.py");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Hello from hush shell!"),
        "expected shell_commands.py to work on slim, got: {output:?}",
    );
}

#[test]
fn agent_slim_no_numpy() {
    let rootfs = require_rootfs!("agent-slim");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    // agent-slim should NOT have numpy — it's the slim rootfs
    run(&mut sandbox, r#"
try:
    import numpy
    print('numpy-found')
except ImportError:
    print('numpy-missing-ok')
"#).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("numpy-missing-ok"),
        "agent-slim should NOT have numpy, got: {output:?}",
    );
}

#[test]
fn agent_slim_snapshot_round_trip() {
    let rootfs = require_rootfs!("agent-slim");
    let snap_dir = snapshot_dir("agent-slim-snap");

    // Save
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    // Restore + run hello.py from snapshot
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, Vec::new(), None, None).unwrap();
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/hello.py");
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg2.drain_output();
    assert!(
        output.contains("Hello from the Hyperlight agent"),
        "expected hello.py to work after slim snapshot restore, got: {output:?}",
    );

    let _ = std::fs::remove_dir_all(&snap_dir);
}

// ── Agent custom rootfs test ─────────────────────────────────────

#[test]
fn agent_custom_hello_flask() {
    let rootfs = require_rootfs!("agent-custom");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/agent/custom/hello_flask.py");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("flask=") && output.contains("pydantic="),
        "expected custom rootfs to have flask and pydantic, got: {output:?}",
    );
    assert!(
        output.contains("custom-rootfs-ok"),
        "expected custom rootfs test to pass, got: {output:?}",
    );
}
