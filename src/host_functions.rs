//! Host function registration for Unikraft guests.
//!
//! Provides [`GuestConfig`] which holds the runtime parameters needed
//! to register Hyperlight host functions.  The same config can register
//! on an [`UninitializedSandbox`] (for evolve) or build a
//! [`HostFunctions`] set (for snapshot restore).

use hyperlight_host::{
    HostFunctions, UninitializedSandbox,
    func::Registerable,
};

/// Runtime parameters for the guest's host functions.
///
/// Built once during sandbox setup, then used to register identical
/// host functions for both the evolve and snapshot-restore paths.
pub struct GuestConfig {
    pub cmdline: String,
    pub scratch_size: usize,
    pub initrd_base: u64,
    pub initrd_size: u64,
}

impl GuestConfig {
    fn paging_budget(&self) -> u64 {
        (self.scratch_size as u64) * 3 / 4
    }

    fn exn_stack_top(&self) -> u64 {
        hyperlight_common::layout::SCRATCH_TOP_GVA as u64
            - hyperlight_common::layout::SCRATCH_TOP_EXN_STACK_OFFSET
            + 1
    }

    /// Register host functions on an uninitialized sandbox (evolve path).
    pub fn register(&self, sandbox: &mut UninitializedSandbox) -> hyperlight_host::Result<()> {
        let cmdline = self.cmdline.clone();
        sandbox.register("GetCmdLine", move || -> hyperlight_host::Result<String> {
            Ok(cmdline.clone())
        })?;

        let budget = self.paging_budget();
        sandbox.register("GetPagingBudget", move || -> hyperlight_host::Result<u64> {
            Ok(budget)
        })?;

        let base = self.initrd_base;
        sandbox.register("GetInitrdBase", move || -> hyperlight_host::Result<u64> {
            Ok(base)
        })?;

        let size = self.initrd_size;
        sandbox.register("GetInitrdSize", move || -> hyperlight_host::Result<u64> {
            Ok(size)
        })?;

        let est = self.exn_stack_top();
        sandbox.register("GetExnStackTop", move || -> hyperlight_host::Result<u64> {
            Ok(est)
        })?;

        Ok(())
    }

    /// Build a [`HostFunctions`] set for snapshot restore.
    pub fn host_functions(&self) -> hyperlight_host::Result<HostFunctions> {
        let mut hf = HostFunctions::default();

        let cmdline = self.cmdline.clone();
        hf.register_host_function("GetCmdLine", move || -> hyperlight_host::Result<String> {
            Ok(cmdline.clone())
        })?;

        let budget = self.paging_budget();
        hf.register_host_function(
            "GetPagingBudget",
            move || -> hyperlight_host::Result<u64> { Ok(budget) },
        )?;

        let base = self.initrd_base;
        hf.register_host_function(
            "GetInitrdBase",
            move || -> hyperlight_host::Result<u64> { Ok(base) },
        )?;

        let size = self.initrd_size;
        hf.register_host_function(
            "GetInitrdSize",
            move || -> hyperlight_host::Result<u64> { Ok(size) },
        )?;

        let est = self.exn_stack_top();
        hf.register_host_function(
            "GetExnStackTop",
            move || -> hyperlight_host::Result<u64> { Ok(est) },
        )?;

        Ok(hf)
    }
}
