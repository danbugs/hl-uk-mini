# Agent rootfs for Hyperlight/Unikraft.
#
# Full-featured agent environment: Python 3.12 + BusyBox (hush) +
# data-science packages + SSL/TLS + pip (runtime install via --net).
#
# Three build stages:
#   1. busybox  — BusyBox in NOMMU/PIE mode (Alpine/musl)
#   2. build    — Python driver + pip packages + shared libs (Debian bookworm)
#   3. scratch  — minimal rootfs assembly
#
# Pin to bookworm (glibc 2.36) — newer Debian ships glibc 2.41 which
# uses syscalls Unikraft doesn't support yet.

# ── Stage 1: BusyBox (NOMMU + PIE on musl) ──────────────────────

FROM alpine:3.20 AS busybox

RUN apk add --no-cache gcc musl-dev make perl linux-headers

ARG BUSYBOX_VERSION=1.36.1
RUN wget -q "https://busybox.net/downloads/busybox-${BUSYBOX_VERSION}.tar.bz2" && \
    tar xjf "busybox-${BUSYBOX_VERSION}.tar.bz2" && \
    mv "busybox-${BUSYBOX_VERSION}" /busybox-src

WORKDIR /busybox-src

# NOMMU          — force vfork() instead of fork()
# PREFER_APPLETS — look up commands as BusyBox applets before PATH
# SH_STANDALONE  — run applets via internal table (vfork+re-exec)
# SH_NOFORK      — run NOFORK-flagged applets in-process (no vfork)
# BUSYBOX_EXEC_PATH — hardcode binary path (no /proc/self/exe needed)
# PIE            — position-independent executable (elfloader requires it)
RUN make defconfig && \
    sed -i \
        -e 's/# CONFIG_NOMMU is not set/CONFIG_NOMMU=y/' \
        -e 's/# CONFIG_FEATURE_PREFER_APPLETS is not set/CONFIG_FEATURE_PREFER_APPLETS=y/' \
        -e 's/# CONFIG_FEATURE_SH_STANDALONE is not set/CONFIG_FEATURE_SH_STANDALONE=y/' \
        -e 's/# CONFIG_FEATURE_SH_NOFORK is not set/CONFIG_FEATURE_SH_NOFORK=y/' \
        -e 's|CONFIG_BUSYBOX_EXEC_PATH=.*|CONFIG_BUSYBOX_EXEC_PATH="/bin/busybox"|' \
        -e 's/# CONFIG_PIE is not set/CONFIG_PIE=y/' \
        .config 2>/dev/null; \
    yes '' | make oldconfig

RUN make -j$(nproc) EXTRA_CFLAGS="-fPIE" EXTRA_LDFLAGS="-pie" && \
    make install

# Set up applet symlinks
RUN mkdir -p /busybox-root/bin /busybox-root/usr/bin \
             /busybox-root/sbin /busybox-root/tmp /busybox-root/etc && \
    cp /busybox-src/busybox /busybox-root/bin/busybox && \
    cd /busybox-root/bin && \
    for applet in \
        sh echo printf cat head tail wc sort uniq tr tee \
        ls mkdir rmdir rm cp mv ln chmod chown touch \
        grep egrep sed awk cut paste \
        basename dirname realpath readlink \
        date env sleep true false yes test \
        uname hostname id whoami which \
        dd od hexdump base64 md5sum sha256sum \
        expr seq xargs \
        find stat du df \
    ; do \
        ln -s busybox "$applet"; \
    done

# Minimal /etc for the shell
RUN printf 'root:x:0:0:root:/root:/bin/sh\n' > /busybox-root/etc/passwd && \
    printf 'root:x:0:\n' > /busybox-root/etc/group

# ── Stage 2: Python driver + deps (Debian bookworm / glibc) ─────

FROM python:3.12-slim-bookworm AS build

