# BusyBox hush runtime for Hyperlight/Unikraft.
#
# Builds BusyBox in NOMMU mode so the shell uses vfork() instead of
# fork().  The Unikraft kernel supports vfork+execve (MULTIPROCESS=y)
# but not fork(), so NOMMU mode lets all applets (cat, grep, sed, awk,
# sort, ls, find, etc.) run as separate processes via vfork+execve.
# (NOMMU disables ash; hush is the active shell.)
#
# Limitation: pipes (|) and output redirects from external commands
# (cmd > file) crash because Unikraft's vfork shares the parent's
# stack with the child — a kernel fix (separate child stack) is needed.
# Use awk single-pass or shell builtins to work around.
#
# Config: NOMMU + PREFER_APPLETS + SH_STANDALONE + SH_NOFORK + PIE.

FROM alpine:3.20 AS build

RUN apk add --no-cache gcc musl-dev make perl linux-headers

# ── Build custom BusyBox with NOFORK ──────────────────────────────
ARG BUSYBOX_VERSION=1.36.1
RUN wget -q "https://busybox.net/downloads/busybox-${BUSYBOX_VERSION}.tar.bz2" && \
    tar xjf "busybox-${BUSYBOX_VERSION}.tar.bz2" && \
    mv "busybox-${BUSYBOX_VERSION}" /busybox-src

WORKDIR /busybox-src

# Start from default config, then enable:
#   NOMMU          — force vfork() instead of fork() (unikernel has no fork)
#   PREFER_APPLETS — look up commands as BusyBox applets before PATH
#   SH_STANDALONE  — run applets via internal table (vfork+re-exec)
#   SH_NOFORK      — run NOFORK-flagged applets in-process (no vfork)
#   BUSYBOX_EXEC_PATH — hardcode binary path (no /proc/self/exe needed)
#   PIE            — position-independent executable (elfloader requires it)
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

# ── Compile the C driver ─────────────────────────────────────────
WORKDIR /src

# Preserve directory layout so #include "../hl_fc.h" resolves
COPY hl_fc.h /src/drivers/hl_fc.h
COPY bash/hl_bashdriver.c /src/drivers/bash/hl_bashdriver.c

RUN gcc -O2 -Wall -Wextra -Wno-unused-parameter \
        -static-pie -fPIE \
        -o /hl_bashdriver /src/drivers/bash/hl_bashdriver.c

# ── Set up rootfs with symlinks ──────────────────────────────────
RUN mkdir -p /busybox-root/bin /busybox-root/usr/bin \
             /busybox-root/sbin /busybox-root/tmp /busybox-root/etc && \
    cp /busybox-src/busybox /busybox-root/bin/busybox && \
    cd /busybox-root/bin && \
    for applet in \
        sh ash \
        echo printf cat head tail wc sort uniq tr tee \
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

# Minimal rootfs
FROM scratch

# musl dynamic linker (busybox is dynamically linked)
COPY --from=build /lib/ld-musl-x86_64.so.1 /lib/ld-musl-x86_64.so.1

# BusyBox with all applet symlinks
COPY --from=build /busybox-root/ /

# Hyperlight driver
COPY --from=build /hl_bashdriver /usr/local/bin/hl_bashdriver
