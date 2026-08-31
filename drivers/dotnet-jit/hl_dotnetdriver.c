/*
 * hl_dotnetdriver — .NET JIT runtime driver for Hyperlight.
 *
 * Spawns a persistent .NET dispatch server during boot via
 * vfork+exec.  The server uses the Roslyn compilation API to compile
 * and execute raw C# source code in-guest — same dispatch pattern
 * as the Python and Node drivers.
 *
 * Flow:
 *   boot (evolve):
 *     main() → set .NET env vars (W^X, GC limits, etc.)
 *            → create pipes
 *            → vfork + exec("/app/HlDotnetDispatch", fd_in, fd_out)
 *            → read "ready" ack from child (blocks, scheduler
 *              switches to child, CoreCLR + Roslyn warm up,
 *              child signals)
 *            → register dispatch callback, halt
 *
 *   host: call("Exec", <C# source code>)
 *     dispatch → hyperlight_dispatch_function (kernel)
 *              → dotnet_dispatch(fc, fc_len)
 *              → write [len:u64][payload] to pipe
 *              → read ack byte (blocks, scheduler switches to
 *                child, Roslyn compile + execute, writes ack)
 *              → halt
 *
 * Roslyn is warmed up during boot so snapshots capture the
 * initialized state — dispatches after restore are fast.
 *
 * The .NET process stays alive across dispatches.  Each dispatch
 * compiles and runs independently (no shared state between calls).
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
static int g_pipe_to_dotnet;    /* parent writes payload here */
static int g_pipe_from_dotnet;  /* parent reads ack here */

/* ── Dispatch callback ─────────────────────────────────────────── */

static int dotnet_dispatch(const uint8_t *fc, size_t fc_len)
{
	/* Extract the C# source code from the FunctionCall FlatBuffer. */
	size_t code_len;
	const char *code = fc_arg0_string(fc, fc_len, &code_len);
	if (!code)
		return -1;

	/* Send length (8 bytes LE) + payload to the child */
	uint64_t len64 = (uint64_t)code_len;
	if (write(g_pipe_to_dotnet, &len64, 8) != 8)
		return -1;
	const char *p = code;
	size_t remaining = code_len;
	while (remaining > 0) {
		ssize_t n = write(g_pipe_to_dotnet, p, remaining);
		if (n <= 0)
			return -1;
		p += n;
		remaining -= n;
	}

	/* Wait for ack — child sends 0x00 on success, 0x01 on error.
	 * This read blocks and yields to the cooperative scheduler,
	 * which switches to the child .NET thread. */
	char ack = 1;
	if (read(g_pipe_from_dotnet, &ack, 1) != 1)
		return -1;
	return ack != 0 ? -1 : 0;
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
			"hl_dotnetdriver: missing HL_DISPATCH_CALLBACK_PTR "
			"or HL_DISPATCH_ENTRY\n");
		return 1;
	}

	/*
	 * Set .NET runtime environment variables before spawning the
	 * child.  These are critical for CoreCLR in a unikernel:
	 *
	 *   W^X:         JIT needs write+execute on the same pages.
	 *   Diagnostics: Named pipes / IPC that doesn't work here.
	 *   GC heap:     768 MB — Roslyn compilation needs significant
	 *                heap for metadata references and compilation
	 *                data structures.
	 *   Server GC:   Disable (single-CPU environment).
	 *   Globalization: Skip ICU (not in rootfs).
	 *   Stack size:  256 KB per thread — Roslyn's deep call stacks
	 *                need more than the default 64 KB.
	 *   ThreadPool:  Limit thread count for cooperative scheduler.
	 */
	setenv("DOTNET_EnableWriteXorExecute", "0", 1);
	setenv("COMPlus_EnableDiagnostics", "0", 1);
	setenv("DOTNET_GCHeapHardLimit", "0x30000000", 1);
	setenv("DOTNET_gcServer", "0", 1);
	setenv("DOTNET_SYSTEM_GLOBALIZATION_INVARIANT", "1", 1);
	setenv("DOTNET_DefaultStackSize", "0x40000", 1);
	setenv("DOTNET_ThreadPool_ForceMinWorkerThreads", "1", 1);
	setenv("DOTNET_ThreadPool_ForceMaxWorkerThreads", "4", 1);

	/* Create pipes: parent→child (payload) and child→parent (ack) */
	int pipe_code[2];  /* [0]=read, [1]=write */
	int pipe_ack[2];
	if (pipe(pipe_code) < 0 || pipe(pipe_ack) < 0) {
		fprintf(stderr, "hl_dotnetdriver: pipe() failed\n");
		return 1;
	}

	/* Spawn persistent .NET dispatch server.
	 * Pass pipe fd numbers as command-line arguments. */
	char fd_in_str[16], fd_out_str[16];
	snprintf(fd_in_str, sizeof(fd_in_str), "%d", pipe_code[0]);
	snprintf(fd_out_str, sizeof(fd_out_str), "%d", pipe_ack[1]);

	pid_t pid = vfork();
	if (pid < 0) {
		fprintf(stderr, "hl_dotnetdriver: vfork() failed\n");
		return 1;
	}
	if (pid == 0) {
		/* Child — only exec or _exit allowed after vfork */
		execl("/app/HlDotnetDispatch", "HlDotnetDispatch",
		      fd_in_str, fd_out_str, (char *)NULL);
		_exit(127);
	}

	/* Parent — close the child's pipe ends */
	close(pipe_code[0]);
	close(pipe_ack[1]);
	g_pipe_to_dotnet = pipe_code[1];
	g_pipe_from_dotnet = pipe_ack[0];

	/* Wait for the child to signal ready.
	 * This blocks, the scheduler switches to the child,
	 * CoreCLR starts up, the dispatch server writes the ready byte. */
	char ready;
	if (read(g_pipe_from_dotnet, &ready, 1) != 1) {
		fprintf(stderr,
			"hl_dotnetdriver: .NET dispatch server "
			"failed to start\n");
		return 1;
	}

	/* Register dispatch callback */
	*g_callback_slot = dotnet_dispatch;

	/*
	 * Halt the VM — same pattern as other drivers.
	 * RAX = dispatch entry point address.
	 */
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
