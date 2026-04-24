#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <signal.h>
#include <sys/wait.h>
#include <unistd.h>

/* ANSI colors (same palette as part1) */
#define RST  "\033[0m"
#define BOLD "\033[1m"
#define DIM  "\033[2m"
#define RED  "\033[91m"
#define GRN  "\033[92m"
#define YLW  "\033[93m"
#define MAG  "\033[95m"
#define CYN  "\033[96m"
#define GRY  "\033[90m"

#define CL  "  " DIM "│" RST "  "

static void drain_line(void) {
    int c;
    while ((c = getchar()) != '\n' && c != EOF);
}

static void code_open(void) {
    printf(DIM "  ╭─ " RST BOLD RED "c" RST DIM " ──────────────────────────────────────────────────────\n");
    printf("  │\n" RST);
}

static void code_close(void) {
    printf(DIM "  │\n");
    printf("  ╰──────────────────────────────────────────────────────────\n\n" RST);
}

static void section_result(void) {
    printf(BOLD CYN "\n  ━━ Result " RST DIM "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n" RST);
}

static void section_explain(void) {
    printf(BOLD MAG "\n  ━━ What happened " RST DIM "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n" RST);
}

static void press_enter(const char *question) {
    printf(BOLD YLW "\n  ╔═ ? ════════════════════════════════════════════════════════╗\n" RST);
    printf(BOLD YLW "  ║" RST "  %s\n", question);
    printf(BOLD YLW "  ║\n" RST);
    printf(BOLD YLW "  ║" RST "  " DIM "Press Enter to see what actually happens..." RST "\n");
    printf(BOLD YLW "  ╚════════════════════════════════════════════════════════════╝\n" RST);
    fflush(stdout);
    drain_line();
    printf("\n");
}

/* ============================================================
 * Demo 1: Dangling pointer — return address of local variable
 * ============================================================ */

/* Route through a global so GCC can't replace the return with NULL.
   GCC detects direct 'return &local' and zeroes it as a hardening
   measure; going via a global defeats that analysis. */
static volatile void *g_leak;

__attribute__((noinline))
static int *get_dangling(int val) {
    int local = val;
    g_leak = &local;  /* capture address before return */
    return (int *)(uintptr_t)g_leak;  /* return stale address */
}

static void demo_dangling(void) {
    code_open();
    printf(CL YLW "int" RST " *get_dangling() {\n");
    printf(CL "    " YLW "int" RST " local = " MAG "42" RST ";\n");
    printf(CL "    " GRY "// stack frame exists here\n" RST);
    printf(CL "    " YLW "return" RST " &local;  " RED "// ← stack frame destroyed on return" RST "\n");
    printf(CL "}\n");
    printf(DIM "  │\n" RST);
    printf(CL YLW "int" RST " *p = get_dangling();\n");
    printf(CL GRY "// p now points into a destroyed stack frame\n" RST);
    printf(CL CYN "printf" RST "(" GRN "\"%%d\\n\"" RST ", *p);         " GRY "// ← what will this print?" RST "\n");
    printf(CL "get_dangling(" RED "0xDEAD" RST ");        " GRY "// ← reuse the same stack frame" RST "\n");
    printf(CL CYN "printf" RST "(" GRN "\"%%d\\n\"" RST ", *p);         " GRY "// ← what about now?" RST "\n");
    code_close();

    press_enter("p points to a local that no longer exists. What will *p be after the frame is reused?");
    section_result();

    int *p = get_dangling(42);
    printf("  [after return]  p = %p\n", (void *)p);
    printf("  *p immediately  = " YLW BOLD "%d" RST "  (frame not yet reused)\n", *p);
    get_dangling(0xDEAD);  /* second call: same frame, different value */
    printf("  *p after reuse  = " RED BOLD "0x%X" RST "  ← first call's 'local' corrupted!\n", (unsigned int)*p);

    section_explain();
    printf("  The pointer is " RED BOLD "dangling" RST ": it refers to memory the function\n");
    printf("  no longer owns. The first read returned " YLW "42" RST " only because\n");
    printf("  nothing had reused that address yet.\n\n");
    printf("  A second call to the same function " RED "reuses the identical frame\n" RST);
    printf("  layout — same address, now containing the new call's " YLW "0xDEAD" RST ".\n");
    printf("  p still points there: " RED BOLD "same pointer, different data, no warning" RST ".\n\n");
    printf("  This is why returning " RED "&local_variable" RST " is always wrong:\n");
    printf("    " RED "•" RST " Correct by accident today\n");
    printf("    " RED "•" RST " Silently broken when the stack layout changes\n");
    printf("    " RED "•" RST " Impossible to detect without tooling\n\n");
    printf("  Run " CYN "'make run2-asan'" RST " to see AddressSanitizer catch this.\n");
}

/* ============================================================
 * Demo 2: Use-after-free
 * ============================================================ */
