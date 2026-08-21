/// Minimal Hyperlight host for booting a Unikraft unikernel.
///
/// The guest boots, calls host functions for configuration (e.g.
/// GetCmdLine to fetch the command line), then halts (port 108).
use std::env;

use hyperlight_host::{
    GuestBinary, MultiUseSandbox, UninitializedSandbox,
    sandbox::SandboxConfiguration,
};

fn main() -> hyperlight_host::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: hyperlight-unikraft-mini <guest-elf> [-- args...]");
        std::process::exit(1);
    }

    let guest_path = &args[1];

    // Build the command line for the guest.
    // Format: "unikraft-hyperlight [-- user-args...]"
    // If "--" is present, everything after it is forwarded as app args.
    let cmdline = {
        let sep_pos = args.iter().position(|a| a == "--");
        match sep_pos {
            Some(pos) => {
                let mut parts = vec!["unikraft-hyperlight".to_string()];
                parts.push("--".to_string());
                parts.extend_from_slice(&args[pos + 1..]);
                parts.join(" ")
            }
            None => "unikraft-hyperlight".to_string(),
        }
    };

    // Scratch memory is shared between CoW page resolution, paging
    // frame allocator, and host-mapped I/O buffers.  16 MiB gives
    // ~12 MiB for the frame allocator (75% dynamic default) plus
    // headroom for CoW faults and boot overhead.
    let scratch_size: usize = 0x1000000; // 16 MiB
    let mut cfg = SandboxConfiguration::default();
    cfg.set_scratch_size(scratch_size);
    // TODO: The PEB heap is only used for the boot stack (allocated before
    // ukplat_mem_init).  Once the guest allocates the boot stack from scratch
    // instead, heap_size can be dropped to 0 and the PEB heap removed entirely.
    cfg.set_heap_size(0x100000); // 1 MiB (only needed for boot stack pre-paging init)

    let mut usandbox =
        UninitializedSandbox::new(GuestBinary::FilePath(guest_path.clone()), Some(cfg))?;

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

    eprintln!("[host] cmdline: {cmdline}");

    // evolve() creates the VM and runs the guest to completion.
    // During evolve, the guest can call registered host functions
    // (e.g. GetCmdLine).  DebugPrint output goes to stderr.
    let _sandbox: MultiUseSandbox = usandbox.evolve()?;

    Ok(())
}
