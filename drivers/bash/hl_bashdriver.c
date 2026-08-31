/*
 * hl_bashdriver — BusyBox hush runtime driver for Hyperlight.
 *
 * Spawns a persistent BusyBox hush child process during boot via
 * vfork+exec.  The child runs a dispatch loop that reads a signal
 * from a pipe, sources /tmp/hl_dispatch.sh, and writes an ack
 * byte back.
 *
 * BusyBox is built in NOMMU mode so hush uses vfork() instead of
 * fork().  All applets (cat, grep, sed, awk, sort, ls, find, etc.)
 * run as separate processes via vfork+execve.  NOFORK applets
 * (echo, mkdir, touch, etc.) still run in-process for speed.
 *
 * Limitation: pipes (|) and output redirects from external commands
 * (cmd > file) crash the kernel because vfork shares the parent's
 * stack with the child.  A kernel fix (separate child stack) is needed.
 * Use temp files from builtins or awk single-pass for chaining.
 *
 * Flow:
 *   boot (evolve):
 *     main() → create pipes
 *            → write bootstrap script to /tmp/hl_bootstrap.sh
 *            → vfork + exec("/bin/sh", "/tmp/hl_bootstrap.sh")
 *            → read "ready" ack from child
 *            → register dispatch callback, halt
 *
 *   host: call("Exec", "echo hello")
 *     dispatch → hyperlight_dispatch_function (kernel)
 *              → bash_dispatch(fc, fc_len)
 *              → write code to /tmp/hl_dispatch.sh
 *              → write signal to pipe
 *              → read ack byte
 *              → halt
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdint.h>

#include "../hl_fc.h"

/* ── State ─────────────────────────────────────────────────────── */

static hl_dispatch_fn_t *g_callback_slot;
static uint64_t g_dispatch_entry;
static int g_pipe_to_sh;    /* parent writes signal here */
static int g_pipe_from_sh;  /* parent reads ack here */

/* ── Dispatch callback ─────────────────────────────────────────── */

static int bash_dispatch(const uint8_t *fc, size_t fc_len)
{
	/* Extract the code string from the FunctionCall FlatBuffer */
	size_t code_len;
	const char *code = fc_arg0_string(fc, fc_len, &code_len);
	if (!code)
		return -1;

	/* Write code to temp file — the shell sources this */
	FILE *f = fopen("/tmp/hl_dispatch.sh", "w");
	if (!f) {
		fprintf(stderr, "hl_bashdriver: cannot write dispatch file\n");
		fflush(stderr);
		return -1;
	}
	fwrite(code, 1, code_len, f);
	fputc('\n', f);
	fclose(f);

	/* Signal the shell to source the file */
	if (write(g_pipe_to_sh, "g\n", 2) != 2) {
		fprintf(stderr, "hl_bashdriver: pipe write (signal) failed\n");
		fflush(stderr);
		return -1;
	}

	/* Wait for ack — '0' on success, '1' on error.
	 * This read blocks and yields to the cooperative scheduler. */
	char ack = '1';
	if (read(g_pipe_from_sh, &ack, 1) != 1) {
		fprintf(stderr, "hl_bashdriver: ack read failed\n");
		fflush(stderr);
		return -1;
	}
	return ack != '0' ? -1 : 0;
}

/* ── Bootstrap script ─────────────────────────────────────────── */

static int write_bootstrap(int fd_in, int fd_out)
{
	const char *path = "/tmp/hl_bootstrap.sh";
	FILE *f = fopen(path, "w");
	if (!f) {
		fprintf(stderr, "hl_bashdriver: cannot create %s\n", path);
		return -1;
	}

	fprintf(f,
		"#!/bin/sh\n"
		"# Hyperlight bash dispatch loop.\n"
		"# Reads signals from FD %d, sources /tmp/hl_dispatch.sh,\n"
		"# writes ack byte ('0'=success, '1'=error) to FD %d.\n"
		"\n"
		"# Signal ready\n"
		"printf '0' >&%d\n"
		"\n"
		"# Dispatch loop\n"
		"while read -r _signal <&%d; do\n"
		"    if . /tmp/hl_dispatch.sh; then\n"
		"        printf '0' >&%d\n"
		"    else\n"
		"        printf '1' >&%d\n"
		"    fi\n"
		"done\n",
		fd_in, fd_out,
		fd_out,
		fd_in,
		fd_out,
		fd_out);

	fclose(f);
	return 0;
}

/* ── Entry point ───────────────────────────────────────────────── */

int main(int argc, char **argv, char **envp)
{
	(void)argc;
	(void)argv;

	/* Parse kernel addresses from env vars */
	for (char **p = envp; p && *p; p++) {
		if (!strncmp(*p, "HL_DISPATCH_CALLBACK_PTR=", 25))
			g_callback_slot = (hl_dispatch_fn_t *)
				hl_parse_hex(*p + 25);
		else if (!strncmp(*p, "HL_DISPATCH_ENTRY=", 18))
			g_dispatch_entry = hl_parse_hex(*p + 18);
	}

	if (!g_callback_slot || !g_dispatch_entry) {
		fprintf(stderr,
			"hl_bashdriver: missing HL_DISPATCH_CALLBACK_PTR "
			"or HL_DISPATCH_ENTRY\n");
		return 1;
	}

	/* Create pipes: parent→child (signal) and child→parent (ack) */
	int pipe_sig[2];  /* [0]=read, [1]=write */
	int pipe_ack[2];
	if (pipe(pipe_sig) < 0 || pipe(pipe_ack) < 0) {
		fprintf(stderr, "hl_bashdriver: pipe() failed\n");
		return 1;
	}

	/* Write bootstrap script with pipe fd numbers baked in */
	if (write_bootstrap(pipe_sig[0], pipe_ack[1]) < 0)
		return 1;

	/* Spawn persistent shell child.
	 * vfork: parent blocks until child calls exec. */
	pid_t pid = vfork();
	if (pid < 0) {
		fprintf(stderr, "hl_bashdriver: vfork() failed\n");
		return 1;
	}
	if (pid == 0) {
		/* Child — only exec or _exit allowed after vfork */
		execl("/bin/sh", "sh", "/tmp/hl_bootstrap.sh",
		      (char *)NULL);
		_exit(127);
	}

	/* Parent — close the child's pipe ends */
	close(pipe_sig[0]);
	close(pipe_ack[1]);
	g_pipe_to_sh = pipe_sig[1];
	g_pipe_from_sh = pipe_ack[0];

	/* Wait for the child to signal ready */
	char ready;
	if (read(g_pipe_from_sh, &ready, 1) != 1) {
		fprintf(stderr, "hl_bashdriver: shell failed to start\n");
		return 1;
	}

	/* Register dispatch callback */
	*g_callback_slot = bash_dispatch;

	/* Halt the VM — same pattern as other drivers */
	__asm__ volatile(
		"andq $~0xf, %%rsp\n\t"
		"movq %0, %%rax\n\t"
		"movw $108, %%dx\n\t"
		"outl %%eax, %%dx\n\t"
		"cli\n\t"
		"hlt\n\t"
		: : "r"(g_dispatch_entry) : "rax", "rdx", "memory"
	);
	__builtin_unreachable();
}