static void demo_use_after_free(void) {
    code_open();
    printf(CL YLW "int" RST " *p = " CYN "malloc" RST "(" YLW "sizeof" RST "(" YLW "int" RST "));\n");
    printf(CL "*p = " MAG "100" RST ";\n");
    printf(CL CYN "free" RST "(p);               " GRY "// memory returned to allocator" RST "\n");
    printf(DIM "  │\n" RST);
    printf(CL CYN "printf" RST "(" GRN "\"%%d\\n\"" RST ", *p); " GRY "// ← read freed memory" RST "\n");
    printf(CL "*p = " RED "999" RST ";              " RED "// ← write freed memory" RST "\n");
    code_close();

    printf("  " DIM "The allocator now owns those bytes. We still have the address.\n\n" RST);

    press_enter("After free(), what will *p read as? Will the write to *p cause an immediate crash?");
    section_result();

    int *p = malloc(sizeof(int));
    *p = 100;
    printf("  Before free: *p = " GRN BOLD "%d" RST "\n", *p);
    free(p);
    printf("  After free:  *p = " RED BOLD "%d" RST "  (value may still linger...)\n", *p);

    /* Trigger allocator activity to corrupt the freed block, then read again */
    void *noise = malloc(sizeof(int));
    free(noise);
    printf("  After alloc/free cycle: *p = " RED BOLD "%d" RST "  (allocator touched it)\n", *p);

    *p = 999;  /* write to freed memory */
    printf("  After *p=999: *p = " RED BOLD "%d" RST "  (wrote to freed heap)\n", *p);

    section_explain();
    printf("  The read after free often returns the old value because the\n");
    printf("  allocator hasn't zeroed the memory. This makes the bug " RED BOLD "invisible\n" RST);
    printf("  in normal testing but " RED BOLD "exploitable" RST " — freed heap blocks contain\n");
    printf("  allocator bookkeeping metadata that an attacker can read.\n\n");
    printf("  The write to freed memory silently corrupts the heap. The\n");
    printf("  crash (if any) happens " RED BOLD "later" RST ", at an unrelated allocation,\n");
    printf("  making the root cause nearly " RED "impossible to trace" RST " without tooling.\n\n");
    printf("  Run " CYN "'make run2-asan'" RST " to see AddressSanitizer pinpoint this.\n");
}

/* ============================================================
 * Demo 3: Double-free — runs in a child process to survive the crash
 * ============================================================ */
static void demo_double_free(void) {
    code_open();
    printf(CL YLW "int" RST " *p = " CYN "malloc" RST "(" YLW "sizeof" RST "(" YLW "int" RST "));\n");
    printf(CL "*p = " MAG "42" RST ";\n");
    printf(CL CYN "free" RST "(p);  " GRY "// correct — first free" RST "\n");
    printf(CL CYN "free" RST "(p);  " RED "// ← second free — double free!" RST "\n");
    code_close();

    printf("  " DIM "Running in a child process so this demo can't kill the shell.\n" RST);

    press_enter("What happens when you free() the same pointer twice?");
    section_result();

    fflush(stdout);
    pid_t pid = fork();
    if (pid == 0) {
        /* child */
        int *p = malloc(sizeof(int));
        *p = 42;
        free(p);
        free(p);  /* double-free: undefined behavior */
        _exit(0); /* should not reach here */
    }

    int status;
    waitpid(pid, &status, 0);

    if (WIFSIGNALED(status)) {
        int sig = WTERMSIG(status);
        const char *signame = (sig == SIGABRT)  ? "SIGABRT" :
                              (sig == SIGSEGV)  ? "SIGSEGV" :
                              (sig == SIGBUS)   ? "SIGBUS"  : "unknown";
        printf("  Child process killed by " RED BOLD "signal %d (%s)" RST "\n\n", sig, signame);
        printf("  " DIM "The crash came from inside malloc's own bookkeeping code,\n");
        printf("  not from the second free() call site.\n" RST);
    } else if (WIFEXITED(status)) {
        printf("  Child exited normally (exit %d) — allocator accepted the double-free.\n",
               WEXITSTATUS(status));
        printf("  " YLW BOLD "Silent heap corruption" RST " — harder to detect than a crash.\n");
    }

    section_explain();
    printf("  Heap allocators track free blocks in a linked list or bitmap.\n");
    printf("  A double-free corrupts that internal structure.\n\n");
    printf("  Outcomes (all possible, all undefined behavior):\n");
    printf("    " RED "•" RST " Crash with SIGABRT from the allocator consistency check\n");
    printf("    " RED "•" RST " Crash with SIGSEGV when the corrupt list is next walked\n");
    printf("    " RED "•" RST " Silent corruption that triggers a crash in unrelated code later\n");
    printf("    " RED "•" RST " Exploitable: attacker shapes heap to make double-free\n");
    printf("      return the same block twice — " RED BOLD "two pointers to one allocation" RST "\n\n");
    printf("  Run " CYN "'make run2-asan'" RST " to see AddressSanitizer catch this cleanly.\n");
}

