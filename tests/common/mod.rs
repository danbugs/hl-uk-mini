//! Shared helpers for integration tests.
//!
//! Each test file imports from `hyperlight_unikraft` directly;
//! this module provides only test-specific helpers.
//!
//! Items are `#[allow(dead_code)]` because each test binary re-compiles
//! this module and uses only a subset of its helpers.

use std::path::PathBuf;

use hyperlight_unikraft::{
    ListenPorts, NetworkPolicy,
    create_sandbox, init, run,
};

#[allow(dead_code)]
pub fn rootfs(runtime: &str) -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("build-elfloader/{runtime}-rootfs.cpio"));
    if path.exists() { Some(path) } else { None }
}

#[allow(dead_code)]
pub fn require_rootfs(runtime: &str) -> PathBuf {
    match rootfs(runtime) {
        Some(p) => p,
        None => {
            panic!(
                "{}-rootfs.cpio not found — run `just build-rootfs {}`",
                runtime, runtime,
            );
        }
    }
}

#[allow(dead_code)]
pub fn snapshot_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "hluk-test-{label}-{}",
        std::process::id(),
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run hluk as a subprocess with piped stdin.
#[allow(dead_code)]
pub fn hluk_with_stdin(rootfs: &std::path::Path, script: &std::path::Path, stdin_data: &[u8]) -> String {
    hluk_with_stdin_scratch(rootfs, script, stdin_data, 256)
}

#[allow(dead_code)]
pub fn hluk_with_stdin_scratch(
    rootfs: &std::path::Path,
    script: &std::path::Path,
    stdin_data: &[u8],
    scratch_mb: u32,
) -> String {
    use std::process::{Command, Stdio};
    use std::io::Write;

    let bin = env!("CARGO_BIN_EXE_hluk");
    let mut child = Command::new(bin)
        .args([
            "run",
            "--initrd", rootfs.to_str().unwrap(),
            "--scratch-mb", &scratch_mb.to_string(),
        ])
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn hluk");

    // Write data then close stdin (sends EOF to the guest).
    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(stdin_data).ok();
    }
    child.stdin.take(); // close → EOF

    let output = child.wait_with_output().expect("hluk didn't finish");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Guest mount point for compiled binaries.
#[allow(dead_code)]
pub const BIN_MOUNT: &str = "/mnt/bin";

/// Compile a source file into the given output path.
#[allow(dead_code)]
pub fn compile_example(cmd: &str, args: &[&str], out_path: &std::path::Path) {
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

/// Run `dotnet publish` and return true on success. Suppresses build output.
#[allow(dead_code)]
pub fn dotnet_publish(proj: &std::path::Path, out_dir: &std::path::Path) -> bool {
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

/// The host's non-loopback IP.
#[allow(dead_code)]
pub fn host_ip() -> String {
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    sock.connect("8.8.8.8:80").unwrap();
    sock.local_addr().unwrap().ip().to_string()
}

/// A high port that nothing listens on.
#[allow(dead_code)]
pub const UNUSED_PORT: u16 = 19999;

/// Helper: run the policy probe inside the guest and return the output.
#[allow(dead_code)]
pub fn net_probe(
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
        include_str!("../../examples/python/net_policy_probe.py"),
    );
    let _ = run(&mut sandbox, &*code);
    cfg.drain_output()
}
