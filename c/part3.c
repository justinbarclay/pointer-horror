#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <time.h>

/* ─── colours ──────────────────────────────────────────────────────── */
#define RST  "\x1b[0m"
#define BOLD "\x1b[1m"
#define DIM  "\x1b[2m"
#define RED  "\x1b[91m"
#define GRN  "\x1b[92m"
#define YLW  "\x1b[93m"
#define BLU  "\x1b[94m"
#define MAG  "\x1b[95m"
#define CYN  "\x1b[96m"
#define GRY  "\x1b[90m"

static void code_open_c(void) {
    printf(DIM "  ╭─ " RST BOLD RED "c" RST " "
           DIM "────────────────────────────────────────────────────\n"
           "  │" RST "\n");
}
static void code_close(void) {
    printf(DIM "  │\n  ╰──────────────────────────────────────────────────────────" RST "\n\n");
}
static void section_result(void) {
    printf(BOLD CYN "\n  ━━ Result " RST DIM
           "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" RST "\n\n");
}
static void section_explain(void) {
    printf(BOLD MAG "\n  ━━ What happened " RST DIM
           "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" RST "\n\n");
}
static void press_enter(const char *q) {
    printf(BOLD YLW "\n  ╔═ ? ════════════════════════════════════════════════════════╗\n"
           "  ║" RST "  %s\n"
           BOLD YLW "  ║\n"
           "  ║" RST "  " DIM "Press Enter to continue..." RST "\n"
           BOLD YLW "  ╚════════════════════════════════════════════════════════════╝" RST "\n", q);
    fflush(stdout);
    char buf[256];
    if (!fgets(buf, sizeof buf, stdin)) exit(0);
    printf("\n");
}
static void drain_line(void) { int c; while ((c = getchar()) != '\n' && c != EOF) {} }

/* ─── timing ───────────────────────────────────────────────────────── */
static int64_t ns_now(void) {
    struct timespec t;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return (int64_t)t.tv_sec * 1000000000LL + t.tv_nsec;
}
static int cmp_i64(const void *a, const void *b) {
    int64_t x = *(const int64_t *)a, y = *(const int64_t *)b;
    return (x > y) - (x < y);
}

/* ─── manual reference count ───────────────────────────────────────── */
typedef struct { int value; int refs; } Shared;

static Shared *shared_new(int v) {
    Shared *s = malloc(sizeof *s);
    if (!s) { fputs("OOM\n", stderr); exit(1); }
    s->value = v;
    s->refs  = 1;
    return s;
}
static void shared_retain(Shared *s)   { s->refs++; }
static void shared_release(Shared **s) {
    if (!*s) return;
    if (--(*s)->refs == 0) { free(*s); *s = NULL; }
}

/* ══════════════════════════════════════════════════════════════════════
   Demo 1 — shared ownership (verbosity)
   ══════════════════════════════════════════════════════════════════════ */
