# Benchmarks

## Modes

| Mode | Description |
|------|-------------|
| `cold` | Fresh boot (evolve) + dispatch, no snapshot. Measures end-to-end startup. |
| `cold-snap` | Load snapshot from disk + restore + dispatch. Measures cold-start from snapshot. |
| `warm-restore` | Load snapshot once, loop (dispatch + restore). Measures steady-state with memory reset. |
| `warm-stateful` | Load snapshot once, loop dispatch only. Measures pure execution overhead. |
| `parallel` | N concurrent VMs from the same snapshot. Measures throughput under contention. |

## Workloads

- **hello.py** — Minimal (`print("ok")`). Measures pure dispatch overhead.
- **compute.py** — CPU-bound (fibonacci + prime sieve). Measures compute performance.
- **stdlib.py** — Stdlib-heavy (json, re, collections, hashlib). Measures real-world library usage.

## Usage

```sh
just bench python              # all modes, all workloads
just bench python cold-snap    # one mode only
```

## Output

Produces a compact summary table at the end, plus a machine-readable
`bench-results.json` at the repo root for CI consumption (format:
`customSmallerIsBetter` for [github-action-benchmark]).

## Notes

**RSS:** The CLI reports `RssAnon` from `/proc/self/status` (Linux) — anonymous
(private) memory only. This is the density-relevant metric: it scales linearly
with VM count and is the closest analog to Windows' `PrivateMemorySize64`
(used by vmm-benchmarks). Values may still differ from Windows due to how guest
memory is backed (anonymous mmap on Linux vs file-backed mapping on Windows).

**Snapshot size:** Reported as total bytes on disk (recursive `stat` of the
snapshot directory). Same value on all platforms.

## TODO

- [ ] Integrate [pyperformance](https://pyperformance.readthedocs.io/) for
      standardized Python benchmarks alongside custom workloads.
- [ ] Enable CI benchmark workflow (`.github/workflows/benchmarks.yml`) once
      tested on a KVM-capable runner.
- [ ] Add Node.js workloads when the Node.js driver is ready.

[github-action-benchmark]: https://github.com/benchmark-action/github-action-benchmark
