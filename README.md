# hyperlight-unikraft (hluk)

[Hyperlight](https://github.com/hyperlight-dev/hyperlight) host for
[Unikraft](https://unikraft.org/) unikernels.

## Usage

```bash
# Build a Python rootfs and run a script
just build-rootfs python
just run python examples/python/hello.py

# Or Node.js
just build-rootfs node
just run node examples/node/hello.js

# Snapshots
just snapshot-save python
just snapshot-run .snapshots/python examples/python/hello.py

# See all recipes
just --list
```

## Supported runtimes

| Runtime | Status |
|---------|--------|
| Python  | ✅ CPython 3.12 (glibc) |
| Node.js | ✅ Node 21 (musl/Alpine) |

## Examples and demos

Single-command guest scripts live under `examples/` and run via `just run`:

- `examples/agent-framework/` — Microsoft Agent Framework agents: offline
  llama.cpp inference (`local.py`) and a remote GitHub Models call (`remote.py`).
- `examples/http-server/` — Flask, Express, and .NET Kestrel HTTP servers that
  run inside the guest and answer on `http://127.0.0.1:8080`.
- `examples/autonomous/` — an appliance whose whole workload is baked into the
  initrd and run autonomously (the urunc-style deployment model).

Larger, self-contained demos live under `demos/` (each with its own `Justfile`
and `README.md`, Linux only):

- `demos/supply-chain/` — a typosquat supply-chain attack, contained by the VM.
- `demos/pptx-gen/` — LLM-written `python-pptx` code run in a sandbox to build a
  `.pptx`.
- `demos/urunc/` — deploy a guest through [urunc](https://github.com/urunc-dev/urunc)
  and containerd.
