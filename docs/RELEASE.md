# Releasing

Releases are driven by the `version` in the root `Cargo.toml` and cut with three
workflows in `.github/workflows/`:

| Workflow | Trigger | What it does |
|---|---|---|
| `release.yml` | manual (`workflow_dispatch`) | Tags the version, creates a GitHub Release with generated notes + the `hluk` binaries, and triggers the GHCR publish. |
| `publish-images.yml` | git tag `v*`, or manual | Pushes each runtime as one package (`:latest` base + `:initrd` cpio), plus busybox, the kernel, and the urunc "hello" image to GHCR. |
| `cargo-publish.yml` | manual | Publishes the `hyperlight-unikraft` crate to crates.io. Dry-run by default. |

## Cutting a release

1. Bump `version` in `Cargo.toml` on `main` (semver, e.g. `0.2.0`), commit, and
   push.
2. Actions → **Create release** → **Run workflow**. It:
   - validates the version and that the tag doesn't exist,
   - builds `hluk` for Linux and Windows (x86_64),
   - creates tag `v<version>` and a GitHub Release with auto-generated notes and
     both binaries attached (`hluk-v<version>-x86_64-*.tar.gz`/`.zip`),
   - triggers `publish-images.yml` for that tag.

Nothing is pushed to crates.io as part of this — that is deliberate.

## What's published

- **GitHub Release**: `hluk` binaries for `x86_64-unknown-linux-gnu` and
  `x86_64-pc-windows-msvc`, plus generated changelog notes.
- **GHCR** (`ghcr.io/<owner>/<repo>/…`):
  - `<runtime>` — one package per runtime (`python`, `node`, `agent`,
    `python-shell`, `bash`, `c`, `go`, `rust`, `dotnet-aot`, `dotnet-jit`,
    `powershell`), with two tags:
    - `:latest` (+ `:v<version>`) — the rootfs filesystem image; build a custom
      guest `FROM <registry>/<runtime>`.
    - `:initrd` (+ `:initrd-v<version>`) — the runnable CPIO; `just pull-rootfs
      <runtime> <registry>` fetches it into `build-elfloader/` to `hluk run` —
      no local build.
  - `busybox` — the shared BusyBox base at `:latest` (bash/agent/python-shell
    build on it),
  - `kernel` — the Unikraft elfloader kernel at `/kernel`,
  - `hello-urunc` — a urunc-runnable OCI image (see `demos/urunc/`).

## crates.io

`hyperlight-unikraft` and `hluk` are the same crate (one `[[bin]]`), so this is a
single `cargo publish`. Run **Publish to crates.io** manually; it's a dry run
unless you set `dry_run = false`.
