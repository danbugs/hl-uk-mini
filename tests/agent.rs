mod common;

use std::path::PathBuf;
use std::sync::Arc;

use common::{require_rootfs, snapshot_dir};
use hyperlight_unikraft::{
    Exec, NetworkPolicy, OciTag, SNAPSHOT_TAG, Snapshot, create_sandbox, init, restore, run,
};

// ── Agent (full) tests ───────────────────────────────────────────

#[test]
fn agent_hello() {
    let rootfs = require_rootfs("agent");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/hello.py");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
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
    let rootfs = require_rootfs("agent");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/data_science.py");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
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
    let rootfs = require_rootfs("agent");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/shell_commands.py");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
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
    let rootfs = require_rootfs("agent");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(
        &mut sandbox,
        r#"
import ssl
print(f'ssl={ssl.OPENSSL_VERSION}')
import sqlite3
print('sqlite3-ok')
import ctypes
print('ctypes-ok')
"#,
    )
    .unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("ssl="),
        "SSL module not available: {output:?}"
    );
    assert!(
        output.contains("sqlite3-ok"),
        "sqlite3 not available: {output:?}"
    );
    assert!(
        output.contains("ctypes-ok"),
        "ctypes not available: {output:?}"
    );
}

#[test]
fn agent_snapshot_round_trip() {
    let rootfs = require_rootfs("agent");
    let snap_dir = snapshot_dir("agent-snap");

    // Save
    let (usandbox, _cfg) =
        create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();
    // Drop the save sandbox before restore — with HYPERLIGHT_MAX_SURROGATES=0
    // only one WHP VM can exist at a time.
    drop(sandbox);

    // Restore + run hello.py from snapshot
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, Vec::new(), None, None).unwrap();
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/hello.py");
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
    let rootfs = require_rootfs("agent");
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/verify_packages.py");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 1536, Vec::new(), None, None).unwrap();
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
    let rootfs = require_rootfs("agent");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/pip_install.py");
    let (usandbox, cfg) = create_sandbox(
        &Some(rootfs),
        &None,
        1536,
        Vec::new(),
        Some(NetworkPolicy::AllowAll),
        None,
    )
    .unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(script)).unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("Success!"),
        "expected pip_install.py to install and import six, got: {output:?}",
    );
}

// ── Agent-slim tests ─────────────────────────────────────────────

#[test]
fn agent_slim_hello() {
    let rootfs = require_rootfs("agent-slim");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/hello.py");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
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
    let rootfs = require_rootfs("agent-slim");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(
        &mut sandbox,
        r#"
import ssl
print(f'ssl={ssl.OPENSSL_VERSION}')
import sqlite3
print('sqlite3-ok')
import ctypes
print('ctypes-ok')
"#,
    )
    .unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("ssl="),
        "SSL not available in agent-slim: {output:?}"
    );
    assert!(
        output.contains("sqlite3-ok"),
        "sqlite3 not available in agent-slim: {output:?}"
    );
    assert!(
        output.contains("ctypes-ok"),
        "ctypes not available in agent-slim: {output:?}"
    );
}

#[test]
fn agent_slim_shell_subprocess() {
    let rootfs = require_rootfs("agent-slim");
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/shell_commands.py");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
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
    let rootfs = require_rootfs("agent-slim");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    // agent-slim should NOT have numpy — it's the slim rootfs
    run(
        &mut sandbox,
        r#"
try:
    import numpy
    print('numpy-found')
except ImportError:
    print('numpy-missing-ok')
"#,
    )
    .unwrap();
    let output = cfg.drain_output();
    assert!(
        output.contains("numpy-missing-ok"),
        "agent-slim should NOT have numpy, got: {output:?}",
    );
}

#[test]
fn agent_slim_snapshot_round_trip() {
    let rootfs = require_rootfs("agent-slim");
    let snap_dir = snapshot_dir("agent-slim-snap");

    // Save
    let (usandbox, _cfg) =
        create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    let snap = sandbox.snapshot().unwrap();
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    snap.save(&snap_dir, &tag).unwrap();
    drop(sandbox);

    // Restore + run hello.py from snapshot
    let tag: OciTag = SNAPSHOT_TAG.parse().unwrap();
    let snap = Arc::new(Snapshot::load(&snap_dir, tag).unwrap());
    let (mut sandbox, cfg2) = restore(snap, Vec::new(), None, None).unwrap();
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/hello.py");
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
    let rootfs = require_rootfs("agent-custom");
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/agent/custom/hello_flask.py");
    let (usandbox, cfg) =
        create_sandbox(&Some(rootfs), &None, 256, Vec::new(), None, None).unwrap();
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