RUN apt-get update && apt-get install -y --no-install-recommends \
        gcc libc6-dev python3-dev \
        libssl-dev libsqlite3-dev libffi-dev libbz2-dev liblzma-dev \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# ── Install pip packages ────────────────────────────────────────
RUN pip install --no-cache-dir \
        numpy pandas scipy scikit-learn \
        matplotlib seaborn sympy \
        pillow openpyxl python-docx python-pptx \
        pypdf pdfplumber reportlab \
        requests httpx beautifulsoup4 \
        python-dateutil pytz tqdm jinja2 \
        lxml pydantic tabulate chardet pyyaml && \
    # Strip test suites and unnecessary files to reduce rootfs size
    find /usr/local/lib/python3.12 -type d -name tests -exec rm -rf {} + 2>/dev/null; \
    find /usr/local/lib/python3.12 -type d -name test -exec rm -rf {} + 2>/dev/null; \
    find /usr/local/lib/python3.12 -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null; \
    find /usr/local/lib/python3.12 -name '*.pyc' -delete 2>/dev/null; \
    rm -rf /usr/local/lib/python3.12/test 2>/dev/null; \
    rm -rf /usr/local/lib/python3.12/idlelib 2>/dev/null; \
    rm -rf /usr/local/lib/python3.12/tkinter 2>/dev/null; \
    rm -rf /usr/local/lib/python3.12/turtle*.py 2>/dev/null; \
    rm -rf /usr/local/lib/python3.12/ensurepip/_bundled 2>/dev/null; \
    true

# ── Compile agent driver ────────────────────────────────────────
# Preserve directory layout so #include "../hl_fc.h" resolves
COPY drivers/hl_fc.h           /src/drivers/hl_fc.h
COPY drivers/hl_env.h /src/drivers/hl_env.h
COPY drivers/agent/hl_agentdriver.c /src/drivers/agent/hl_agentdriver.c

RUN gcc -O2 -Wall -Wextra -Wno-unused-parameter \
        $(python3-config --cflags --embed) \
        -o /hl_agentdriver /src/drivers/agent/hl_agentdriver.c \
        $(python3-config --ldflags --embed)

# No-op lsb_release stub — pip's distro detection execs this; without
# a valid ELF binary Unikraft's execve crashes (no graceful failure).
COPY drivers/lsb_release_stub.c /src/drivers/lsb_release_stub.c
RUN gcc -O2 -pie -fPIE -o /usr/bin/lsb_release /src/drivers/lsb_release_stub.c

# ── Stage 3: Minimal rootfs ─────────────────────────────────────

FROM scratch

# glibc + dynamic linker
COPY --from=build /lib/x86_64-linux-gnu/libc.so.6     /lib/x86_64-linux-gnu/libc.so.6
COPY --from=build /lib/x86_64-linux-gnu/libm.so.6     /lib/x86_64-linux-gnu/libm.so.6
COPY --from=build /lib64/ld-linux-x86-64.so.2          /lib64/ld-linux-x86-64.so.2
COPY --from=build /usr/lib/x86_64-linux-gnu/libz.so.1  /usr/lib/x86_64-linux-gnu/libz.so.1

# NSS + resolver — needed for DNS resolution (glibc loads these dynamically)
COPY --from=build /usr/lib/x86_64-linux-gnu/libnss_dns.so.2  /usr/lib/x86_64-linux-gnu/libnss_dns.so.2
COPY --from=build /usr/lib/x86_64-linux-gnu/libnss_files.so.2 /usr/lib/x86_64-linux-gnu/libnss_files.so.2
COPY --from=build /usr/lib/x86_64-linux-gnu/libresolv.so.2   /usr/lib/x86_64-linux-gnu/libresolv.so.2

# DNS configuration — glibc defaults to nameserver 127.0.0.1 without this,
# which fails on hosts without a local DNS server (e.g. Windows CI).
COPY drivers/resolv.conf /etc/resolv.conf

