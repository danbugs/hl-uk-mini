//! Hyperlight-Unikraft — host library for running Unikraft unikernels
//! on Hyperlight.
//!
//! ```no_run
//! use hyperlight_unikraft::{create_sandbox, init, run, Exec, Mount};
//!
//! // Simple sandbox — no mounts, no networking.
//! let (usandbox, cfg) = create_sandbox(
//!     &Some("rootfs/python.cpio".into()),
//!     &None,
//!     256,
//!     Vec::new(),
//!     None,
//!     None,
//! )?;
//! let mut sandbox = init(usandbox)?;
//! run(&mut sandbox, "print('hello')")?;
//! let output = cfg.drain_output();
//! assert!(output.contains("hello"));
//! # Ok::<(), hyperlight_unikraft::hyperlight_host::HyperlightError>(())
//! ```

use std::fmt::Write as _;
use std::fs::File;
use std::io::{Read as _, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub use hyperlight_host;

use hyperlight_host::{
    GuestBinary, MultiUseSandbox, UninitializedSandbox,
    func::Registerable,
    sandbox::SandboxConfiguration,
};

// Re-export snapshot types so dependents don't need hyperlight-host directly.
pub use hyperlight_host::{HostFunctions, sandbox::snapshot::{OciTag, Snapshot}};

use tracing::{debug, info};

mod hostfs;
mod hostnet;
pub mod net_policy;

pub use net_policy::{AllowList, BlockList, ListenPorts, NetworkPolicy};

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

/// PEB I/O stack size for host-call data transfer.
///
/// Both the input stack (host→guest results) and output stack
/// (guest→host calls) must hold a FlatBuffer-encoded message.  The
/// guest's generic hcall encoder (`g_generic_fc_buf`) is 64 KiB, so
/// the I/O stacks must be at least that large.  We add headroom for
/// the stack header (8 bytes) and alignment padding.
///
/// Default Hyperlight stacks are only 16 KiB — too small for large
/// file or network transfers.
const IO_STACK_SIZE: usize = 65536 + 4096;

/// PEB heap size.
///
/// Only needed for the boot stack (allocated before `ukplat_mem_init`).
/// Can be dropped to 0 once the guest allocates the boot stack from
/// scratch instead.
pub const HEAP_SIZE: u64 = 0x10_0000; // 1 MiB

/// OCI tag used when saving/loading snapshots to disk.
pub const SNAPSHOT_TAG: &str = "latest";

// ── Mount ───────────────────────────────────────────────────────────────

/// A host filesystem mount passed to the guest.
#[derive(Debug, Clone)]
pub struct Mount {
    /// Guest-visible mount point (e.g. `/mnt/data`).
    pub guest_path: String,
    /// Host directory to expose.
    pub host_path: PathBuf,
    /// Mount read-only (`true` → `MNT_RDONLY`, writes return `EROFS`).
    pub readonly: bool,
}

impl Mount {
    /// Create a read-write mount.
    ///
    /// Parameter order matches Docker convention: host (source) first,
    /// guest (target) second.
    pub fn rw(host_path: impl Into<PathBuf>, guest_path: impl Into<String>) -> Self {
        Self { guest_path: guest_path.into(), host_path: host_path.into(), readonly: false }
    }

    /// Create a read-only mount.
    ///
    /// Parameter order matches Docker convention: host (source) first,
    /// guest (target) second.
    pub fn ro(host_path: impl Into<PathBuf>, guest_path: impl Into<String>) -> Self {
        Self { guest_path: guest_path.into(), host_path: host_path.into(), readonly: true }
    }
}

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
    /// Host filesystem mounts.
    pub mounts: Vec<Mount>,
    /// Network access policy (`None` = networking disabled).
    pub network: Option<NetworkPolicy>,
    /// Ports the guest is allowed to `bind()` for inbound connections.
    pub listen_ports: Option<ListenPorts>,
    /// Captured guest stdout — accumulated by the HostPrint callback.
    output: Arc<Mutex<String>>,
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
    /// Drain captured guest output, clearing the buffer.
    pub fn drain_output(&self) -> String {
        self.output.lock().unwrap().split_off(0)
    }

    pub fn register(&self, target: &mut impl Registerable) -> hyperlight_host::Result<()> {
        // Override Hyperlight's default HostPrint (which wraps output in
        // green ANSI on stdout) — send guest output to stdout uncolored,
        // and capture it for programmatic access.
        let output = self.output.clone();
        target.register_host_function(
            "HostPrint",
            move |msg: String| -> hyperlight_host::Result<i32> {
                use std::io::Write;
                let len = msg.len() as i32;
                print!("{msg}");
                let _ = std::io::stdout().flush();
                output.lock().unwrap().push_str(&msg);
                Ok(len)
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

        target.register_host_function(
            "GetHostFsChunkSize",
            || -> hyperlight_host::Result<u64> {
                Ok(hostfs::CHUNK as u64)
            },
        )?;

        // Register per-operation host functions for filesystem and networking.
        if !self.mounts.is_empty() {
            let hfs_mounts: Vec<(String, PathBuf, bool)> = self
                .mounts
                .iter()
                .map(|m| (m.guest_path.clone(), m.host_path.clone(), m.readonly))
                .collect();
            hostfs::register(target, &hfs_mounts)?;
        }
        if self.network.is_some() {
            hostnet::register(
                target,
                self.network.clone(),
                self.listen_ports.clone(),
            )?;
        }

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
    if let Some(path) = initrd
        && let Some(detected) = find_cpio_entry(path) {
            info!(entry = %detected, "auto-detected driver entry point");
            return Some(detected);
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
    mounts: Vec<Mount>,
    network: Option<NetworkPolicy>,
    listen_ports: Option<ListenPorts>,
) -> hyperlight_host::Result<(UninitializedSandbox, GuestConfig)> {
    let scratch_size = scratch_mb * 1024 * 1024;
    let mut cfg = SandboxConfiguration::default();
    cfg.set_scratch_size(scratch_size);
    cfg.set_heap_size(HEAP_SIZE);

    cfg.set_input_data_size(IO_STACK_SIZE);
    cfg.set_output_data_size(IO_STACK_SIZE);

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

    // Build the kernel command line.
    //
    // Unikraft's uklibparam parser requires a `--` separator between
    // kernel parameters (like vfs.fstab) and application arguments
    // (like the entry point path).  Without `--`, uklibparam skips
    // parsing entirely and cmdline parameters are silently ignored.
    //
    // Layout: <progname> [kernel params...] -- [entry point]
    let mut cmdline = "unikraft-hyperlight".to_string();

    // Inject vfs.fstab entries so the kernel mounts hostfs at each
    // guest path.  The source-device field carries the mount index
    // (used by hostfs to route hcalls to the correct host Dir).
    if !mounts.is_empty() {
        cmdline.push_str(" vfs.fstab=[");
        for (i, m) in mounts.iter().enumerate() {
            if i > 0 {
                cmdline.push(' ');
            }
            // MNT_RDONLY = 0x1
            let flags = if m.readonly { "0x1" } else { "0x0" };
            // Format: sdev:path:drv:flags:opts:ukopts
            // mkmp = make mount point (creates the directory if missing)
            // No quotes — uk_libparam doesn't strip them.
            write!(cmdline, "{i}:{}:hostfs:{flags}::mkmp", m.guest_path).unwrap();
        }
        cmdline.push(']');
    }

    // Entry point path.  The `--` separator is needed only when there
    // are kernel params (like vfs.fstab) before it — uklibparam strips
    // everything up to `--` and adjusts argv so the elfloader sees the
    // driver path at argv[1].  Without kernel params, skip `--` so
    // argv[1] is the path directly (uklibparam's scan returns 0 for a
    // leading `--` and skips adjustment).
    if let Some(e) = &entry {
        if !mounts.is_empty() {
            write!(cmdline, " -- {e}").unwrap();
        } else {
            write!(cmdline, " {e}").unwrap();
        }
    }

    let config = GuestConfig {
        cmdline,
        scratch_size,
        initrd_base,
        initrd_size,
        mounts,
        network,
        listen_ports,
        output: Arc::new(Mutex::new(String::new())),
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
    /// Inline code string — passed to the guest's dispatch callback.
    Code(String),
    /// Script file — read to string and passed to the guest's dispatch callback.
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
/// Dispatches to the guest's driver callback. Accepts inline code
/// (`"print('hi')"`) or a file path (`Exec::File("hello.py".into())`).
///
/// Guest stdout is captured in the [`GuestConfig`] returned by
/// [`create_sandbox`].  Call [`GuestConfig::drain_output`] after
/// `run()` to retrieve what the guest printed.
pub fn run(
    sandbox: &mut MultiUseSandbox,
    exec: impl Into<Exec>,
) -> hyperlight_host::Result<()> {
    match exec.into() {
        Exec::Code(s) => sandbox.call::<()>("Exec", s),
        Exec::File(path) => {
            let code = std::fs::read_to_string(&path).map_err(|e| {
                hyperlight_host::HyperlightError::Error(format!(
                    "failed to read {}: {e}",
                    path.display(),
                ))
            })?;
            sandbox.call::<()>("Exec", code)
        }
    }
}

/// Restore a sandbox from a saved snapshot.
///
/// Convenience wrapper: creates a default [`GuestConfig`] (snapshot
/// already has the guest's cmdline/initrd), registers host functions,
/// and builds a [`MultiUseSandbox`] from the snapshot.
///
/// **Note:** If the snapshot was created with filesystem mounts, the
/// same mounts must be passed here.  The guest kernel's fstab entries
/// are baked into the snapshot; the `mounts` parameter re-registers
/// the host-side functions that serve those mounts.  Passing different
/// or empty mounts when the snapshot expects them will cause guest I/O
/// errors.
///
/// TODO: Add a `GetMountConfig` host function so the kernel can query
/// mount configuration at restore time and reconcile its VFS mount
/// table — unmounting stale entries and mounting new ones — instead of
/// requiring the caller to pass identical mounts.
pub fn restore(
    snapshot: Arc<Snapshot>,
    mounts: Vec<Mount>,
    network: Option<NetworkPolicy>,
    listen_ports: Option<ListenPorts>,
) -> hyperlight_host::Result<(MultiUseSandbox, GuestConfig)> {
    if mounts.is_empty() {
        debug!("restore: no mounts provided — if the snapshot was saved with mounts, hostfs operations will fail");
    }
    let config = GuestConfig {
        cmdline: String::new(),
        scratch_size: DEFAULT_SCRATCH_MB * 1024 * 1024,
        initrd_base: 0,
        initrd_size: 0,
        mounts,
        network,
        listen_ports,
        output: Arc::new(Mutex::new(String::new())),
    };
    let mut hf = HostFunctions::default();
    config.register(&mut hf)?;
    let sandbox = MultiUseSandbox::from_snapshot(snapshot, hf, None)?;
    Ok((sandbox, config))
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
            mounts: Vec::new(),
            network: None,
            listen_ports: None,
            output: Arc::new(Mutex::new(String::new())),
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
        buf.extend(std::iter::repeat_n(0u8, name_pad));
        buf.extend_from_slice(data);
        let data_pad = (4 - (filesize % 4)) % 4;
        buf.extend(std::iter::repeat_n(0u8, data_pad));
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

    // -- Mount / fstab tests --

    #[test]
    fn mount_rw_host_first() {
        let m = Mount::rw("/host/dir", "/guest/path");
        assert_eq!(m.host_path, PathBuf::from("/host/dir"));
        assert_eq!(m.guest_path, "/guest/path");
        assert!(!m.readonly);
    }

    #[test]
    fn mount_ro_host_first() {
        let m = Mount::ro("/host/dir", "/guest/path");
        assert_eq!(m.host_path, PathBuf::from("/host/dir"));
        assert_eq!(m.guest_path, "/guest/path");
        assert!(m.readonly);
    }

    #[test]
    fn fstab_cmdline_single_rw_mount() {
        let mounts = [Mount::rw("/tmp/share", "/mnt/host")];
        let mut cmdline = "unikraft-hyperlight /entry".to_string();
        if !mounts.is_empty() {
            cmdline.push_str(" vfs.fstab=[");
            for (i, m) in mounts.iter().enumerate() {
                if i > 0 { cmdline.push(' '); }
                let flags = if m.readonly { "0x1" } else { "0x0" };
                std::fmt::Write::write_fmt(
                    &mut cmdline,
                    format_args!("{i}:{}:hostfs:{flags}::mkmp", m.guest_path),
                ).unwrap();
            }
            cmdline.push(']');
        }
        assert_eq!(
            cmdline,
            "unikraft-hyperlight /entry vfs.fstab=[0:/mnt/host:hostfs:0x0::mkmp]",
        );
    }

    /// Reproduce the exact byte output of the C `fb_encode_generic` encoder
    /// for `fs_stat(0, "hello.txt")` and verify Hyperlight can parse it.
    #[test]
    fn flatbuffer_generic_encoder_roundtrip() {
        use hyperlight_common::flatbuffer_wrappers::function_call::FunctionCall;

        // Construct the exact bytes the C fb_encode_generic would produce
        // for hl_hcall_vecbytes("fs_stat", [hlint(0), hlstring("hello.txt")], 2).
        //
        // This replicates the logic in hcall.c fb_encode_generic.
        let c_bytes = build_c_generic_fb(
            "fs_stat",
            2,  // HL_FCT_HOST
            9,  // HL_RT_VECBYTES (= hlsizeprefixedbuffer)
            &[
                CParam::Int(0),
                CParam::Str("hello.txt"),
            ],
        );

        eprintln!("C-encoded ({} bytes):", c_bytes.len());
        for (i, chunk) in c_bytes.chunks(16).enumerate() {
            eprint!("  {:04x}:", i * 16);
            for b in chunk { eprint!(" {:02x}", b); }
            eprintln!();
        }

        // Try to parse the C-style bytes.
        let c_parsed = FunctionCall::try_from(c_bytes.as_slice());
        assert!(c_parsed.is_ok(), "C-encoded FunctionCall should parse: {:?}", c_parsed.err());
        let c_parsed = c_parsed.unwrap();
        assert_eq!(c_parsed.function_name, "fs_stat");
    }

    /// Roundtrip test for fs_write_bytes with empty VecBytes.
    #[test]
    fn flatbuffer_generic_encoder_roundtrip_write() {
        use hyperlight_common::flatbuffer_wrappers::function_call::FunctionCall;

        let c_bytes = build_c_generic_fb(
            "fs_write_bytes",
            2,  // HL_FCT_HOST
            0,  // HL_RT_INT
            &[
                CParam::Int(0),          // mount_idx
                CParam::Str("written.txt"),  // path
                CParam::ULong(0),        // offset
                CParam::Int(0),          // append
                CParam::VecBytes(&[]),   // empty data
            ],
        );

        eprintln!("C-encoded fs_write_bytes ({} bytes):", c_bytes.len());
        for (i, chunk) in c_bytes.chunks(16).enumerate() {
            eprint!("  {:04x}:", i * 16);
            for b in chunk { eprint!(" {:02x}", b); }
            eprintln!();
        }

        let result = FunctionCall::try_from(c_bytes.as_slice());
        match &result {
            Ok(fc) => eprintln!("PARSED: name={}", fc.function_name),
            Err(e) => eprintln!("FAILED: {:?}", e),
        }
        assert!(result.is_ok(), "Should parse: {:?}", result.err());
    }

    // Helper types and function to replicate fb_encode_generic from hcall.c
    enum CParam<'a> {
        Int(i32),
        Str(&'a str),
        ULong(u64),
        VecBytes(&'a [u8]),
    }

    fn align4(x: usize) -> usize { (x + 3) & !3 }
    fn align2(x: usize) -> usize { (x + 1) & !1 }
    /// Smallest value >= x that is congruent to 4 mod 8.
    /// Ensures u64 field at (result + 4) is 8-byte aligned.
    fn align8_off4(x: usize) -> usize { ((x + 3) & !7) | 4 }

    fn ew16(buf: &mut [u8], pos: usize, val: u16) {
        buf[pos] = val as u8;
        buf[pos + 1] = (val >> 8) as u8;
    }
    fn ew32(buf: &mut [u8], pos: usize, val: u32) {
        buf[pos] = val as u8;
        buf[pos + 1] = (val >> 8) as u8;
        buf[pos + 2] = (val >> 16) as u8;
        buf[pos + 3] = (val >> 24) as u8;
    }
    fn ew64(buf: &mut [u8], pos: usize, val: u64) {
        for i in 0..8 {
            buf[pos + i] = (val >> (i * 8)) as u8;
        }
    }

    fn build_c_generic_fb(name: &str, call_type: u8, ret_type: u8, params: &[CParam]) -> Vec<u8> {
        let nlen = name.len();
        let np = params.len();

        const PM_VT_SZ: usize = 8;
        const PM_TBL_SZ: usize = 12;
        const VW_SCALAR_VT_SZ: usize = 6;
        const VW_INT_TBL_SZ: usize = 8;
        const VW_ULONG_TBL_SZ: usize = 12;
        const VW_REF_TBL_SZ: usize = 8;

        struct PLay { pvt: usize, ptbl: usize, vvt: usize, vtbl: usize, vdata: usize, vvtsz: usize, vtblsz: usize }

        let mut pos: usize = 36;
        let pvec = if np > 0 { let v = align4(pos); pos = v + 4 + np * 4; v } else { 0 };

        let mut pl: Vec<PLay> = Vec::new();
        for param in params.iter().take(np) {
            let pvt = align2(pos);
            let ptbl = align4(pvt + PM_VT_SZ);
            let (vvtsz, vtblsz) = match param {
                CParam::Int(_) => (VW_SCALAR_VT_SZ, VW_INT_TBL_SZ),
                CParam::ULong(_) => (VW_SCALAR_VT_SZ, VW_ULONG_TBL_SZ),
                CParam::Str(_) | CParam::VecBytes(_) => (VW_SCALAR_VT_SZ, VW_REF_TBL_SZ),
            };
            let vvt = align2(ptbl + PM_TBL_SZ);
            let vtbl = match param {
                CParam::ULong(_) => align8_off4(vvt + vvtsz),
                _ => align4(vvt + vvtsz),
            };
            pos = vtbl + vtblsz;
            pl.push(PLay { pvt, ptbl, vvt, vtbl, vdata: 0, vvtsz, vtblsz });
        }

        // Variable-length data
        for (i, param) in params.iter().enumerate().take(np) {
            match param {
                CParam::Str(s) => {
                    pl[i].vdata = align4(pos);
                    pos = pl[i].vdata + 4 + align4(s.len() + 1);
                }
                CParam::VecBytes(v) => {
                    pl[i].vdata = align4(pos);
                    let dlen = if v.is_empty() { 1 } else { v.len() };
                    pos = pl[i].vdata + 4 + align4(dlen);
                }
                _ => {}
            }
        }

        let fnpos = align4(pos);
        pos = fnpos + 4 + align4(nlen + 1);
        let total = pos;

        let mut buf = vec![0u8; total];

        // Size prefix
        ew32(&mut buf, 0, (total - 4) as u32);
        // Root offset
        ew32(&mut buf, 4, 16);

        // Root vtable at 8
        ew16(&mut buf, 8, 12);
        ew16(&mut buf, 10, 16);
        ew16(&mut buf, 12, 4); // VT+4: function_name
        ew16(&mut buf, 14, if np > 0 { 8 } else { 0 }); // VT+6: parameters
        ew16(&mut buf, 16, 12); // VT+8: function_call_type
        ew16(&mut buf, 18, 13); // VT+10: expected_return_type

        // Root table at 20
        ew32(&mut buf, 20, 12); // soffset → vtable at 8
        ew32(&mut buf, 24, (fnpos - 24) as u32); // func name uoffset
        if np > 0 {
            ew32(&mut buf, 28, (pvec - 28) as u32); // params vector uoffset
        }
        buf[32] = call_type;
        buf[33] = ret_type;

        // Params vector
        if np > 0 {
            ew32(&mut buf, pvec, np as u32);
            for (i, layout) in pl.iter().enumerate().take(np) {
                let ep = pvec + 4 + i * 4;
                ew32(&mut buf, ep, (layout.ptbl - ep) as u32);
            }
        }

        // Each parameter
        for (param, layout) in params.iter().zip(pl.iter()).take(np) {
            // Parameter vtable
            ew16(&mut buf, layout.pvt, PM_VT_SZ as u16);
            ew16(&mut buf, layout.pvt + 2, PM_TBL_SZ as u16);
            ew16(&mut buf, layout.pvt + 4, 4);
            ew16(&mut buf, layout.pvt + 6, 8);

            // Parameter table
            ew32(&mut buf, layout.ptbl, (layout.ptbl - layout.pvt) as u32);
            let pv_type = match param {
                CParam::Int(_) => 1u8, // HL_PV_HLINT
                CParam::ULong(_) => 4u8, // HL_PV_HLULONG (was incorrectly 5=hlfloat!)
                CParam::Str(_) => 7u8, // HL_PV_HLSTRING
                CParam::VecBytes(_) => 9u8, // HL_PV_HLVECBYTES
            };
            buf[layout.ptbl + 4] = pv_type;
            ew32(&mut buf, layout.ptbl + 8, (layout.vtbl - (layout.ptbl + 8)) as u32);

            // Value vtable
            ew16(&mut buf, layout.vvt, layout.vvtsz as u16);
            ew16(&mut buf, layout.vvt + 2, layout.vtblsz as u16);
            ew16(&mut buf, layout.vvt + 4, 4);

            // Value table
            ew32(&mut buf, layout.vtbl, (layout.vtbl - layout.vvt) as u32);

            match param {
                CParam::Int(v) => {
                    ew32(&mut buf, layout.vtbl + 4, *v as u32);
                }
                CParam::ULong(v) => {
                    ew64(&mut buf, layout.vtbl + 4, *v);
                }
                CParam::Str(s) => {
                    ew32(&mut buf, layout.vtbl + 4, (layout.vdata - (layout.vtbl + 4)) as u32);
                    ew32(&mut buf, layout.vdata, s.len() as u32);
                    buf[layout.vdata + 4..layout.vdata + 4 + s.len()].copy_from_slice(s.as_bytes());
                }
                CParam::VecBytes(v) => {
                    ew32(&mut buf, layout.vtbl + 4, (layout.vdata - (layout.vtbl + 4)) as u32);
                    ew32(&mut buf, layout.vdata, v.len() as u32);
                    if !v.is_empty() {
                        buf[layout.vdata + 4..layout.vdata + 4 + v.len()].copy_from_slice(v);
                    }
                }
            }
        }

        // Function name string
        ew32(&mut buf, fnpos, nlen as u32);
        buf[fnpos + 4..fnpos + 4 + nlen].copy_from_slice(name.as_bytes());

        buf
    }

    /// Roundtrip test for fs_read_bytes(mount_idx=0, path="test.txt", offset=0, len=32768)
    /// which uses u64 parameters.
    #[test]
    fn flatbuffer_generic_encoder_roundtrip_ulong() {
        use hyperlight_common::flatbuffer_wrappers::function_call::FunctionCall;

        let c_bytes = build_c_generic_fb(
            "fs_read_bytes",
            2,  // HL_FCT_HOST
            9,  // HL_RT_VECBYTES
            &[
                CParam::Int(0),
                CParam::Str("test.txt"),
                CParam::ULong(0),
                CParam::ULong(32768),
            ],
        );

        eprintln!("C-encoded fs_read_bytes ({} bytes):", c_bytes.len());
        for (i, chunk) in c_bytes.chunks(16).enumerate() {
            eprint!("  {:04x}:", i * 16);
            for b in chunk { eprint!(" {:02x}", b); }
            eprintln!();
        }

        let c_parsed = FunctionCall::try_from(c_bytes.as_slice());
        assert!(c_parsed.is_ok(), "C-encoded FunctionCall should parse: {:?}", c_parsed.err());
        let c_parsed = c_parsed.unwrap();
        assert_eq!(c_parsed.function_name, "fs_read_bytes");
        assert_eq!(c_parsed.parameters.as_ref().unwrap().len(), 4);
    }

    /// Verify that the C encoder's ULong discriminant matches HL_PV_HLULONG=4
    /// (not 5=hlfloat) and that 8-byte alignment is respected.
    #[test]
    fn c_encoder_ulong_alignment_check() {
        use hyperlight_common::flatbuffer_wrappers::function_call::FunctionCall;

        // First, fix the discriminant and test with type=4 (the REAL HL_PV_HLULONG)
        let c_bytes = build_c_generic_fb(
            "fs_write_bytes",
            2,  // HL_FCT_HOST
            0,  // HL_RT_INT
            &[
                CParam::Int(0),
                CParam::Str("written.txt"),
                CParam::ULong(0),
                CParam::Int(0),
                CParam::VecBytes(&[]),
            ],
        );

        eprintln!("C-encoded fs_write_bytes ({} bytes):", c_bytes.len());
        for (i, chunk) in c_bytes.chunks(16).enumerate() {
            eprint!("  {:04x}:", i * 16);
            for b in chunk { eprint!(" {:02x}", b); }
            eprintln!();
        }

        let result = FunctionCall::try_from(c_bytes.as_slice());
        match &result {
            Ok(fc) => eprintln!("PARSED: name={}", fc.function_name),
            Err(e) => eprintln!("FAILED: {:?}", e),
        }
        assert!(result.is_ok(), "Should parse: {:?}", result.err());
    }

    #[test]
    fn fstab_cmdline_multiple_mixed_mounts() {
        let mounts = [Mount::rw("/a", "/mnt/a"),
            Mount::ro("/b", "/mnt/b")];
        let mut cmdline = "unikraft-hyperlight /entry".to_string();
        if !mounts.is_empty() {
            cmdline.push_str(" vfs.fstab=[");
            for (i, m) in mounts.iter().enumerate() {
                if i > 0 { cmdline.push(' '); }
                let flags = if m.readonly { "0x1" } else { "0x0" };
                std::fmt::Write::write_fmt(
                    &mut cmdline,
                    format_args!("{i}:{}:hostfs:{flags}::mkmp", m.guest_path),
                ).unwrap();
            }
            cmdline.push(']');
        }
        assert_eq!(
            cmdline,
            "unikraft-hyperlight /entry vfs.fstab=[0:/mnt/a:hostfs:0x0::mkmp 1:/mnt/b:hostfs:0x1::mkmp]",
        );
    }
}
