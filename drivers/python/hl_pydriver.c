/*
 * hl_pydriver — Python runtime driver for Hyperlight.
 *
 * Loaded by app-elfloader during boot (evolve).  Initializes CPython,
 * registers a dispatch callback, then halts the VM directly
 *
 * Flow:
 *   boot (evolve):
 *     main() → parse env vars for kernel addresses
 *            → Py_Initialize()
 *            → *callback_slot = py_dispatch
 *            → outl port 108 (halt VM, RAX = dispatch entry)
 *            → host: evolve() returns
 *
 *   host: call("Exec", "print(42)")
 *     dispatch → hyperlight_dispatch_function (kernel)
 *              → py_dispatch(fc, fc_len)
 *              → restore FS_BASE
 *              → PyRun_SimpleString("print(42)")
 *              → print() → write(1,...) → works (fds still open)
 *              → halt
 *
 * TODO: HL_DISPATCH_CALLBACK_PTR / HL_DISPATCH_ENTRY are raw kernel
 * addresses injected as env vars.  Replace with a cleaner interface
 * (vDSO export, syscall, or device ioctl).
 */

#define PY_SSIZE_T_CLEAN
#include <Python.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "../hl_fc.h"

/* ── State ─────────────────────────────────────────────────────── */

static hl_dispatch_fn_t *g_callback_slot;
static uint64_t g_dispatch_entry;
static uint64_t g_py_fsbase;

/* ── FS_BASE MSR save/restore ──────────────────────────────────── */

/*
 * CPython's thread-local storage is keyed off FS_BASE.  The host
 * resets general-purpose registers and FPU between dispatch calls
 * but preserves segment registers and MSRs (including FS_BASE) in
 * both normal and snapshot/restore paths.  We save/restore here as
 * a defensive measure — kernel exception handlers during CoW
 * prefault could theoretically modify FS_BASE.
 */
static inline uint64_t rdmsr_fsbase(void)
{
	uint32_t lo, hi;
	__asm__ volatile("rdmsr" : "=a"(lo), "=d"(hi) : "c"(0xC0000100));
	return ((uint64_t)hi << 32) | lo;
}

static inline void wrmsr_fsbase(uint64_t v)
{
	uint32_t lo = (uint32_t)v, hi = (uint32_t)(v >> 32);
	__asm__ volatile("wrmsr" : : "c"(0xC0000100), "a"(lo), "d"(hi));
}

/* ── Dispatch callback ─────────────────────────────────────────── */

static void py_dispatch(const uint8_t *fc, size_t fc_len)
{
	if (g_py_fsbase)
		wrmsr_fsbase(g_py_fsbase);

	/* Extract the code string from the FunctionCall FlatBuffer */
	size_t code_len;
	const char *code = fc_arg0_string(fc, fc_len, &code_len);
	if (!code)
		return;

	/* NUL-terminate — fc_arg0_string returns a non-terminated slice */
	char stack_buf[4096];
	char *buf;
	if (code_len < sizeof(stack_buf)) {
		buf = stack_buf;
	} else {
		buf = malloc(code_len + 1);
		if (!buf)
			return;
	}
	memcpy(buf, code, code_len);
	buf[code_len] = '\0';

	PyRun_SimpleString(buf);

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

	/* Parse kernel addresses from env vars injected by
	 * dispatch.c's uk_late_initcall. */
	for (char **p = envp; p && *p; p++) {
		if (!strncmp(*p, "HL_DISPATCH_CALLBACK_PTR=", 25))
			g_callback_slot = (hl_dispatch_fn_t *)
				hl_parse_hex(*p + 25);
		else if (!strncmp(*p, "HL_DISPATCH_ENTRY=", 18))
			g_dispatch_entry = hl_parse_hex(*p + 18);
	}

	if (!g_callback_slot || !g_dispatch_entry) {
		fprintf(stderr,
			"hl_pydriver: missing HL_DISPATCH_CALLBACK_PTR "
			"or HL_DISPATCH_ENTRY\n");
		return 1;
	}

	/* Initialize Python while VFS is fully alive — open(),
	 * read(), etc. all work for loading /usr/lib/python3.12/ */
	Py_UTF8Mode = 1;
	Py_Initialize();

	PyRun_SimpleString(
		"import sys\n"
		"sys.argv = ['hl_pydriver']\n");

	/* Save FS_BASE after Python init — the host may clobber it
	 * on dispatch (different thread context or snapshot restore). */
	g_py_fsbase = rdmsr_fsbase();

	/* Register dispatch callback */
	*g_callback_slot = py_dispatch;

	fprintf(stderr, "hl_pydriver: ready\n");
	fflush(stderr);

	/*
	 * Halt the VM.
	 *
	 * Why not exit?  exit_group tears down the VFS fd table
	 * (closes stdout/stderr) and runs atexit handlers (which
	 * call Py_Finalize, destroying the interpreter).  By halting
	 * directly, everything stays intact:
	 *   - fds 0/1/2 remain open → print() works in dispatch
	 *   - Python heap/TLS intact → PyRun_SimpleString works
	 *   - no atexit handlers run → no Py_Finalize
	 *
	 * The elfloader thread is left frozen mid-wait — it never
	 * runs again, which is fine.  On snapshot/restore, the
	 * same frozen state is captured and restored.
	 *
	 * RAX = hyperlight_dispatch_function address, so the host
	 * knows where to set RIP for subsequent call() invocations.
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