# SSL/TLS — unlocks pip install over --net and https from Python
COPY --from=build /lib/x86_64-linux-gnu/libssl.so.3    /lib/x86_64-linux-gnu/libssl.so.3
COPY --from=build /lib/x86_64-linux-gnu/libcrypto.so.3 /lib/x86_64-linux-gnu/libcrypto.so.3
COPY --from=build /etc/ssl/certs                        /etc/ssl/certs
COPY --from=build /etc/ssl/openssl.cnf                   /etc/ssl/openssl.cnf

# C++ runtime (needed by numpy, scipy, sklearn C extensions)
COPY --from=build /usr/lib/x86_64-linux-gnu/libstdc++.so.6  /usr/lib/x86_64-linux-gnu/libstdc++.so.6
COPY --from=build /usr/lib/x86_64-linux-gnu/libgcc_s.so.1   /usr/lib/x86_64-linux-gnu/libgcc_s.so.1

# glibc compat stubs (modern glibc merges these into libc, but
# numpy/scipy .so files still reference them by name)
COPY --from=build /lib/x86_64-linux-gnu/libpthread.so.0     /lib/x86_64-linux-gnu/libpthread.so.0
COPY --from=build /lib/x86_64-linux-gnu/libdl.so.2          /lib/x86_64-linux-gnu/libdl.so.2
COPY --from=build /lib/x86_64-linux-gnu/librt.so.1          /lib/x86_64-linux-gnu/librt.so.1

# Additional shared libs for stdlib modules (_sqlite3, _ctypes, _bz2, _lzma)
COPY --from=build /usr/lib/x86_64-linux-gnu/libsqlite3.so.0 /usr/lib/x86_64-linux-gnu/libsqlite3.so.0
COPY --from=build /usr/lib/x86_64-linux-gnu/libffi.so.8     /usr/lib/x86_64-linux-gnu/libffi.so.8
COPY --from=build /lib/x86_64-linux-gnu/libbz2.so.1.0       /lib/x86_64-linux-gnu/libbz2.so.1.0
COPY --from=build /usr/lib/x86_64-linux-gnu/liblzma.so.5    /usr/lib/x86_64-linux-gnu/liblzma.so.5

# Python interpreter binary (keep at /usr/local/bin so RPATH ../lib resolves)
COPY --from=build /usr/local/bin/python3.12 /usr/local/bin/python3

# Python shared library + soname symlinks
COPY --from=build /usr/local/lib/libpython3.12.so.1.0 /usr/local/lib/libpython3.12.so.1.0
COPY --from=build /usr/local/lib/libpython3.12.so     /usr/local/lib/libpython3.12.so
COPY --from=build /usr/local/lib/libpython3.so        /usr/local/lib/libpython3.so

# Python standard library (includes pip)
COPY --from=build /usr/local/lib/python3.12 /usr/local/lib/python3.12

# ld.so config so the driver can find libpython in /usr/local/lib
COPY --from=build /etc/ld.so.conf   /etc/ld.so.conf
COPY --from=build /etc/ld.so.conf.d /etc/ld.so.conf.d
COPY --from=build /etc/ld.so.cache  /etc/ld.so.cache

# /tmp needed for guest filesystem writes (ramfs, in-memory)
COPY --from=build /tmp /tmp

# Fix sys.executable for subprocess support
COPY drivers/python/sitecustomize.py /usr/local/lib/python3.12/sitecustomize.py

# BusyBox with all applet symlinks + musl linker (BusyBox is musl-linked)
COPY --from=busybox /lib/ld-musl-x86_64.so.1 /lib/ld-musl-x86_64.so.1
COPY --from=busybox /busybox-root/ /

# Agent driver + lsb_release stub (must be in /bin — guest PATH is /bin)
COPY --from=build /hl_agentdriver /usr/local/bin/hl_agentdriver
COPY --from=build /usr/bin/lsb_release /bin/lsb_release
