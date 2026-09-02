mod common;

use common::require_rootfs;
use hyperlight_unikraft::{Exec, create_sandbox, init, run};

#[test]
fn exec_file_not_found() {
    let rootfs = require_rootfs("python");
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

#[test]
fn cli_env_flag() {
    let rootfs = require_rootfs("python");
    let bin = env!("CARGO_BIN_EXE_hluk");
    let output = std::process::Command::new(bin)
        .args([
            "run",
            "--initrd", rootfs.to_str().unwrap(),
            "--scratch-mb", "256",
            "--env", "MY_VAR=cli_test",
            "--env", "NUM=42",
            "--exec", "import os; print(f\"MY_VAR={os.environ.get('MY_VAR','?')}\"); print(f\"NUM={os.environ.get('NUM','?')}\")",
        ])
        .output()
        .expect("failed to run hluk");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("MY_VAR=cli_test"),
        "expected MY_VAR=cli_test, got: {stdout:?}",
    );
    assert!(
        stdout.contains("NUM=42"),
        "expected NUM=42, got: {stdout:?}",
    );
}