static void demo_shared_ownership(void) {
    const char *p = "  " DIM "│" RST "  ";

    printf("  To safely share a heap value between two handles in C you need\n"
           "  a manual reference count. This is the " RED "minimum toy version" RST "\n"
           "  (single-threaded, no cycle detection):\n\n");

    code_open_c();
    printf("%s" YLW "typedef struct" RST " { " YLW "int" RST " value; "
           YLW "int" RST " refs; } " GRN "Shared" RST ";\n\n", p);
    printf("%s" GRN "Shared" RST " *" CYN "shared_new" RST "(" YLW "int" RST " v) {\n"
           "%s    " GRN "Shared" RST " *s = " CYN "malloc" RST "(" YLW "sizeof" RST " *s);\n"
           "%s    " YLW "if" RST " (!s) exit(" MAG "1" RST ");\n"
           "%s    s->value = v; s->refs = " MAG "1" RST "; " YLW "return" RST " s;\n"
           "%s}\n", p, p, p, p, p);
    printf("%s" YLW "void" RST " " CYN "shared_retain" RST "(" GRN "Shared" RST " *s)"
           "    { s->refs++; }\n", p);
    printf("%s" YLW "void" RST " " CYN "shared_release" RST "(" GRN "Shared" RST " **s) {\n"
           "%s    " YLW "if" RST " (!*s) " YLW "return" RST ";\n"
           "%s    " YLW "if" RST " (--(*s)->refs == " MAG "0" RST ")"
           " { " CYN "free" RST "(*s); *s = " RED "NULL" RST "; }\n"
           "%s}\n", p, p, p, p);
    code_close();

    press_enter("~15 lines of boilerplate to share one integer. How does Rust compare?");
    section_result();

    Shared *a = shared_new(42);
    shared_retain(a);
    Shared *b = a;
    printf("  Created:    a->value=%d  refs=%d\n", a->value, a->refs);
    shared_release(&a);
    printf("  Released a: refs=%d  (b still alive)\n", b->refs);
    shared_release(&b);
    printf("  Released b: freed — no leak, no double-free\n\n");

    printf("  " BOLD "Lines of boilerplate to safely share one heap value:\n\n" RST);
    printf("  " RED  "C (manual refcount)" RST "  ~15 lines  " RED  "███████████████" RST "\n");
    printf("  " GRN  "Rust (Rc<T>)" RST "          4 lines  " GRN  "████" RST "\n");
    printf("  " BLU  "TypeScript (GC)" RST "        0 lines  "
           DIM  "(the GC tracks references invisibly)" RST "\n");

    section_explain();
    printf("  " BOLD "Rc<T>" RST " in Rust is the same mechanism — a counter on the heap.\n"
           "  The difference: Rust generates the boilerplate for you at compile\n"
           "  time. Same runtime overhead, a fraction of the code.\n\n"
           "  TypeScript makes sharing " BLU "invisible" RST " — zero boilerplate, but you\n"
           "  give up control over " YLW "when" RST " memory is freed. See demo 2.\n");
}

/* ══════════════════════════════════════════════════════════════════════
   Demo 2 — allocation latency benchmark
   ══════════════════════════════════════════════════════════════════════ */
#define BATCH_SIZE  1000
#define BATCH_COUNT 500

typedef struct { int a, b, c, d, e, f; } Node;

