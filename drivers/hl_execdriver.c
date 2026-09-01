/*
 * hl_execdriver — Exec-based runtime driver for compiled languages.
 *
 * On dispatch, receives a binary path (e.g. "/mnt/bin/hello") and runs
 * it via vfork+execve.  Exit detection uses a pipe: the child inherits
 * the write end, and when it exits the write end closes, producing
 * EOF on the parent's read.  Used by C, Rust, Go, and .NET Native AOT
 * runtimes where the user mounts a host directory containing the
 * compiled binary into the guest.
 *
 * Flow:
 *   boot (evolve):
 *     main() → register dispatch callback → halt
 *
 *   host: call("Exec", "/bin/hello")
 *     dispatch → exec_dispatch(fc, fc_len)
 *              → pipe() + vfork + execl(path)
 *              → read(pipe) blocks until child exits (EOF)
 *              → return → halt
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <stdint.h>

#include "hl_fc.h"
#include "hl_env.h"

/* ── State ─────────────────────────────────────────────────────── */

static hl_dispatch_fn_t *g_callback_slot;
static uint64_t g_dispatch_entry;

/* ── Dispatch callback ─────────────────────────────────────────── */

static int exec_dispatch(const uint8_t *fc, size_t fc_len)
{
	/* Refresh glibc environ so the exec'd child inherits host vars. */
	hl_env_refresh(NULL, NULL);

	/* Extract the command path from the FunctionCall FlatBuffer */
	size_t cmd_len;
	const char *cmd = fc_arg0_string(fc, fc_len, &cmd_len);
	if (!cmd)
		return -1;

	/* Null-terminate */
	char path[4096];
	if (cmd_len >= sizeof(path))
		return -1;
	memcpy(path, cmd, cmd_len);
	path[cmd_len] = '\0';

	/* Create a pipe for exit detection.  The child inherits the
	 * write end via exec (no CLOEXEC).  When the child exits,
	 * the kernel closes its fds, dropping the last writer and
	 * producing EOF on the parent's read end. */
	int fds[2];
	if (pipe(fds) < 0) {
		fprintf(stderr, "hl_execdriver: pipe() failed\n");
		fflush(stderr);
		return -1;
	}

	pid_t pid = vfork();
	if (pid < 0) {
		close(fds[0]);
		close(fds[1]);
		fprintf(stderr, "hl_execdriver: vfork() failed\n");
		fflush(stderr);
		return -1;
	}
	if (pid == 0) {
		/* Child — only exec or _exit allowed after vfork.
		 * Both pipe ends are inherited; exec keeps them. */
		execl(path, path, (char *)NULL);
		_exit(127);
	}

	/* Parent — close write end so the only writer is the child */
	close(fds[1]);

	/* Block until the child exits (EOF on the read end) */
	char buf;
	while (read(fds[0], &buf, 1) > 0)
		;
	close(fds[0]);

	return 0;
}

/* ── Entry point ───────────────────────────────────────────────── */

int main(int argc, char **argv, char **envp)
{
	(void)argc;
	(void)argv;

	/* Parse kernel addresses from env vars */
	hl_env_init(envp, &g_callback_slot, &g_dispatch_entry);
	hl_env_clean_reserved();

	if (!g_callback_slot || !g_dispatch_entry) {
		fprintf(stderr,
			"hl_execdriver: missing HL_DISPATCH_CALLBACK_PTR "
			"or HL_DISPATCH_ENTRY\n");
		return 1;
	}

	/* Register dispatch callback */
	*g_callback_slot = exec_dispatch;

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
