/*
 * hl_nodedriver — Node.js runtime driver for Hyperlight.
 *
 * Loaded by app-elfloader during boot (evolve).  Registers a
 * dispatch callback that runs Node.js code via vfork+exec on
 * each call.
 *
 * Flow:
 *   boot (evolve):
 *     main() → parse env vars for kernel addresses
 *            → *callback_slot = node_dispatch
 *            → outl port 108 (halt VM, RAX = dispatch entry)
 *
 *   host: call("Exec", "console.log(42)")
 *     dispatch → hyperlight_dispatch_function (kernel)
 *              → node_dispatch(fc, fc_len)
 *              → write code to /tmp/hl_dispatch.js
 *              → vfork + execl("node", "/tmp/hl_dispatch.js")
 *              → waitpid → halt
 *
 * Each dispatch pays full V8 startup cost (~50-100ms).  A future
 * optimization would keep a persistent Node child process.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <sys/wait.h>

#include "../hl_fc.h"

/* ── State ─────────────────────────────────────────────────────── */

static hl_dispatch_fn_t *g_callback_slot;
static uint64_t g_dispatch_entry;

/* ── Dispatch callback ─────────────────────────────────────────── */

static void node_dispatch(const uint8_t *fc, size_t fc_len)
{
	/* Extract the code string from the FunctionCall FlatBuffer */
	size_t code_len;
	const char *code = fc_arg0_string(fc, fc_len, &code_len);
	if (!code)
		return;

	/* Build: "user_code\n; process.exit(0)" to force clean exit.
	 * Node's event loop can hang during shutdown in this env. */
	static const char suffix[] = "\n; process.exit(0)";
	size_t total = code_len + sizeof(suffix); /* includes NUL */
	char stack_buf[4096];
	char *buf;
	if (total <= sizeof(stack_buf)) {
		buf = stack_buf;
	} else {
		buf = malloc(total);
		if (!buf)
			return;
	}
	memcpy(buf, code, code_len);
	memcpy(buf + code_len, suffix, sizeof(suffix));

	pid_t pid = vfork();
	if (pid == 0) {
		execl("/usr/bin/node", "node", "-e", buf, NULL);
		_exit(127);
	}
	if (pid > 0) {
		/* Non-blocking wait with timeout — Node's exit can hang
		 * in the unikernel due to exit_group/cleanup issues. */
		int status = 0;
		int tries = 0;
		while (tries < 5000) {
			int r = waitpid(pid, &status, WNOHANG);
			if (r > 0)
				break;
			if (r < 0)
				break;
			/* Yield to the cooperative scheduler */
			usleep(1000);
			tries++;
		}
		if (tries >= 5000) {
			/* Timed out — kill the child */
			kill(pid, 9);
			waitpid(pid, &status, 0);
		}
	} else {
		fprintf(stderr, "hl_nodedriver: vfork failed\n");
	}

	if (buf != stack_buf)
		free(buf);

	fflush(stdout);
	fflush(stderr);
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
			"hl_nodedriver: missing HL_DISPATCH_CALLBACK_PTR "
			"or HL_DISPATCH_ENTRY\n");
		return 1;
	}

	/* Register dispatch callback */
	*g_callback_slot = node_dispatch;

	fprintf(stderr, "hl_nodedriver: ready\n");
	fflush(stderr);

	/*
	 * Halt the VM — same pattern as pydriver.
	 * RAX = dispatch entry point address.
	 */
	__asm__ volatile(
		"andq $~0xf, %%rsp\n\t"
		"movq %0, %%rax\n\t"
		"movw $108, %%dx\n\t"
		"outl %%eax, %%dx\n\t"
		"cli\n\t"
		"hlt\n\t"
		: : "r"(g_dispatch_entry) : "rax", "rdx"
	);
	__builtin_unreachable();
}
