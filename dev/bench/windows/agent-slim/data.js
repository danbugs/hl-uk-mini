window.BENCHMARK_DATA = {
  "lastUpdate": 1788480053215,
  "repoUrl": "https://github.com/danbugs/hl-uk-mini",
  "entries": {
    "agent-slim benchmarks": [
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
        "date": 1788480051044,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "cold/compute",
            "value": 1682.726,
            "unit": "ms"
          },
          {
            "name": "cold/hello",
            "value": 1558.261,
            "unit": "ms"
          },
          {
            "name": "cold/stdlib",
            "value": 1597.536,
            "unit": "ms"
          },
          {
            "name": "cold-snap/compute",
            "value": 56.176,
            "unit": "ms"
          },
          {
            "name": "cold-snap/hello",
            "value": 34.435,
            "unit": "ms"
          },
          {
            "name": "cold-snap/stdlib",
            "value": 94.372,
            "unit": "ms"
          },
          {
            "name": "warm-restore/compute",
            "value": 21.138,
            "unit": "ms"
          },
          {
            "name": "warm-restore/hello",
            "value": 10.106,
            "unit": "ms"
          },
          {
            "name": "warm-restore/stdlib",
            "value": 37.8,
            "unit": "ms"
          },
          {
            "name": "restore-cost/compute",
            "value": 10.012,
            "unit": "ms"
          },
          {
            "name": "restore-cost/hello",
            "value": 9.758,
            "unit": "ms"
          },
          {
            "name": "restore-cost/stdlib",
            "value": 10.177,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/compute",
            "value": 3.013,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/hello",
            "value": 0.379,
            "unit": "ms"
          },
          {
            "name": "warm-stateful/stdlib",
            "value": 10.833,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/compute",
            "value": 33.956,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/hello",
            "value": 16.218,
            "unit": "ms"
          },
          {
            "name": "parallel-exec/stdlib",
            "value": 63.055,
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
            "value": 14,
            "unit": "MB"
          },
          {
            "name": "rss/hello",
            "value": 12,
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