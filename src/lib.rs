//! Hyperlight-Unikraft — host library for running Unikraft unikernels
//! on Hyperlight.
//!
//! ```no_run
//! use hyperlight_unikraft::{create_sandbox, init, run, Exec};
//!
//! let (usandbox, _cfg) = create_sandbox(
//!     &Some("rootfs/python.cpio".into()),
//!     &None,
//!     256,
//! )?;
//! let mut sandbox = init(usandbox)?;
//! run(&mut sandbox, "print('hello')")?;
//! run(&mut sandbox, Exec::File("examples/python/hello.py".into()))?;
//! # Ok::<(), hyperlight_unikraft::hyperlight_host::HyperlightError>(())
//! ```

use std::fs::File;
use std::io::{Read as _, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub use hyperlight_host;

use hyperlight_host::{
    GuestBinary, MultiUseSandbox, UninitializedSandbox,
    func::Registerable,
    sandbox::SandboxConfiguration,
};

// Re-export snapshot types so dependents don't need hyperlight-host directly.
pub use hyperlight_host::{HostFunctions, sandbox::snapshot::{OciTag, Snapshot}};

use tracing::{debug, info};

// ── Constants ───────────────────────────────────────────────────────────

/// Embedded Unikraft app-elfloader kernel binary.
pub static KERNEL: &[u8] = include_bytes!("../kernel/elfloader_hyperlight-x86_64");

/// GPA where the initrd is mapped via `map_file_cow`.
///
/// Past the x86 LAPIC MMIO page (0xFEE0_0000) to avoid collisions
/// with KVM's in-kernel IRQCHIP reservation.
pub const INITRD_MAP_BASE: u64 = 0xFEF0_0000;

/// Default scratch memory budget in MiB.
///
/// The frame allocator gets 75% of this; the rest covers CoW faults
/// and boot overhead.  Override with `--scratch-mb` for large rootfs
/// images (e.g. Node's 100 MiB binary needs ~512 MiB).
pub const DEFAULT_SCRATCH_MB: usize = 256;

/// PEB heap size.
///
/// Only needed for the boot stack (allocated before `ukplat_mem_init`).
/// Can be dropped to 0 once the guest allocates the boot stack from
/// scratch instead.
pub const HEAP_SIZE: u64 = 0x10_0000; // 1 MiB

/// OCI tag used when saving/loading snapshots to disk.
pub const SNAPSHOT_TAG: &str = "latest";

// ── GuestConfig ─────────────────────────────────────────────────────────

/// Runtime parameters for the guest's host functions.
///
/// Built once during sandbox setup, then used to register identical
/// host functions for both the init and snapshot-restore paths.
pub struct GuestConfig {
    pub cmdline: String,
    pub scratch_size: usize,
    pub initrd_base: u64,
    pub initrd_size: u64,
}

impl GuestConfig {
    /// How much scratch memory to give the paging frame allocator (75%).
    pub fn paging_budget(&self) -> u64 {
        (self.scratch_size as u64) * 3 / 4
    }

    /// Top of the exception stack in guest virtual address space.
    pub fn exn_stack_top(&self) -> u64 {
        hyperlight_common::layout::SCRATCH_TOP_GVA as u64
            - hyperlight_common::layout::SCRATCH_TOP_EXN_STACK_OFFSET
            + 1
    }

    /// Register host functions on any [`Registerable`] target.
    ///
    /// Works for both the init path (`UninitializedSandbox`) and the
    /// snapshot-restore path (`HostFunctions`).
    pub fn register(&self, target: &mut impl Registerable) -> hyperlight_host::Result<()> {
        // Override Hyperlight's default HostPrint (which wraps output in
        // green ANSI on stdout) — send guest output to stdout uncolored.
        target.register_host_function(
            "HostPrint",
            |msg: String| -> hyperlight_host::Result<i32> {
                print!("{msg}");
                Ok(msg.len() as i32)
            },
        )?;

        let cmdline = self.cmdline.clone();
        target.register_host_function(
            "GetCmdLine",
            move || -> hyperlight_host::Result<String> { Ok(cmdline.clone()) },
        )?;

        let budget = self.paging_budget();
        target.register_host_function(
            "GetPagingBudget",
            move || -> hyperlight_host::Result<u64> { Ok(budget) },
        )?;

        let base = self.initrd_base;
        target.register_host_function(
            "GetInitrdBase",
            move || -> hyperlight_host::Result<u64> { Ok(base) },
        )?;

        let size = self.initrd_size;
        target.register_host_function(
            "GetInitrdSize",
            move || -> hyperlight_host::Result<u64> { Ok(size) },
        )?;

        let est = self.exn_stack_top();
        target.register_host_function(
            "GetExnStackTop",
            move || -> hyperlight_host::Result<u64> { Ok(est) },
        )?;

        target.register_host_function(
            "GetWallClockNs",
            || -> hyperlight_host::Result<u64> {
                Ok(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0))
            },
        )?;

        Ok(())
    }
}