/* ============================================================
 * Demo 4: Null pointer — forgotten NULL check
 * ============================================================ */
static int *find_value(int *arr, size_t len, int target) {
    for (size_t i = 0; i < len; i++)
        if (arr[i] == target) return &arr[i];
    return NULL;  /* not found */
}

static void demo_null_deref(void) {
    code_open();
    printf(CL YLW "int" RST " *find_value(" YLW "int" RST " *arr, " YLW "size_t" RST " len, " YLW "int" RST " target) {\n");
    printf(CL "    " YLW "for" RST " (...) " YLW "if" RST " (arr[i] == target) " YLW "return" RST " &arr[i];\n");
    printf(CL "    " YLW "return" RST " " RED "NULL" RST ";   " GRY "// not found" RST "\n");
    printf(CL "}\n");
    printf(DIM "  │\n" RST);
    printf(CL YLW "int" RST " arr[] = {" MAG "1" RST "," MAG "2" RST "," MAG "3" RST "};\n");
    printf(CL YLW "int" RST " *p = find_value(arr, " MAG "3" RST ", " RED "99" RST "); " GRY "// 99 not in array" RST "\n");
    printf(CL GRY "// forgot to check: if (p == NULL) ...\n" RST);
    printf(CL CYN "printf" RST "(" GRN "\"%%d\\n\"" RST ", *p);  " RED "// ← dereference NULL" RST "\n");
    code_close();

    printf("  " DIM "99 is not in the array. find_value returns NULL.\n" RST);

    press_enter("find_value returns NULL when not found. We dereference it without checking. What happens?");
    section_result();

    int arr[] = {1, 2, 3};
    int *found = find_value(arr, 3, 2);
    printf("  find_value(arr, 3, 2)  = " GRN BOLD "%d" RST "  (found)\n", *found);

    fflush(stdout);
    pid_t pid = fork();
    if (pid == 0) {
        int *p = find_value(arr, 3, 99);
        printf("  find_value(arr, 3, 99) returned pointer %p\n", (void *)p);
        printf("  Dereferencing NULL...\n");
        fflush(stdout);
        printf("  *p = %d\n", *p);  /* crashes here */
        _exit(0);
    }

    int status;
    waitpid(pid, &status, 0);
    if (WIFSIGNALED(status)) {
        int sig = WTERMSIG(status);
        printf("  Dereference NULL → " RED BOLD "signal %d (SIGSEGV)" RST "\n", sig);
    }

    section_explain();
    printf("  NULL is address 0x0. The OS maps no memory there.\n");
    printf("  Dereferencing it always causes a " RED BOLD "segmentation fault" RST ".\n\n");
    printf("  The silent problem: " BOLD "nothing forces you to check the return value" RST ".\n");
    printf("  In C, NULL is just a convention — the type of find_value is\n");
    printf("  " YLW "int *" RST ", which could be a valid pointer or NULL. There is no\n");
    printf("  difference in the type system.\n\n");
    printf("  Every pointer in C is implicitly " RED "nullable" RST ". The compiler\n");
    printf("  will not warn you if you forget the NULL check.\n");
    printf("  Result: NULL dereferences are one of the most common C crashes.\n");
}

/* ============================================================ */

static void print_menu(void) {
    printf(RED BOLD "\n  ┌──────────────────────────────────────┐\n");
    printf("  │      C Pointer Lifetime Horror       │\n");
    printf("  └──────────────────────────────────────┘\n" RST);
    printf(DIM "  Part 2: dangling pointers, use-after-free, double-free, null.\n\n" RST);
    printf("  " CYN "1)" RST " Dangling pointer (return &local)\n");
    printf("  " CYN "2)" RST " Use-after-free\n");
    printf("  " CYN "3)" RST " Double-free (runs in a subprocess)\n");
    printf("  " CYN "4)" RST " Null pointer — forgotten NULL check\n");
    printf("  " DIM "q) Quit\n" RST);
    printf("\n  " BOLD "> " RST);
    fflush(stdout);
}

int main(void) {
    int ch;
    while (1) {
        print_menu();
        ch = getchar();
        if (ch == EOF) break;
        drain_line();
        printf("\n");

        switch (ch) {
            case '1': demo_dangling();       break;
            case '2': demo_use_after_free(); break;
            case '3': demo_double_free();    break;
            case '4': demo_null_deref();     break;
            case 'q': case 'Q':
                printf(DIM "\n  Exiting. Leaks not included.\n\n" RST);
                return 0;
            default:
                printf(RED "  Unknown option '%c'.\n" RST, ch);
        }
    }
    return 0;
}
