/// Minimal Hyperlight host for booting a Unikraft unikernel.
///
/// The guest boots, calls host functions for configuration (e.g.
/// GetCmdLine to fetch the command line), then halts (port 108).
use std::env;
use std::path::PathBuf;

use hyperlight_host::{
    GuestBinary, MultiUseSandbox, UninitializedSandbox,
    sandbox::SandboxConfiguration,
};

/// GPA where the initrd is mapped via map_file_cow.
/// Past the x86 LAPIC MMIO page (0xFEE0_0000) to avoid collisions
/// with KVM's in-kernel IRQCHIP reservation.
const INITRD_MAP_BASE: u64 = 0xFEF0_0000;

fn main() -> hyperlight_host::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: hyperlight-unikraft-mini <guest-elf> [--initrd <cpio>] [-- args...]");
        std::process::exit(1);
    }

    let guest_path = &args[1];

    // Parse --initrd and -- separator
    let mut initrd_path: Option<PathBuf> = None;
    let mut app_args_start: Option<usize> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--initrd" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--initrd requires a path argument");
                    std::process::exit(1);
                }
                initrd_path = Some(PathBuf::from(&args[i]));
            }
            "--" => {
                app_args_start = Some(i + 1);
                break;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Build the command line for the guest.
    //
    // elfloader with CUSTOMAPPNAME expects argv[1] to be the
    // executable path on the VFS.  uk_libparam_parse strips
    // everything before "--" as kernel parameters, so we pass
    // the executable and its args as a flat list WITHOUT "--".
    //
    // Host CLI:  ... -- /usr/bin/python3 -c "print('hello')"
    // Guest cmd: unikraft-hyperlight /usr/bin/python3 -c print('hello')
    let cmdline = {
        match app_args_start {
            Some(pos) if pos < args.len() => {
                let mut parts = vec!["unikraft-hyperlight".to_string()];
                parts.extend_from_slice(&args[pos..]);
                parts.join(" ")
            }
            _ => "unikraft-hyperlight".to_string(),
        }
    };

    // Scratch memory is shared between CoW page resolution, paging
    // frame allocator, and host-mapped I/O buffers.  The frame
    // allocator gets 75% of this budget; the rest is for CoW faults
    // and boot overhead.  Python's rootfs cpio extracts ~68 MiB into
    // ramfs via demand paging, so we need plenty of scratch.
    let scratch_size: usize = 0x10000000; // 256 MiB
    let mut cfg = SandboxConfiguration::default();
    cfg.set_scratch_size(scratch_size);
    // TODO: The PEB heap is only used for the boot stack (allocated before
    // ukplat_mem_init).  Once the guest allocates the boot stack from scratch
    // instead, heap_size can be dropped to 0 and the PEB heap removed entirely.
    cfg.set_heap_size(0x100000); // 1 MiB (only needed for boot stack pre-paging init)

    let mut usandbox =
        UninitializedSandbox::new(GuestBinary::FilePath(guest_path.clone()), Some(cfg))?;

    // Map initrd via zero-copy CoW if provided.
    // The host decides the GPA and tells the guest via GetInitrdBase/GetInitrdSize.
    let (initrd_base, initrd_size): (u64, u64) = if let Some(ref path) = initrd_path {
        let size = usandbox.map_file_cow(path, INITRD_MAP_BASE)?;
        eprintln!("[host] initrd: {} ({} bytes) mapped at GPA {:#x}",
                  path.display(), size, INITRD_MAP_BASE);
        (INITRD_MAP_BASE, size)
    } else {
        (0, 0)
    };

    // Register host functions that the guest can call during boot.

    // GetCmdLine() -> String: returns the command line for Unikraft.
    let cmdline_clone = cmdline.clone();
    usandbox.register("GetCmdLine", move || -> hyperlight_host::Result<String> {
        Ok(cmdline_clone.clone())
    })?;

    // GetPagingBudget() -> u64: tells the guest how many bytes of
    // scratch to give the paging frame allocator.  Give 75% of
    // scratch — the remaining 25% is for CoW faults + boot overhead.
    let paging_budget = (scratch_size as u64) * 3 / 4;
    usandbox.register("GetPagingBudget", move || -> hyperlight_host::Result<u64> {
        Ok(paging_budget)
    })?;

    // GetInitrdBase() -> u64: GPA where the initrd was mapped (0 = none).
    usandbox.register("GetInitrdBase", move || -> hyperlight_host::Result<u64> {
        Ok(initrd_base)
    })?;

    // GetInitrdSize() -> u64: size of the mapped initrd (0 = none).
    usandbox.register("GetInitrdSize", move || -> hyperlight_host::Result<u64> {
        Ok(initrd_size)
    })?;

    eprintln!("[host] cmdline: {cmdline}");

    // evolve() creates the VM and runs the guest to completion.
    // During evolve, the guest can call registered host functions
    // (e.g. GetCmdLine).  DebugPrint output goes to stderr.
    let _sandbox: MultiUseSandbox = usandbox.evolve()?;

    Ok(())
}
