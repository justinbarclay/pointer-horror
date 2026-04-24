#define _POSIX_C_SOURCE 200809L
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

/* ANSI colors */
#define RST  "\033[0m"
#define BOLD "\033[1m"
#define DIM  "\033[2m"
#define RED  "\033[91m"
#define GRN  "\033[92m"
#define YLW  "\033[93m"
#define MAG  "\033[95m"
#define CYN  "\033[96m"

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

static void await_enter(void) {
    printf(DIM "\n  [press Enter to continue]" RST);
    fflush(stdout);
    drain_line();
    printf("\n");
}

/* ─── the expensive function we'll memoize ──────────────────────────── */

static double slow_sqrt(double x) {
    /* Burn time so the cache speedup is obvious. */
    volatile double acc = 0.0;
    for (int i = 0; i < 80000000; i++) acc += 1e-15;
    return sqrt(x) + (acc * 0.0); /* acc*0 keeps optimizer honest */
}

/* ─── a demo-grade single-slot memoizer ─────────────────────────────── */

typedef double (*dfn_t)(double);

static dfn_t  memo_fn  = NULL;
static double memo_key = NAN;
static double memo_val = 0.0;

static double memo_dispatch(double x) {
    if (!isnan(memo_key) && memo_key == x) {
        printf(GRN "  [cache hit]" RST "\n");
        return memo_val;
    }
    printf(YLW "  [cache miss — computing…]" RST "\n");
    memo_key = x;
    memo_val = memo_fn(x);
    return memo_val;
}

/*
 * memoize: the function whose type we're hiding.
 *
 * Plain C declaration (no typedef):
 *
 *   double (*memoize(double (*fn)(double)))(double)
 *
 * Expanded to a pointer-to-function:
 *
 *   double (*(*f)(double (*)(double)))(double) = memoize;
 */
dfn_t memoize(dfn_t fn) {
    memo_fn  = fn;
    memo_key = NAN;
    return memo_dispatch;
}

/* ─── timing ─────────────────────────────────────────────────────────── */

static int64_t now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (int64_t)ts.tv_sec * 1000000000LL + ts.tv_nsec;
}

/* ─── main ───────────────────────────────────────────────────────────── */

