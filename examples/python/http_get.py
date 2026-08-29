"""HTTP GET — demonstrates host networking with urllib."""

import urllib.request

url = "http://httpbin.org/get"
print(f"GET {url}")

try:
    with urllib.request.urlopen(url, timeout=10) as resp:
        print(f"Status: {resp.status}")
        print(f"Content-Type: {resp.headers['Content-Type']}")
        body = resp.read().decode()
        # Print first 500 chars to keep output manageable.
        print(body[:500])
except Exception as e:
    print(f"Request failed: {e}")

print("HTTP GET test done.")
