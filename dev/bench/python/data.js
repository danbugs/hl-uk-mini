window.BENCHMARK_DATA = {
  "lastUpdate": 1788456061587,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "python benchmarks": [
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
        "date": 1788389294750,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 284.86,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 297.945,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 428.757,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 15.301,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 9.213,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 149.274,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 13.551,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 7.831,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 151.798,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 0.926,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 0.868,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.263,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.481,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.112,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 44.109,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 21.719,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 12.742,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 253.499,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 18,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 18,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 22,
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
        "date": 1788455730845,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 278.408,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 275.503,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 392.465,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 15.169,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 9.135,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 144.62,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 14.298,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 7.127,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 142.937,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 0.983,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 0.824,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.273,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.496,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.15,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 44.236,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 21.686,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 11.7,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 246.26,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 18,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 18,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 22,
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
        "date": 1788456060973,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 289.642,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 289.416,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 406.551,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 14.647,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 8.704,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 144.378,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 13.295,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 7.091,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 143.469,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 0.918,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 0.851,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.29,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.515,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.124,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 42.759,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 20.695,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 11.392,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 243.803,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 78.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 18,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 18,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 22,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}