#include <stdio.h>
#include <string.h>

/* ANSI colors */
#define RST  "\033[0m"
#define BOLD "\033[1m"
#define DIM  "\033[2m"
#define RED  "\033[91m"
#define GRN  "\033[92m"
#define YLW  "\033[93m"
#define MAG  "\033[95m"
#define CYN  "\033[96m"
#define GRY  "\033[90m"

/* Code-line prefix: dim left border */
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
 * Demo 1: sizeof decay
 * ============================================================ */
static void sizeof_in_function(int arr[]) {
    printf("  sizeof(arr) inside function : " RED BOLD "%zu bytes" RST "  ← pointer size, not array size\n", sizeof(arr));
}

static void demo_sizeof_decay(void) {
    code_open();
    printf(CL YLW "void" RST " sizeof_in_function(" YLW "int" RST " arr[]) {\n");
    printf(CL GRY "    // compiler rewrites: int arr[]  →  int *arr\n" RST);
    printf(CL "    " CYN "printf" RST "(" GRN "\"%%zu\\n\"" RST ", " CYN "sizeof" RST "(arr));\n");
    printf(CL "}\n");
    printf(DIM "  │\n" RST);
    printf(CL YLW "int" RST " data[10] = {0};\n");
    printf(CL CYN "printf" RST "(" GRN "\"%%zu\\n\"" RST ", " CYN "sizeof" RST "(data));    " GRY "// ← in main" RST "\n");
    printf(CL "sizeof_in_function(data);           " GRY "// ← inside function" RST "\n");
    code_close();

    press_enter("sizeof(data) in main is 40 bytes (10 ints x 4). What will sizeof(arr) be inside the function?");
    section_result();

    int data[10] = {0};
    printf("  sizeof(data) in main          : " GRN BOLD "%zu bytes" RST "\n", sizeof(data));
    sizeof_in_function(data);

    section_explain();
    printf("  The compiler silently rewrites " BOLD YLW "int arr[]" RST " as " BOLD RED "int *arr" RST ".\n");
    printf("  This is \"" BOLD "array decay" RST "\". At the call boundary, the array becomes\n");
    printf("  a raw pointer — all size information is " RED BOLD "permanently lost" RST ".\n\n");
    printf("    sizeof(data) in main     = " GRN BOLD "%zu" RST "  (10 ints × 4 bytes)\n", sizeof(int) * 10);
    printf("    sizeof(arr)  in function = " RED BOLD "%zu" RST "  (pointer size on this arch)\n\n", sizeof(int *));
    printf("  Every C function that takes an array needs:\n");
    printf("    " YLW "void" RST " process(" YLW "int" RST " *arr, " YLW "size_t" RST " len)  " GRY "// 'len' compensates for decay" RST "\n");
}

/* ============================================================
 * Demo 2: Silent out-of-bounds read
 * ============================================================ */
static void demo_oob_read(void) {
    code_open();
    printf(CL YLW "int" RST " arr[3] = {" MAG "10" RST ", " MAG "20" RST ", " MAG "30" RST "};\n");
    printf(CL CYN "printf" RST "(" GRN "\"%%d\\n\"" RST ", arr[" MAG "3" RST "]);  " GRY "// one past the end" RST "\n");
    printf(CL CYN "printf" RST "(" GRN "\"%%d\\n\"" RST ", arr[" RED "9" RST "]);  " GRY "// way past the end" RST "\n");
    code_close();

    press_enter("What do you think arr[3] and arr[9] print? A crash? An error? Specific values?");
    section_result();

    int arr[3] = {10, 20, 30};
    printf("  arr[0] = " GRN BOLD "%10d" RST "  (valid)\n",  arr[0]);
    printf("  arr[1] = " GRN BOLD "%10d" RST "  (valid)\n",  arr[1]);
    printf("  arr[2] = " GRN BOLD "%10d" RST "  (valid)\n",  arr[2]);
    printf("  arr[3] = " RED BOLD "%10d" RST "  ← past end: adjacent stack memory\n",  arr[3]);
    printf("  arr[9] = " RED BOLD "%10d" RST "  ← reading further into the stack\n",   arr[9]);

    section_explain();
    printf("  C performs " BOLD "no bounds checking, ever" RST ". arr[3] computes:\n");
    printf("    " CYN "&arr[0] + 3 * sizeof(int)" RST "\n");
    printf("  and dereferences whatever bytes live at that address.\n\n");
    printf("  " RED BOLD "No crash. No error. No signal." RST " Just garbage stack data.\n\n");
    printf("  In practice: exposes stack contents (locals, return addresses).\n");
    printf("  This is how " BOLD "information-disclosure vulnerabilities" RST " are born.\n");
}

/* ============================================================
 * Demo 3: Silent out-of-bounds write
 * ============================================================ */
