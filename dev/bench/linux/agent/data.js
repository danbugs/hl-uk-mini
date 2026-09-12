window.BENCHMARK_DATA = {
  "lastUpdate": 1789196736994,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "agent benchmarks": [
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
        "date": 1788389859095,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 7694.713,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 7615.568,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 7619.842,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 20.455,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 11.11,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 48.888,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 18.021,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 8.98,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 46.047,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 1.997,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 1.774,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 3.33,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.538,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.128,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 11.076,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 28.314,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 14.409,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 67.393,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 876.4,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 876.4,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 876.4,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 26,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 28,
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
        "date": 1788456543937,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 7839.386,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 7778.426,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 7775.392,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 20.623,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 11.462,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 38.756,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 18.225,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 9.039,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 36.246,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 2.207,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 1.819,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 2.577,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.598,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.081,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 11.038,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 28.557,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 13.87,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 51.306,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 876.4,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 876.4,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 876.4,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 26,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 28,
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
        "date": 1788480226065,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 7737.264,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 7674.931,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 7569.271,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 20.873,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 11.06,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 48.857,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 18.664,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 8.908,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 46.347,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 2.042,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 1.711,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 2.909,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.567,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.152,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 10.94,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 28.328,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 13.725,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 67.091,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 876.4,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 876.4,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 876.4,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 26,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 30,
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
          "id": "707fce8f86b78bbbb183e3a46d35706290913460",
          "message": "Merge pull request #7 from danbugs/hyperlight-0.17\n\nMigrate to hyperlight 0.17 (crates.io) with driver dedup and release/CI tooling",
          "timestamp": "2026-09-10T13:47:22-07:00",
          "tree_id": "24e5029fcb022d636267b060fdda78da62c9295a",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/707fce8f86b78bbbb183e3a46d35706290913460"
        },
        "date": 1789074456443,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 7529.69,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 7514.121,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 7436.051,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 29.941,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 11.285,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 47.055,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 10.078,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 4.472,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 21.042,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 1.173,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 1.14,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.236,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.556,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.152,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 10.209,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 16.67,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 7.127,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 30.472,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 27,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 27,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 29,
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
          "id": "c8f50bf0d99c21de40d573fee1599506b2f2b48c",
          "message": "Merge pull request #8 from danbugs/repo-polish\n\nRepo polish: standard files, license headers, README, and python-shell rename",
          "timestamp": "2026-09-10T17:37:19-07:00",
          "tree_id": "5efb02e9c2735a8a3a5829b10eab4f38f703c4e9",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/c8f50bf0d99c21de40d573fee1599506b2f2b48c"
        },
        "date": 1789088260889,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 7587.224,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 7554.194,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 7558.301,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 30.049,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 11.383,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 49.159,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 9.992,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 4.246,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 21.97,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 1.206,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 1.176,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.238,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.634,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.091,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 11.036,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 16.117,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 6.993,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 30.639,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 875.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 27,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 27,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 29,
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
          "id": "5e5aa8c5b3e0da73c4d52377eb1bb195e54a46db",
          "message": "docs(fs): correct create_sandbox example to the 6-arg signature\n\nThe Rust example passed a bare `false` in place of the network policy and\nwas missing the listen-ports argument; create_sandbox takes\n(initrd, entry, scratch_mb, mounts, network, listen_ports).\n\nSigned-off-by: danbugs <danilochiarlone@gmail.com>",
          "timestamp": "2026-09-11T17:27:09Z",
          "tree_id": "50ed154778cd7231daaa0f8d5e0b0f8361ae1f18",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/5e5aa8c5b3e0da73c4d52377eb1bb195e54a46db"
        },
        "date": 1789148182774,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 7802.175,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 7825.403,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 7780.656,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 31.672,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 12.078,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 51.34,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 11.141,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 4.466,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 23.295,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 1.247,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 1.219,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.309,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.582,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.111,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 11.165,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 17.313,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 7.209,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 32.029,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 875.2,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 875.2,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 875.2,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 29,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 27,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 29,
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
          "id": "55d3e7b363dc3a6a42acc293487b042f608e99c4",
          "message": "Merge pull request #9 from danbugs/custom-kernel-and-tier2-tests\n\nCustom kernel support, .NET tier-2 tests, and Unikraft platform fixes",
          "timestamp": "2026-09-11T13:55:04-07:00",
          "tree_id": "ea0e3c5f41fae60982c33768bf20135bad4c4558",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/55d3e7b363dc3a6a42acc293487b042f608e99c4"
        },
        "date": 1789161153139,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 6489.636,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 6449.557,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 6501.687,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 23.706,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 9.224,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 39.151,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 7.779,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 3.961,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 17.278,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 0.986,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 1.029,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.082,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 1.569,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.108,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 8.683,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 12.396,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 5.997,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 25.627,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 875.8,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 875.8,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 875.8,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 30,
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
          "id": "6deccb111e1cbbb485dc79300f70f5f4b3ee875c",
          "message": "Merge pull request #10 from danbugs/sandbox-builder-boot-api\n\nSandboxBuilder + boot() as the library API; CHANGELOG-driven release notes",
          "timestamp": "2026-09-11T23:57:36-07:00",
          "tree_id": "9902f340cc8a8a76ee16123c29e3ec3c981c5ded",
          "url": "https://github.com/danbugs/hl-uk-mini/commit/6deccb111e1cbbb485dc79300f70f5f4b3ee875c"
        },
        "date": 1789196735120,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 6630.711,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 6558.041,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 6593.885,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 22.72,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 9.085,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 36.316,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 8.084,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 3.353,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 18.156,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 1.082,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 1.03,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 1.271,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.039,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.099,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 8.865,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 14.175,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 5.731,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 26.298,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 875.9,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 875.9,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 875.9,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 28,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 30,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}