// ── CPIO helpers (private) ─────────────────────────────────────────────

/// Scan a newc-format CPIO archive for a Hyperlight driver binary.
///
/// The initrd is a CPIO archive containing the guest's root filesystem.
/// This function walks entries looking for `usr/local/bin/hl_*` or
/// `usr/bin/hl_*` — the conventional path for Hyperlight driver binaries
/// (e.g. `hl_pydriver`, `hl_nodedriver`) — and returns the first match
/// as a guest-absolute path.
///
/// Used internally by [`create_sandbox`] to auto-detect the entry point
/// so callers don't need to pass `--entry` manually.
fn find_cpio_entry(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut header = [0u8; 110];

    loop {
        if file.read_exact(&mut header).is_err() {
            break;
        }

        // Every newc CPIO entry starts with magic "070701" (or "070702"
        // for CRC variant).  Anything else means corrupt or non-CPIO data.
        let magic = std::str::from_utf8(&header[0..6]).ok()?;
        if magic != "070701" && magic != "070702" {
            break;
        }

        let namesize =
            u32::from_str_radix(std::str::from_utf8(&header[94..102]).ok()?, 16).ok()?;
        let filesize =
            u64::from_str_radix(std::str::from_utf8(&header[54..62]).ok()?, 16).ok()?;

        let mut name_buf = vec![0u8; namesize as usize];
        file.read_exact(&mut name_buf).ok()?;
        let name = std::str::from_utf8(&name_buf)
            .ok()?
            .trim_end_matches('\0');

        // "TRAILER!!!" is the standard CPIO end-of-archive marker.
        if name == "TRAILER!!!" {
            break;
        }

        // Pad past filename to 4-byte boundary (CPIO alignment rule)
        let name_padding = (4 - ((110 + namesize) % 4)) % 4;
        file.seek(SeekFrom::Current(name_padding as i64)).ok()?;

        if name.starts_with("usr/local/bin/hl_") || name.starts_with("usr/bin/hl_") {
            return Some(format!("/{name}"));
        }

        // Skip file data + padding to 4-byte boundary
        let data_padding = (4 - (filesize % 4)) % 4;
        file.seek(SeekFrom::Current((filesize + data_padding) as i64))
            .ok()?;
    }

    None
}

/// Resolve the entry point: explicit value → auto-detected from initrd → None.
fn resolve_entry(entry: &Option<String>, initrd: &Option<PathBuf>) -> Option<String> {
    if let Some(e) = entry {
        return Some(e.clone());
    }
    if let Some(path) = initrd {
        if let Some(detected) = find_cpio_entry(path) {
            info!(entry = %detected, "auto-detected driver entry point");
            return Some(detected);
        }
    }
    None
}

// ── Public API ─────────────────────────────────────────────────────────

/// Create an uninitialized sandbox with host functions registered.
///
/// Uses the embedded kernel binary ([`KERNEL`]).  Returns the sandbox
/// ready for [`init`] and the [`GuestConfig`] used to register the
/// host functions.
pub fn create_sandbox(
    initrd: &Option<PathBuf>,
    entry: &Option<String>,
    scratch_mb: usize,
) -> hyperlight_host::Result<(UninitializedSandbox, GuestConfig)> {
    let scratch_size = scratch_mb * 1024 * 1024;
    let mut cfg = SandboxConfiguration::default();
    cfg.set_scratch_size(scratch_size);
    cfg.set_heap_size(HEAP_SIZE);

    let mut usandbox = UninitializedSandbox::new(GuestBinary::Buffer(KERNEL), Some(cfg))?;

    let (initrd_base, initrd_size) = if let Some(path) = initrd {
        let size = usandbox.map_file_cow(path, INITRD_MAP_BASE)?;
        info!(
            path = %path.display(),
            size,
            gpa = format_args!("{INITRD_MAP_BASE:#x}"),
            "mapped initrd",
        );
        (INITRD_MAP_BASE, size)
    } else {
        (0, 0)
    };

    let entry = resolve_entry(entry, initrd);
    let cmdline = match &entry {
        Some(e) => format!("unikraft-hyperlight {e}"),
        None => "unikraft-hyperlight".to_string(),
    };
    let config = GuestConfig {
        cmdline,
        scratch_size,
        initrd_base,
        initrd_size,
    };

    config.register(&mut usandbox)?;

    debug!(cmdline = %config.cmdline, "sandbox created");

    Ok((usandbox, config))
}

/// Initialize (evolve) a sandbox — boots the guest and returns a
/// ready-to-use multi-use sandbox.
pub fn init(usandbox: UninitializedSandbox) -> hyperlight_host::Result<MultiUseSandbox> {
    usandbox.evolve()
}

/// What to execute in the guest.
#[derive(Debug, Clone)]
pub enum Exec {
    /// Inline code string.
    Code(String),
    /// Path to a script file (read into a string before dispatch).
    File(PathBuf),
}