int main(void) {
    printf("\n");
    printf(RED BOLD "  ┌──────────────────────────────────────────┐\n");
    printf(         "  │   ☠   Bonus: The Cryptic Signature   ☠   │\n");
    printf(         "  └──────────────────────────────────────────┘\n" RST);
    printf(DIM      "  One type.  What does it do?\n\n" RST);

    /* ── show the puzzle ── */
    code_open();
    printf(CL BOLD "double (*(*f)(double (*)(double)))(double);\n" RST);
    code_close();

    printf(BOLD YLW "  ╔═ ? ════════════════════════════════════════════════════════╗\n" RST);
    printf(BOLD YLW "  ║" RST "  What type is f?\n");
    printf(BOLD YLW "  ║\n" RST);
    printf(BOLD YLW "  ║" RST "  " DIM "Write your best guess, then press Enter." RST "\n");
    printf(BOLD YLW "  ╚════════════════════════════════════════════════════════════╝\n" RST);
    printf("\n  > ");
    fflush(stdout);

    char buf[256] = {0};
    if (fgets(buf, sizeof(buf), stdin) == NULL) buf[0] = '\0';

    /* ── optional hint ── */
    printf(BOLD YLW "  ╔═ ? ════════════════════════════════════════════════════════╗\n" RST);
    printf(BOLD YLW "  ║" RST "  Want a hint? (y/n)\n");
    printf(BOLD YLW "  ╚════════════════════════════════════════════════════════════╝\n" RST);
    printf("\n  > ");
    fflush(stdout);

    char hint_buf[8] = {0};
    if (fgets(hint_buf, sizeof(hint_buf), stdin) == NULL) hint_buf[0] = '\0';

    if (hint_buf[0] == 'y' || hint_buf[0] == 'Y') {
        printf(BOLD CYN "\n  ━━ Hint " RST DIM "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n" RST);
        printf("  " BOLD "f" RST " can be called like this:\n\n");
        code_open();
        printf(CL "/* slow_sqrt has type: double (*)(double) */\n");
        printf(CL "f(slow_sqrt);\n");
        code_close();
        await_enter();
    }

    /* ── reveal ── */
    printf(BOLD CYN "\n  ━━ Answer " RST DIM "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n" RST);

    printf("  " BOLD "f" RST " is a pointer to a function that:\n\n");
    printf("    takes  " YLW "double (*)(double)" RST "   ← a function pointer\n");
    printf("    returns " GRN "double (*)(double)" RST "  ← another function pointer\n\n");
    printf("  Shape of a " BOLD "higher-order function" RST ": it wraps one function\n");
    printf("  and hands back a new one.  Classic use case: " BOLD "memoization" RST ".\n\n");

    /* annotated breakdown */
    code_open();
    printf(CL "double  (* (*f) (double (*)(double)) ) (double)\n");
    printf(CL "           │    └──── takes: fn ─────┘   │\n");
    printf(CL "           └────────── returns ───────────┘\n");
    printf(CL "  a function  (double) → double\n");
    code_close();

    printf("  In TypeScript the same shape is just:\n\n");
    printf(DIM "  ╭─ " RST BOLD CYN "typescript" RST DIM " ────────────────────────────────────────────────\n");
    printf("  │\n" RST);
    printf(CL "type NumFn = (x: number) => number;\n");
    printf(CL "const f: (fn: NumFn) => NumFn = memoize;\n");
    printf(DIM "  │\n");
    printf("  ╰──────────────────────────────────────────────────────────\n\n" RST);

    await_enter();

    /* ── live demo ── */
    printf(BOLD MAG "\n  ━━ Live demo " RST DIM "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n" RST);

    code_open();
    printf(CL "/* a function pointer: double → double */\n");
    printf(CL "typedef double (*dfn_t)(double);\n");
    printf(CL "\n");
    printf(CL "/* memoize: takes a function, returns a function */\n");
    printf(CL "dfn_t memoize(dfn_t fn) { /* ... */ }\n");
    printf(CL "\n");
    printf(CL "/* f is memoize — but written without the typedef: */\n");
    printf(CL "double (*(*" BOLD YLW "f" RST ")(double (*)(double)))(double) = memoize;\n");
    printf(CL DIM "/*          ^ f is the variable                    */\n" RST);
    printf(CL "\n");
    printf(CL "dfn_t fast_sqrt = f(slow_sqrt);\n");
    printf(CL "\n");
    printf(CL "fast_sqrt(2.0);   /* first call  */\n");
    printf(CL "fast_sqrt(2.0);   /* second call */\n");
    code_close();

    /* run it */
    double (*(*f)(dfn_t))(double) = memoize;
    dfn_t fast_sqrt = f(slow_sqrt);

    printf("  Call 1 — fast_sqrt(2.0)\n");
    int64_t t0 = now_ns();
    double  r1 = fast_sqrt(2.0);
    int64_t t1 = now_ns();
    printf("  result = %.10f   (%lld ms)\n\n", r1, (long long)(t1 - t0) / 1000000LL);

    printf("  Call 2 — fast_sqrt(2.0)  ← same input\n");
    int64_t t2 = now_ns();
    double  r2 = fast_sqrt(2.0);
    int64_t t3 = now_ns();
    printf("  result = %.10f   (%lld ns)\n\n", r2, (long long)(t3 - t2));

    printf("  Call 3 — fast_sqrt(9.0)  ← different input\n");
    int64_t t4 = now_ns();
    double  r3 = fast_sqrt(9.0);
    int64_t t5 = now_ns();
    printf("  result = %.10f   (%lld ms)\n\n", r3, (long long)(t5 - t4) / 1000000LL);

    printf(DIM "  The type that looks like line noise is what makes\n");
    printf(    "  map(), filter(), memoize(), and every higher-order\n");
    printf(    "  function in C possible.\n" RST);
    printf("\n");

    return 0;
}