static void demo_alloc_latency(void) {
    const char *p = "  " DIM "│" RST "  ";

    printf("  Each batch: " YLW "%d" RST " individual " CYN "malloc" RST "/"
           CYN "free" RST " calls — one per struct.\n"
           "  Total: " YLW "500,000" RST " allocations. Compiled with " GRN "-O2" RST ".\n\n",
           BATCH_SIZE);

    code_open_c();
    printf("%s" GRN "Node" RST " **nodes = " CYN "malloc" RST
           "(" YLW "sizeof" RST "(" GRN "Node" RST "*) * BATCH_SIZE);\n", p);
    printf("%s" YLW "for" RST " (i = 0; i < BATCH_SIZE; i++)\n"
           "%s    nodes[i] = " CYN "malloc" RST "(" YLW "sizeof" RST "(" GRN "Node" RST "));\n"
           "%s" GRY "/* checksum, then: */" RST "\n", p, p, p);
    printf("%s" YLW "for" RST " (i = 0; i < BATCH_SIZE; i++)\n"
           "%s    " CYN "free" RST "(nodes[i]); "
           GRY "// freed deterministically, right here" RST "\n", p, p);
    code_close();

    press_enter("C calls free() immediately. Will batch times be flat and consistent?");
    section_result();

    int64_t *times = malloc(sizeof(int64_t) * BATCH_COUNT);
    if (!times) { fputs("OOM\n", stderr); return; }
    volatile long checksum = 0;

    for (int b = 0; b < BATCH_COUNT; b++) {
        int filled = (b + 1) * 38 / BATCH_COUNT;
        printf("\r  " DIM "[" RST);
        for (int x = 0; x < 38; x++)
            printf("%s", x < filled ? GRN "▓" RST : DIM "░" RST);
        printf(DIM "]" RST "  %d/%d   ", b + 1, BATCH_COUNT);
        fflush(stdout);

        int64_t t0 = ns_now();
        Node **nodes = malloc(sizeof(Node *) * BATCH_SIZE);
        for (int i = 0; i < BATCH_SIZE; i++) {
            nodes[i] = malloc(sizeof(Node));
            nodes[i]->a = i; nodes[i]->b = i*2; nodes[i]->c = i*3;
            nodes[i]->d = i*4; nodes[i]->e = i*5; nodes[i]->f = i*6;
        }
        for (int i = 0; i < BATCH_SIZE; i++) checksum += nodes[i]->a;
        for (int i = 0; i < BATCH_SIZE; i++) free(nodes[i]);
        free(nodes);
        times[b] = (ns_now() - t0) / 1000; /* µs */
    }
    printf("\r  " GRN "Complete." RST
           "                                              \n\n");

    qsort(times, BATCH_COUNT, sizeof(int64_t), cmp_i64);
    int64_t p50  = times[BATCH_COUNT / 2];
    int64_t p95  = times[BATCH_COUNT * 95 / 100];
    int64_t tmax = times[BATCH_COUNT - 1];
    int64_t sc   = tmax > 0 ? tmax : 1;
#define BAR(n) (int)((n) * 30 / sc)
    printf("  " BOLD "Batch time (%d batches × %d allocs):" RST "\n\n",
           BATCH_COUNT, BATCH_SIZE);
    printf("   p50  " GRN BOLD "%5lld" RST "µs  " GRN, (long long)p50);
    for (int x = 0; x < BAR(p50); x++) { printf("█"); } printf(RST "\n");
    printf("   p95  " YLW BOLD "%5lld" RST "µs  " YLW, (long long)p95);
    for (int x = 0; x < BAR(p95); x++) { printf("█"); } printf(RST "\n");
    printf("   max  " RED BOLD "%5lld" RST "µs  " RED, (long long)tmax);
    for (int x = 0; x < 30;       x++) { printf("█"); } printf(RST "\n\n");
    printf("  " DIM "(checksum=%ld — prevents dead-code elimination)" RST "\n", (long)checksum);
#undef BAR
    free(times);

    section_explain();
    printf("  " CYN "free()" RST " is " BOLD "deterministic" RST " — it runs the instant you call it.\n"
           "  C and Rust batch times are " GRN "flat" RST ": p50 ≈ p95 ≈ max.\n\n"
           "  Run the Rust and TypeScript part 3 CLIs and compare:\n"
           "    " GRN "Rust" RST "       — should match C closely (same allocator, no GC)\n"
           "    " BLU "TypeScript" RST " — watch p95 and max for GC pause spikes\n\n"
           "  " YLW "Note:" RST " for a fair comparison use optimised builds:\n"
           "    C:    " DIM "make run3" RST "  (already uses -O2)\n"
           "    Rust: " DIM "cargo run --release --bin part3" RST "\n");
}

/* ─── menu ─────────────────────────────────────────────────────────── */
static void print_menu(void) {
    printf(RED BOLD "\n  ┌──────────────────────────────────────┐\n"
                    "  │   C — Performance & Verbosity        │\n"
                    "  └──────────────────────────────────────┘" RST "\n");
    printf(DIM "  Part 3: memory management cost in code and latency.\n\n" RST);
    printf("  " CYN "1)" RST " Shared ownership — verbosity comparison\n");
    printf("  " CYN "2)" RST " Allocation latency — deterministic free vs GC\n");
    printf("  " DIM "q) Quit" RST "\n");
    printf("\n  " BOLD "> " RST);
    fflush(stdout);
}

int main(void) {
    char line[64];
    for (;;) {
        print_menu();
        if (!fgets(line, sizeof line, stdin)) break;
        if (!strchr(line, '\n')) drain_line();
        printf("\n");
        char ch = line[0];
        if      (ch == '1')            demo_shared_ownership();
        else if (ch == '2')            demo_alloc_latency();
        else if (ch == 'q' || ch == 'Q') {
            printf(DIM "\n  Exiting. All memory freed by free().\n\n" RST);
            break;
        } else printf(RED "  Unknown option.\n" RST);
    }
    return 0;
}
