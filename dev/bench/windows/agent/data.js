window.BENCHMARK_DATA = {
  "lastUpdate": 1788480583959,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "agent benchmarks": [
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
        "date": 1788480581153,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 11002.436,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 11003.118,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 11111.217,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 71.704,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 53.88,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 130.146,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 18.266,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 8.649,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 33.44,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 41.971,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 41.589,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 42.221,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 2.228,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.217,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 8.853,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 27.759,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 12.758,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 57.4,
            "unit": "ms"
          },
          {
            "name": "snapshot-size/compute",
            "value": 876.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/hello",
            "value": 876.3,
            "unit": "MiB"
          },
          {
            "name": "snapshot-size/stdlib",
            "value": 876.3,
            "unit": "MiB"
          },
          {
            "name": "rss/compute",
            "value": 23,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 20,
            "unit": "MB"
          },
          {
            "name": "rss/stdlib",
            "value": 32,
            "unit": "MB"
          }
        ]
      }
    ]
  }
}