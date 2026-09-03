window.BENCHMARK_DATA = {
  "lastUpdate": 1788479773361,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "agent-slim benchmarks": [
      {
        "commit": {
          "author": {
            "email": "danilochiarlone@gmail.com",
            "name": "Dan Chiarlone",
            "username": "danbugs"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65916668b7553f058722185033a7b100aabf301f",
          "message": "ci: add Linux CI workflow (#2)\n\n* ci: add Linux CI workflow\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* style: cargo fmt\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* fix: mkdir .snapshots parent dir in bench recipe\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: remove kernel-verify from gate until binary is rebuilt\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* feat: dockerize kernel build for reproducibility\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* chore: rebuild kernel binary with Docker (ubuntu:24.04 gcc-13)\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: simplify kernel-verify (Docker handles deps), re-add to gate\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* fix: add test_timeout to known_failures, fail on crash/error\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* fix: conformance recipe exits non-zero on failures/crashes\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: build all runtimes, bench agent, readable summaries\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: update actions to node24, fix bench summary extraction\n\n- actions/checkout v4.2.2 -> v7.0.1 (node24)\n- Swatinem/rust-cache v2.7.8 -> v2.9.2 (node24)\n- fix bench summary: use awk to extract table between box footer and JSON line\n  (sed pattern used unicode box-drawing chars that didn't match ASCII output)\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* refactor: deduplicate scratch-size lookup in justfile\n\nExtract the 4 duplicated 12-line case blocks into a single\n_scratch-mb private recipe. Adding a new runtime now means\nupdating one place instead of four.\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: use just build-all-rootfs instead of manual loop\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: cache rootfs CPIOs across runs\n\nSkip Docker rootfs builds when Dockerfiles haven't changed.\nEach job caches its CPIOs keyed on driver file hashes:\n- test: all rootfs CPIOs (+ .NET SDK, Go, cpio skipped on hit)\n- conformance: python + conformance CPIOs\n- bench: per-runtime CPIO\n\nAlso removes redundant 'cargo build --release' step from bench\n(the just bench recipe already calls 'just build').\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n---------\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>",
          "timestamp": "2026-09-02T15:45:53-07:00",
          "tree_id": "bfb6d61e4b83ff8d9b1755c1a729e9ee1ff56041",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/65916668b7553f058722185033a7b100aabf301f"
        },
        "date": 1788389382487,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 533.998,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 535.125,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 548.225,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 14.2,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 8.404,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 29.382,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 12.594,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 6.492,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 28.79,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 0.914,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 0.825,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.102,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.335,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.137,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 11.347,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 20.334,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 11.039,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 43.183,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 19,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 17,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 19,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "danilochiarlone@gmail.com",
            "name": "danbugs",
            "username": "danbugs"
          },
          "committer": {
            "email": "danilochiarlone@gmail.com",
            "name": "danbugs",
            "username": "danbugs"
          },
          "distinct": true,
          "id": "766bea65ee61108d87caeb8ddeac082b8e3e6735",
          "message": "ci: add Windows build and unit-test job\n\nCompiles for Windows and runs the host-side unit tests (socket layer over\nWinSock, errno translation).  Guest tests need rootfs images built on\nLinux and are not run on Windows CI yet.\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>",
          "timestamp": "2026-09-03T17:08:20Z",
          "tree_id": "022c836ccdfb8e8484170181b97163eacee4f707",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/766bea65ee61108d87caeb8ddeac082b8e3e6735"
        },
        "date": 1788455687334,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 579.499,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 569.063,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 575.793,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 16.206,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 9.468,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 29.79,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 14.899,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 7.022,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 29.57,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 1.083,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 0.818,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.189,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.371,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.111,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 10.133,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 20.153,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 11.231,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 42.459,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 19,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 17,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 19,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "danilochiarlone@gmail.com",
            "name": "Dan Chiarlone",
            "username": "danbugs"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "65916668b7553f058722185033a7b100aabf301f",
          "message": "ci: add Linux CI workflow (#2)\n\n* ci: add Linux CI workflow\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* style: cargo fmt\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* fix: mkdir .snapshots parent dir in bench recipe\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: remove kernel-verify from gate until binary is rebuilt\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* feat: dockerize kernel build for reproducibility\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* chore: rebuild kernel binary with Docker (ubuntu:24.04 gcc-13)\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: simplify kernel-verify (Docker handles deps), re-add to gate\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* fix: add test_timeout to known_failures, fail on crash/error\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* fix: conformance recipe exits non-zero on failures/crashes\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: build all runtimes, bench agent, readable summaries\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: update actions to node24, fix bench summary extraction\n\n- actions/checkout v4.2.2 -> v7.0.1 (node24)\n- Swatinem/rust-cache v2.7.8 -> v2.9.2 (node24)\n- fix bench summary: use awk to extract table between box footer and JSON line\n  (sed pattern used unicode box-drawing chars that didn't match ASCII output)\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* refactor: deduplicate scratch-size lookup in justfile\n\nExtract the 4 duplicated 12-line case blocks into a single\n_scratch-mb private recipe. Adding a new runtime now means\nupdating one place instead of four.\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: use just build-all-rootfs instead of manual loop\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n* ci: cache rootfs CPIOs across runs\n\nSkip Docker rootfs builds when Dockerfiles haven't changed.\nEach job caches its CPIOs keyed on driver file hashes:\n- test: all rootfs CPIOs (+ .NET SDK, Go, cpio skipped on hit)\n- conformance: python + conformance CPIOs\n- bench: per-runtime CPIO\n\nAlso removes redundant 'cargo build --release' step from bench\n(the just bench recipe already calls 'just build').\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>\n\n---------\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>",
          "timestamp": "2026-09-02T15:45:53-07:00",
          "tree_id": "bfb6d61e4b83ff8d9b1755c1a729e9ee1ff56041",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/65916668b7553f058722185033a7b100aabf301f"
        },
        "date": 1788456076658,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 535.968,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 538.188,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 549.054,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 13.903,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 7.719,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 28.41,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 12.436,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 6.089,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 27.69,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 0.885,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 0.755,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.049,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.171,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.107,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 9.738,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 19.237,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 9.571,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 40.072,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 19,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 17,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 19,
            "unit": "MB"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "danilochiarlone@gmail.com",
            "name": "danbugs",
            "username": "danbugs"
          },
          "committer": {
            "email": "danilochiarlone@gmail.com",
            "name": "Dan Chiarlone",
            "username": "danbugs"
          },
          "distinct": true,
          "id": "f9d0fa22492afa5c1fe89b5e7c078e94fe3f337c",
          "message": "bench: report resident memory on Windows\n\nThe RSS line only existed on Linux (RssAnon from /proc), so the Windows\nbench summary lacked the density metric.  Windows now reports the\nworking set from GetProcessMemoryInfo, the counter family vmm-benchmarks\nuses there: guest memory is a section mapping, which the private-commit\ncounters do not attribute to the process.  Values are comparable within\nan OS, not across them.\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>",
          "timestamp": "2026-09-03T16:47:57-07:00",
          "tree_id": "41a3e81cd10063e2f8bcb277811c8986702a359f",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/f9d0fa22492afa5c1fe89b5e7c078e94fe3f337c"
        },
        "date": 1788479771930,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 651.948,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 650.406,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 664.087,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 18.368,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 10.361,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 35.546,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 16.424,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 8.462,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 33.509,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 1.408,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 1.245,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.496,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.556,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.122,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 10.94,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 24.571,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 12.297,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 46.834,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 111.6,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 19,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 17,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 19,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}