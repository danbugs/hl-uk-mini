//! Tests for compiled-language runtimes: C, C++, Rust, Go, .NET AOT.
//!
//! These compile the example source at test time, mount the build
//! directory into the guest via hostfs, and dispatch the guest path.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use common::{compile_example, dotnet_publish, require_rootfs, snapshot_dir, BIN_MOUNT};
use hyperlight_unikraft::{
    Mount, OciTag, Snapshot, SNAPSHOT_TAG,
    create_sandbox, init, restore, run,
};

// ── C ────────────────────────────────────────────────────────────

#[test]
fn c_hello() {
    let rootfs = require_rootfs("c");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/c/hello.c");
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
    assert!(output.contains("Hello from C on Hyperlight"), "expected C hello output, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn c_snapshot_round_trip() {
    let rootfs = require_rootfs("c");
    let snap_dir = snapshot_dir("c-snap");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/c/hello.c");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-c-snap-bin-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello");
    compile_example("gcc", &[
        "-O2", "-Wall", "-static-pie", "-fPIE",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    let empty_mount = std::env::temp_dir().join(format!("hluk-test-c-snap-mount-{}", std::process::id()));
    std::fs::create_dir_all(&empty_mount).unwrap();
    let mounts_save = vec![Mount::rw(&empty_mount, BIN_MOUNT)];
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts_save, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    let mounts_run = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, mounts_run, None, None).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg2.drain_output();
    assert!(output.contains("Hello from C on Hyperlight"), "expected C hello from snapshot, got: {output:?}");

    let _ = std::fs::remove_dir_all(&build_dir);
    let _ = std::fs::remove_dir_all(&empty_mount);
    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn c_multi_binary_mount() {
    let rootfs = require_rootfs("c");
    let hello_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/c/hello.c");
    let goodbye_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/c/goodbye.c");
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
    assert!(output.contains("Hello from C on Hyperlight"), "expected hello output, got: {output:?}");
    assert!(output.contains("Goodbye from C on Hyperlight"), "expected goodbye output, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn cpp_hello() {
    let rootfs = require_rootfs("c");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/c/hello_cpp.cpp");
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
    assert!(output.contains("Hello from C++ on Hyperlight"), "expected C++ hello output, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn c_env_vars() {
    let rootfs = require_rootfs("c");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/c/env_vars.c");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-c-env-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("env_vars");
    compile_example("gcc", &[
        "-O2", "-Wall", "-static-pie", "-fPIE",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts, None, None).unwrap();
    cfg.set_env_vars(&[("MY_VAR", "hello_world"), ("DEBUG", "1"), ("GREETING", "hi there")]).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/env_vars").unwrap();
    let output = cfg.drain_output();
    assert!(output.contains("MY_VAR=hello_world"), "expected MY_VAR=hello_world, got: {output:?}");
    assert!(output.contains("DEBUG=1"), "expected DEBUG=1, got: {output:?}");
    assert!(output.contains("GREETING=hi there"), "expected GREETING=hi there, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}

// ── Rust ─────────────────────────────────────────────────────────

#[test]
fn rust_hello() {
    let rootfs = require_rootfs("rust");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/rust/hello.rs");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-rust-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello");
    compile_example("rustc", &[
        "-C", "opt-level=2", "-C", "target-feature=+crt-static", "-C", "relocation-model=pie",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg.drain_output();
    assert!(output.contains("Hello from Rust on Hyperlight"), "expected Rust hello output, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn rust_snapshot_round_trip() {
    let rootfs = require_rootfs("rust");
    let snap_dir = snapshot_dir("rust-snap");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/rust/hello.rs");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-rust-snap-bin-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("hello");
    compile_example("rustc", &[
        "-C", "opt-level=2", "-C", "target-feature=+crt-static", "-C", "relocation-model=pie",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    let empty_mount = std::env::temp_dir().join(format!("hluk-test-rust-snap-mount-{}", std::process::id()));
    std::fs::create_dir_all(&empty_mount).unwrap();
    let mounts_save = vec![Mount::rw(&empty_mount, BIN_MOUNT)];
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts_save, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    let mounts_run = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, mounts_run, None, None).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg2.drain_output();
    assert!(output.contains("Hello from Rust on Hyperlight"), "expected Rust hello from snapshot, got: {output:?}");

    let _ = std::fs::remove_dir_all(&build_dir);
    let _ = std::fs::remove_dir_all(&empty_mount);
    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn rust_env_vars() {
    let rootfs = require_rootfs("rust");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/rust/env_vars.rs");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-rust-env-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("env_vars");
    compile_example("rustc", &[
        "-C", "opt-level=2", "-C", "target-feature=+crt-static", "-C", "relocation-model=pie",
        "-o", out.to_str().unwrap(), src.to_str().unwrap(),
    ], &out);

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 64, mounts, None, None).unwrap();
    cfg.set_env_vars(&[("MY_VAR", "hello_world"), ("DEBUG", "1"), ("GREETING", "hi there")]).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/env_vars").unwrap();
    let output = cfg.drain_output();
    assert!(output.contains("MY_VAR=hello_world"), "expected MY_VAR=hello_world, got: {output:?}");
    assert!(output.contains("DEBUG=1"), "expected DEBUG=1, got: {output:?}");
    assert!(output.contains("GREETING=hi there"), "expected GREETING=hi there, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}

// ── Go ───────────────────────────────────────────────────────────

#[test]
fn go_hello() {
    let rootfs = require_rootfs("go");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/go/hello.go");
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
    assert!(output.contains("Hello from Go on Hyperlight"), "expected Go hello output, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn go_snapshot_round_trip() {
    let rootfs = require_rootfs("go");
    let snap_dir = snapshot_dir("go-snap");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/go/hello.go");
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

    let empty_mount = std::env::temp_dir().join(format!("hluk-test-go-snap-mount-{}", std::process::id()));
    std::fs::create_dir_all(&empty_mount).unwrap();
    let mounts_save = vec![Mount::rw(&empty_mount, BIN_MOUNT)];
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 128, mounts_save, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    let mounts_run = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, mounts_run, None, None).unwrap();
    run(&mut sandbox, "/mnt/bin/hello").unwrap();
    let output = cfg2.drain_output();
    assert!(output.contains("Hello from Go on Hyperlight"), "expected Go hello from snapshot, got: {output:?}");

    let _ = std::fs::remove_dir_all(&build_dir);
    let _ = std::fs::remove_dir_all(&empty_mount);
    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn go_env_vars() {
    let rootfs = require_rootfs("go");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/go/env_vars.go");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-go-env-{}", std::process::id()));
    std::fs::create_dir_all(&build_dir).unwrap();
    let out = build_dir.join("env_vars");
    let status = std::process::Command::new("go")
        .args(["build", "-buildmode=pie", "-ldflags=-s -w",
               "-o", out.to_str().unwrap(), src.to_str().unwrap()])
        .env("CGO_ENABLED", "0")
        .status()
        .expect("go build failed to start");
    assert!(status.success(), "go build failed");

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 128, mounts, None, None).unwrap();
    cfg.set_env_vars(&[("MY_VAR", "hello_world"), ("DEBUG", "1"), ("GREETING", "hi there")]).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/env_vars").unwrap();
    let output = cfg.drain_output();
    assert!(output.contains("MY_VAR=hello_world"), "expected MY_VAR=hello_world, got: {output:?}");
    assert!(output.contains("DEBUG=1"), "expected DEBUG=1, got: {output:?}");
    assert!(output.contains("GREETING=hi there"), "expected GREETING=hi there, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}

// ── .NET AOT ─────────────────────────────────────────────────────

#[test]
fn dotnet_aot_hello() {
    let rootfs = require_rootfs("dotnet-aot");
    let proj = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/dotnet-aot");
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
    assert!(output.contains("Hello from .NET AOT on Hyperlight"), "expected .NET AOT hello output, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}

#[test]
fn dotnet_aot_snapshot_round_trip() {
    let rootfs = require_rootfs("dotnet-aot");
    let snap_dir = snapshot_dir("dotnet-aot-snap");
    let proj = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/dotnet-aot");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-dotnet-aot-snap-bin-{}", std::process::id()));

    if !dotnet_publish(&proj, &build_dir) {
        eprintln!("SKIP: dotnet publish failed");
        return;
    }

    let empty_mount = std::env::temp_dir().join(format!("hluk-test-dotnet-aot-snap-mount-{}", std::process::id()));
    std::fs::create_dir_all(&empty_mount).unwrap();
    let mounts_save = vec![Mount::rw(&empty_mount, BIN_MOUNT)];
    let (usandbox, _cfg) = create_sandbox(&Some(rootfs), &None, 256, mounts_save, None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();

    let mounts_run = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, mounts_run, None, None).unwrap();
    run(&mut sandbox, "/mnt/bin/Hello").unwrap();
    let output = cfg2.drain_output();
    assert!(output.contains("Hello from .NET AOT on Hyperlight"), "expected .NET AOT hello from snapshot, got: {output:?}");

    let _ = std::fs::remove_dir_all(&build_dir);
    let _ = std::fs::remove_dir_all(&empty_mount);
    let _ = std::fs::remove_dir_all(&snap_dir);
}

#[test]
fn dotnet_aot_env_vars() {
    let rootfs = require_rootfs("dotnet-aot");
    let proj = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/dotnet-aot-envvars");
    let build_dir = std::env::temp_dir().join(format!("hluk-test-dotnet-aot-env-{}", std::process::id()));

    if !dotnet_publish(&proj, &build_dir) {
        eprintln!("SKIP: dotnet publish failed");
        return;
    }

    let mounts = vec![Mount::rw(&build_dir, BIN_MOUNT)];
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 256, mounts, None, None).unwrap();
    cfg.set_env_vars(&[("MY_VAR", "hello_world"), ("DEBUG", "1"), ("GREETING", "hi there")]).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, "/mnt/bin/EnvVars").unwrap();
    let output = cfg.drain_output();
    assert!(output.contains("MY_VAR=hello_world"), "expected MY_VAR=hello_world, got: {output:?}");
    assert!(output.contains("DEBUG=1"), "expected DEBUG=1, got: {output:?}");
    assert!(output.contains("GREETING=hi there"), "expected GREETING=hi there, got: {output:?}");
    let _ = std::fs::remove_dir_all(&build_dir);
}