impl From<&str> for Exec {
    fn from(code: &str) -> Self {
        Exec::Code(code.to_string())
    }
}

impl From<String> for Exec {
    fn from(code: String) -> Self {
        Exec::Code(code)
    }
}

/// Execute code or a script file in the guest.
///
/// Dispatches to the guest's driver via the `Exec` function.
/// Accepts inline code (`"print('hi')"`) or a file path
/// (`Exec::File("hello.py".into())`).
pub fn run(
    sandbox: &mut MultiUseSandbox,
    exec: impl Into<Exec>,
) -> hyperlight_host::Result<()> {
    let code = match exec.into() {
        Exec::Code(s) => s,
        Exec::File(path) => std::fs::read_to_string(&path).map_err(|e| {
            hyperlight_host::HyperlightError::Error(format!(
                "failed to read {}: {e}",
                path.display(),
            ))
        })?,
    };
    sandbox.call::<()>("Exec", code)
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn paging_budget_is_75_percent() {
        let cfg = GuestConfig {
            cmdline: String::new(),
            scratch_size: 256 * 1024 * 1024,
            initrd_base: 0,
            initrd_size: 0,
        };
        assert_eq!(cfg.paging_budget(), 192 * 1024 * 1024);
    }

    #[test]
    fn resolve_entry_prefers_explicit() {
        let explicit = Some("/custom/bin/myapp".to_string());
        let initrd = Some(PathBuf::from("/nonexistent/initrd.cpio"));
        assert_eq!(
            resolve_entry(&explicit, &initrd),
            Some("/custom/bin/myapp".to_string())
        );
    }

    // -- CPIO parsing --
    //
    // These test the internal CPIO scanner against synthetic archives
    // to verify it correctly finds driver binaries and handles edge
    // cases (no driver, data-heavy entries, empty archives).

    /// Build a minimal newc-format CPIO entry.
    fn cpio_entry(name: &str, data: &[u8]) -> Vec<u8> {
        let namesize = name.len() + 1; // include NUL
        let filesize = data.len();
        let header = format!(
            "070701\
             00000000\
             00000000\
             00000000\
             00000000\
             00000001\
             00000000\
             {:08X}\
             00000000\
             00000000\
             00000000\
             00000000\
             {:08X}\
             00000000",
            filesize, namesize,
        );
        assert_eq!(header.len(), 110);

        let mut buf = Vec::new();
        buf.extend_from_slice(header.as_bytes());
        buf.extend_from_slice(name.as_bytes());
        buf.push(0);
        let name_pad = (4 - ((110 + namesize) % 4)) % 4;
        buf.extend(std::iter::repeat(0u8).take(name_pad));
        buf.extend_from_slice(data);
        let data_pad = (4 - (filesize % 4)) % 4;
        buf.extend(std::iter::repeat(0u8).take(data_pad));
        buf
    }

    fn cpio_trailer() -> Vec<u8> {
        cpio_entry("TRAILER!!!", &[])
    }

    fn write_cpio(dir: &Path, name: &str, entries: &[(&str, &[u8])]) -> PathBuf {
        let path = dir.join(name);
        let mut f = File::create(&path).unwrap();
        for (entry_name, data) in entries {
            f.write_all(&cpio_entry(entry_name, data)).unwrap();
        }
        f.write_all(&cpio_trailer()).unwrap();
        path
    }

    fn test_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "hl-test-{label}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn find_cpio_entry_detects_drivers() {
        let dir = test_dir("cpio-drivers");
        // pydriver in usr/local/bin
        let cpio = write_cpio(&dir, "py.cpio", &[
            ("usr/local/bin/hl_pydriver", b"ELF"),
        ]);
        assert_eq!(
            find_cpio_entry(&cpio),
            Some("/usr/local/bin/hl_pydriver".to_string())
        );

        // nodedriver in usr/bin
        let cpio = write_cpio(&dir, "node.cpio", &[
            ("usr/bin/hl_nodedriver", b"ELF"),
        ]);
        assert_eq!(
            find_cpio_entry(&cpio),
            Some("/usr/bin/hl_nodedriver".to_string())
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_cpio_entry_skips_non_drivers() {
        let dir = test_dir("cpio-nodriver");
        let big_data = vec![0xABu8; 1024];
        let cpio = write_cpio(&dir, "test.cpio", &[
            ("etc/big_config", &big_data),
            ("usr/bin/python3", b"ELF"),
        ]);
        assert_eq!(find_cpio_entry(&cpio), None);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn find_cpio_entry_skips_data_to_find_driver() {
        let dir = test_dir("cpio-skip-data");
        let big_data = vec![0xABu8; 4096];
        let cpio = write_cpio(&dir, "test.cpio", &[
            ("etc/config", &big_data),
            ("usr/local/bin/hl_pydriver", b"ELF"),
        ]);
        assert_eq!(
            find_cpio_entry(&cpio),
            Some("/usr/local/bin/hl_pydriver".to_string())
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
