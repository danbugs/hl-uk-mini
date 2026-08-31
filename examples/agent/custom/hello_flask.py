"""Custom agent rootfs example — verifies pip-installed packages work."""
import flask
import pydantic

print(f"flask={flask.__version__}")
print(f"pydantic={pydantic.__version__}")
print("custom-rootfs-ok")