static void demo_oob_write(void) {
    code_open();
    printf(CL YLW "struct" RST " { " YLW "int" RST " arr[3]; " YLW "int" RST " canary; } s =\n");
    printf(CL "    { .arr = {" MAG "1" RST "," MAG "2" RST "," MAG "3" RST "}, .canary = " GRN "0xDEAD" RST " };\n");
    printf(DIM "  │\n" RST);
    printf(CL "s.arr[" RED "3" RST "] = " RED "0xBEEF" RST ";  " GRY "// one past the end" RST "\n");
    code_close();

    printf("  arr[3] and canary are at the " BOLD "same memory address" RST ".\n");
    printf("  " DIM "(struct layout is guaranteed: arr ends at offset 12,\n");
    printf("   canary starts at offset 12 — so arr[3] == canary)" RST "\n");

    press_enter("What will canary's value be after the write? Will anything complain?");
    section_result();

    struct { int arr[3]; int canary; } s = { .arr = {1, 2, 3}, .canary = 0xDEAD };
    printf("  Before: arr=[%d,%d,%d]  canary=" GRN BOLD "0x%X" RST "\n",
           s.arr[0], s.arr[1], s.arr[2], s.canary);
    s.arr[3] = 0xBEEF;
    printf("  After:  arr=[%d,%d,%d]  canary=" RED BOLD "0x%X" RST "\n",
           s.arr[0], s.arr[1], s.arr[2], s.canary);

    section_explain();
    if (s.canary != 0xDEAD)
        printf("  " RED BOLD "The write to arr[3] silently overwrote canary." RST "\n");
    printf("  " RED BOLD "No error. No warning at runtime." RST " The write just happened.\n\n");
    printf("  In a real binary the bytes past an array could be:\n");
    printf("    " RED "•" RST " A saved return address  → " BOLD "classic stack-smashing exploit" RST "\n");
    printf("    " RED "•" RST " A local variable        → " BOLD "silent logic corruption" RST "\n");
    printf("    " RED "•" RST " A saved frame pointer   → " BOLD "crash on function return" RST "\n\n");
    printf("  Run " CYN "'make run-asan'" RST " to see AddressSanitizer catch this.\n");
}

/* ============================================================
 * Demo 4: The manual length convention
 * ============================================================ */
static int sum(const int *arr, size_t len) {
    int total = 0;
    for (size_t i = 0; i < len; i++) total += arr[i];
    return total;
}

static void demo_length_convention(void) {
    code_open();
    printf(CL YLW "int" RST " data[5] = {" MAG "1" RST "," MAG "2" RST "," MAG "3" RST "," MAG "4" RST "," MAG "5" RST "};  " GRY "// correct sum = 15" RST "\n");
    printf(DIM "  │\n" RST);
    printf(CL "sum(data, " GRN "5" RST ");   " GRY "// correct" RST "\n");
    printf(CL "sum(data, " YLW "6" RST ");   " GRY "// off by one" RST "\n");
    printf(CL "sum(data, " RED "50" RST ");  " GRY "// wildly wrong" RST "\n");
    code_close();

    printf("  'sum' has no way to know the true length of 'data'.\n");
    printf("  It trusts whatever " BOLD "'len'" RST " the caller provides.\n");

    press_enter("What will sum(data, 6) and sum(data, 50) return? An error? A wrong number?");
    section_result();

    int data[5] = {1, 2, 3, 4, 5};
    printf("  sum(data,  5) = " GRN BOLD "%8d" RST "  (correct)\n",                    sum(data, 5));
    printf("  sum(data,  6) = " YLW BOLD "%8d" RST "  (one garbage value included)\n", sum(data, 6));
    printf("  sum(data, 50) = " RED BOLD "%8d" RST "  (45 garbage values included)\n", sum(data, 50));

    section_explain();
    printf("  C " BOLD "cannot verify 'len'" RST ". The function loops 50 times reading\n");
    printf("  50 ints from data[0] — 45 of which are past the end of the\n");
    printf("  array, reading " RED "unrelated stack memory" RST " as if it were data.\n\n");
    printf("  Wrong 'len' is the root cause of nearly every " BOLD "buffer overflow" RST "\n");
    printf("  in C code. Caller and callee operate on the " RED "honor system" RST ".\n");
}

/* ============================================================ */

static void print_menu(void) {
    printf(RED BOLD "\n  ┌──────────────────────────────────────┐\n");
    printf("  │          C Array Horror              │\n");
    printf("  └──────────────────────────────────────┘\n" RST);
    printf(DIM "  Demonstrates silent undefined behavior in C arrays.\n\n" RST);
    printf("  " CYN "1)" RST " sizeof decay\n");
    printf("  " CYN "2)" RST " Silent out-of-bounds read\n");
    printf("  " CYN "3)" RST " Silent out-of-bounds write\n");
    printf("  " CYN "4)" RST " The manual length convention\n");
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
            case '1': demo_sizeof_decay();       break;
            case '2': demo_oob_read();           break;
            case '3': demo_oob_write();          break;
            case '4': demo_length_convention();  break;
            case 'q': case 'Q':
                printf(DIM "\n  Exiting. Segfaults not included.\n\n" RST);
                return 0;
            default:
                printf(RED "  Unknown option '%c'.\n" RST, ch);
        }
    }
    return 0;
}
