/*
 * hl_pwshdriver — PowerShell runtime driver for Hyperlight.
 *
 * On dispatch, receives PowerShell code, writes it to a temp file,
 * and runs it via vfork+execve of pwsh.  Each dispatch is a fresh
 * pwsh invocation (no persistent subprocess).
 *
 * Exit detection uses a pipe (same as the exec driver): the child
 * inherits the write end, and EOF signals the parent when it exits.
 *
 * Flow:
 *   boot (evolve):
 *     main() → register dispatch callback → halt
 *
 *   host: call("Exec", "Write-Host 'hello'")
 *     dispatch → pwsh_dispatch(fc, fc_len)
 *              → write code to /tmp/hl_dispatch.ps1
 *              → pipe() + vfork + execl("pwsh", "-File", ...)
 *              → read(pipe) blocks until child exits
 *              → return → halt
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

/* ── Dispatch callback ─────────────────────────────────────────── */

static int pwsh_dispatch(const uint8_t *fc, size_t fc_len)
{
	/* Extract the PS1 code from the FunctionCall FlatBuffer */
	size_t code_len;
	const char *code = fc_arg0_string(fc, fc_len, &code_len);
	if (!code)
		return -1;

	/* Write code to temp file */
	FILE *f = fopen("/tmp/hl_dispatch.ps1", "w");
	if (!f) {
		fprintf(stderr, "hl_pwshdriver: cannot write dispatch file\n");
		fflush(stderr);
		return -1;
	}
	fwrite(code, 1, code_len, f);
	fputc('\n', f);
	fclose(f);

	/* Pipe for exit detection */
	int fds[2];
	if (pipe(fds) < 0) {
		fprintf(stderr, "hl_pwshdriver: pipe() failed\n");
		fflush(stderr);
		return -1;
	}

	pid_t pid = vfork();
	if (pid < 0) {
		close(fds[0]);
		close(fds[1]);
		fprintf(stderr, "hl_pwshdriver: vfork() failed\n");
		fflush(stderr);
		return -1;
	}
	if (pid == 0) {
		/* Child — only exec or _exit allowed after vfork */
		execl("/opt/microsoft/powershell/7/pwsh", "pwsh",
		      "-NoProfile", "-NonInteractive",
		      "-File", "/tmp/hl_dispatch.ps1",
		      (char *)NULL);
		_exit(127);
	}

	/* Parent — close write end, read until EOF */
	close(fds[1]);
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
	for (char **p = envp; p && *p; p++) {
		if (!strncmp(*p, "HL_DISPATCH_CALLBACK_PTR=", 25))
			g_callback_slot = (hl_dispatch_fn_t *)
				hl_parse_hex(*p + 25);
		else if (!strncmp(*p, "HL_DISPATCH_ENTRY=", 18))
			g_dispatch_entry = hl_parse_hex(*p + 18);
	}

	if (!g_callback_slot || !g_dispatch_entry) {
		fprintf(stderr,
			"hl_pwshdriver: missing HL_DISPATCH_CALLBACK_PTR "
			"or HL_DISPATCH_ENTRY\n");
		return 1;
	}

	/* .NET requires ICU for globalization; skip it in the unikernel */
	putenv("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=true");

	/* Register dispatch callback */
	*g_callback_slot = pwsh_dispatch;

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
