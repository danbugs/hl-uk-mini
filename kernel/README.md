# Kernel

The embedded kernel binary (`elfloader_hyperlight-x86_64`) is a Unikraft
app-elfloader built against the `plat-hyperlight-cleanup` branch.

Users don't need to build this — it's embedded in the `hluk` binary via
`include_bytes!`. Only the rootfs (built with `just build-rootfs`) needs
to be produced by users.

## Configuration

See `../defconfig-elfloader` for the full kconfig used to build this kernel.

## Rebuilding from source

If you need to rebuild the kernel (e.g. to pick up Unikraft changes):

```bash
# 1. Clone and checkout the plat-hyperlight-cleanup branch
git clone https://github.com/unikraft/unikraft.git
cd unikraft
git checkout plat-hyperlight-cleanup

# 2. Configure
mkdir -p build
cp /path/to/hyperlight-unikraft/defconfig-elfloader build/.config
make A=/path/to/app-elfloader O=build olddefconfig

# 3. Build
make A=/path/to/app-elfloader O=build -j$(nproc)

# 4. Copy the binary
cp build/elfloader_hyperlight-x86_64 /path/to/hyperlight-unikraft/kernel/
```

<!-- TODO: upstream kernel changes to kraft so this can be built with
     `kraft build --plat hyperlight --arch x86_64` without manual patching. -->
