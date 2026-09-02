mod common;

use std::path::PathBuf;
use std::sync::Arc;

use common::{require_rootfs, snapshot_dir};
use hyperlight_unikraft::{
    Exec, OciTag, Snapshot, SNAPSHOT_TAG,
    create_sandbox, init, restore, run,
};

#[test]
fn powershell_hello() {
    let rootfs = require_rootfs("powershell");
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
    let rootfs = require_rootfs("powershell");
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

#[test]
fn powershell_env_vars() {
    let rootfs = require_rootfs("powershell");
    let (usandbox, cfg) = create_sandbox(&Some(rootfs), &None, 1024, Vec::new(), None, None).unwrap();
    cfg.set_env_vars(&[
        ("MY_VAR", "hello_world"),
        ("DEBUG", "1"),
        ("GREETING", "hi there"),
    ]).unwrap();
    let mut sandbox = init(usandbox).unwrap();
    run(&mut sandbox, Exec::File(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/powershell/env_vars.ps1"),
    )).unwrap();
    let output = cfg.drain_output();
    assert!(output.contains("MY_VAR=hello_world"), "expected MY_VAR=hello_world, got: {output:?}");
    assert!(output.contains("DEBUG=1"), "expected DEBUG=1, got: {output:?}");
    assert!(output.contains("GREETING=hi there"), "expected GREETING=hi there, got: {output:?}");
}
