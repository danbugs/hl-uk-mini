window.BENCHMARK_DATA = {
  "lastUpdate": 1789196478911,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "python benchmarks": [
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
        "date": 1788479971548,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 871.931,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 873.734,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1022.789,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 43.308,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 29.217,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 241.037,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 15.068,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 8.579,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 173.881,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 9.658,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.573,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 11.003,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.783,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.245,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 43.527,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 22.103,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 12.479,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 288.192,
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
            "value": 13,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 11,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 25,
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
        "date": 1789074176484,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 982.138,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 973.014,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1129.068,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 49.067,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 33.277,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 261.709,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 16.938,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 9.028,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 184.749,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 9.73,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.575,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 11.027,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.746,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.27,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 43.675,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 24.49,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 15.986,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 309.678,
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
            "value": 13,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 11,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 26,
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
        "date": 1789088101040,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 1058.183,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 1063.146,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1231.014,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 52.718,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 36.466,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 287.973,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 17.954,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 10.473,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 202.786,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 10.11,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.998,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 11.186,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.894,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.325,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 45.81,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 28.25,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 15.634,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 358.617,
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
            "value": 13,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 11,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 26,
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
        "date": 1789147884669,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 1050.341,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 1068.387,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1244.111,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 53.701,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 35.675,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 286.005,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 17.412,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 10.27,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 195.161,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 9.828,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.822,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 11.048,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.844,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.333,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 45.692,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 31.583,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 15.383,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 443.96,
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
            "value": 13,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 12,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 26,
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
        "date": 1789160891245,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 1024.06,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 1034.564,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1193.992,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 51.277,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 35.327,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 273.928,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 17.006,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 9.856,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 188.005,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 9.707,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.603,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 10.734,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.708,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.317,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 44.912,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 25.855,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 15.898,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 328.403,
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
            "value": 13,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 12,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 26,
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
        "date": 1789196476148,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 1030.993,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 1032.623,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1193.576,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 50.068,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 33.904,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 272.227,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 17.03,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 9.849,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 189.805,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 9.639,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.624,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 10.78,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.703,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.32,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 44.892,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 26.92,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 15.163,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 321.158,
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
            "value": 13,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 11,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 26,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}