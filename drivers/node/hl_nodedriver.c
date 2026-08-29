/*
 * hl_nodedriver — Node.js runtime driver for Hyperlight.
 *
 * Spawns a persistent Node.js child process during boot via
 * vfork+exec.  The child runs a dispatch loop that reads code
 * from a pipe, evals it, and writes an ack byte back.
 *
 * Flow:
 *   boot (evolve):
 *     main() → create pipes
 *            → write bootstrap JS to /tmp/hl_bootstrap.js
 *            → vfork + exec("node", "/tmp/hl_bootstrap.js")
 *            → read "ready" ack from child (blocks, scheduler
 *              switches to child, V8 starts up, child signals)
 *            → register dispatch callback, halt
 *
 *   host: call("Exec", "console.log(42)")
 *     dispatch → hyperlight_dispatch_function (kernel)
 *              → node_dispatch(fc, fc_len)
 *              → write [len:u64][code] to pipe
 *              → read ack byte (blocks, scheduler switches to
 *                child, child evals code, writes ack)
 *              → halt
 *
 * The child Node process stays alive across dispatches — no V8
 * startup cost per call, and no exit/cleanup hang.
 *
 * TODO: The persistent child is a workaround for a vfork+exec hang
 * where the child Node process can't exit cleanly.  The root cause
 * is likely that hyperlight_dispatch_function doesn't restore the
 * cooperative scheduler's state (current thread, run queues) on
 * re-entry — the same class of bug we fixed for VFS, fd table,
 * and SYSCALL MSRs.  Once scheduler state is properly restored on
 * dispatch re-entry, a simpler per-dispatch vfork+exec+waitpid
 * approach should work.
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
static int g_pipe_to_node;    /* parent writes code here */
static int g_pipe_from_node;  /* parent reads ack here */

/* ── Dispatch callback ─────────────────────────────────────────── */

static int node_dispatch(const uint8_t *fc, size_t fc_len)
{
	/* Extract the code string from the FunctionCall FlatBuffer */
	size_t code_len;
	const char *code = fc_arg0_string(fc, fc_len, &code_len);
	if (!code)
		return -1;

	/* Send length (8 bytes LE) + code to the child */
	uint64_t len64 = (uint64_t)code_len;
	if (write(g_pipe_to_node, &len64, 8) != 8) {
		fprintf(stderr, "hl_nodedriver: pipe write (len) failed\n");
		fflush(stderr);
		return -1;
	}
	const char *p = code;
	size_t remaining = code_len;
	while (remaining > 0) {
		ssize_t n = write(g_pipe_to_node, p, remaining);
		if (n <= 0) {
			fprintf(stderr, "hl_nodedriver: pipe write (data) failed\n");
			fflush(stderr);
			return -1;
		}
		p += n;
		remaining -= n;
	}

	/* Wait for ack — child sends 0x00 on success, 0x01 on error.
	 * This read blocks and yields to the cooperative scheduler,
	 * which switches to the child Node thread. */
	char ack = 1;
	if (read(g_pipe_from_node, &ack, 1) != 1) {
		fprintf(stderr, "hl_nodedriver: ack read failed\n");
		fflush(stderr);
		return -1;
	}
	return ack != 0 ? -1 : 0;
}

/* ── Bootstrap JS ─────────────────────────────────────────────── */

static int write_bootstrap(int fd_in, int fd_out)
{
	const char *path = "/tmp/hl_bootstrap.js";
	FILE *f = fopen(path, "w");
	if (!f) {
		fprintf(stderr, "hl_nodedriver: cannot create %s\n", path);
		return -1;
	}

	fprintf(f,
		"'use strict';\n"
		"const fs = require('fs');\n"
		"const fd_in = %d;\n"
		"const fd_out = %d;\n"
		"\n"
		"// Expose CJS module globals so eval'd code can require()\n"
		"globalThis.require = require;\n"
		"globalThis.module = module;\n"
		"globalThis.__dirname = __dirname;\n"
		"globalThis.__filename = __filename;\n"
		"\n"
		"// Signal ready to parent\n"
		"fs.writeSync(fd_out, Buffer.from([0]));\n"
		"\n"
		"// Dispatch loop — runs forever, one eval per iteration\n"
		"const hdr = Buffer.alloc(8);\n"
		"while (true) {\n"
		"  let n = fs.readSync(fd_in, hdr, 0, 8);\n"
		"  if (n < 8) break;\n"
		"  let len = Number(hdr.readBigUInt64LE(0));\n"
		"  let buf = Buffer.alloc(len);\n"
		"  let off = 0;\n"
		"  while (off < len) {\n"
		"    n = fs.readSync(fd_in, buf, off, len - off);\n"
		"    if (n <= 0) process.exit(1);\n"
		"    off += n;\n"
		"  }\n"
		"  let code = buf.toString('utf8');\n"
		"  try {\n"
		"    let result = (0, eval)(code);\n"
		"    if (result !== undefined) console.log(result);\n"
		"    fs.writeSync(fd_out, Buffer.from([0]));  // success ack\n"
		"  } catch (e) {\n"
		"    console.error(e.stack || e);\n"
		"    fs.writeSync(fd_out, Buffer.from([1]));  // error ack\n"
		"  }\n"
		"}\n",
		fd_in, fd_out);

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
			"hl_nodedriver: missing HL_DISPATCH_CALLBACK_PTR "
			"or HL_DISPATCH_ENTRY\n");
		return 1;
	}

	/* Create pipes: parent→child (code) and child→parent (ack) */
	int pipe_code[2];  /* [0]=read, [1]=write */
	int pipe_ack[2];
	if (pipe(pipe_code) < 0 || pipe(pipe_ack) < 0) {
		fprintf(stderr, "hl_nodedriver: pipe() failed\n");
		return 1;
	}

	/* Write bootstrap JS with the pipe fd numbers baked in */
	if (write_bootstrap(pipe_code[0], pipe_ack[1]) < 0)
		return 1;

	/* Spawn persistent Node child.
	 * vfork: parent blocks until child calls exec.
	 * After exec, parent resumes.  Child inherits all fds. */
	pid_t pid = vfork();
	if (pid < 0) {
		fprintf(stderr, "hl_nodedriver: vfork() failed\n");
		return 1;
	}
	if (pid == 0) {
		/* Child — only exec or _exit allowed after vfork */
		execl("/usr/bin/node", "node", "/tmp/hl_bootstrap.js",
		      (char *)NULL);
		_exit(127);
	}

	/* Parent — close the child's pipe ends */
	close(pipe_code[0]);
	close(pipe_ack[1]);
	g_pipe_to_node = pipe_code[1];
	g_pipe_from_node = pipe_ack[0];

	/* Wait for the child to signal ready.
	 * This blocks, the scheduler switches to the child,
	 * V8 starts up, the bootstrap writes the ready byte. */
	char ready;
	if (read(g_pipe_from_node, &ready, 1) != 1) {
		fprintf(stderr, "hl_nodedriver: child failed to start\n");
		return 1;
	}

	/* Register dispatch callback */
	*g_callback_slot = node_dispatch;

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
