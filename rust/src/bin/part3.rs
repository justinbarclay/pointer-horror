use std::hint::black_box;
use std::io::{self, BufRead, Write};
use std::rc::Rc;
use std::time::Instant;

const RST:  &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM:  &str = "\x1b[2m";
const GRN:  &str = "\x1b[92m";
const YLW:  &str = "\x1b[93m";
const BLU:  &str = "\x1b[94m";
const MAG:  &str = "\x1b[95m";
const CYN:  &str = "\x1b[96m";
const GRY:  &str = "\x1b[90m";
const RED:  &str = "\x1b[91m";

fn code_open(lang: &str, color: &str) {
    println!("{DIM}  ╭─ {RST}{BOLD}{color}{lang}{RST} {DIM}────────────────────────────────────────────────────");
    println!("  │{RST}");
}
fn code_open_rust() { code_open("rust", GRN); }
fn code_close() {
    println!("{DIM}  │");
    println!("  ╰──────────────────────────────────────────────────────────{RST}");
    println!();
}
fn section_result() {
    println!("{BOLD}{CYN}\n  ━━ Result {RST}{DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RST}\n");
}
fn section_explain() {
    println!("{BOLD}{MAG}\n  ━━ What happened {RST}{DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RST}\n");
}
fn press_enter(question: &str) {
    println!("{BOLD}{YLW}\n  ╔═ ? ════════════════════════════════════════════════════════╗{RST}");
    println!("{BOLD}{YLW}  ║{RST}  {question}");
    println!("{BOLD}{YLW}  ║{RST}");
    println!("{BOLD}{YLW}  ║{RST}  {DIM}Press Enter to continue...{RST}");
    println!("{BOLD}{YLW}  ╚════════════════════════════════════════════════════════════╝{RST}");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    let n = io::stdin().lock().read_line(&mut buf).unwrap_or(0);
    if n == 0 { std::process::exit(0); }
    println!();
}
fn p() -> String { format!("  {DIM}│{RST}  ") }

// ══════════════════════════════════════════════════════════════════
// Demo 1 — shared ownership (verbosity)
// ══════════════════════════════════════════════════════════════════
fn demo_shared_ownership() {
    let p = p();

    println!("  {GRN}Rc<T>{RST} is Rust's reference-counted smart pointer —");
    println!("  the same mechanism as C's manual refcount, generated for you:\n");

    code_open_rust();
    println!("{p}{YLW}use{RST} std::rc::{GRN}Rc{RST};");
    println!("{DIM}  │{RST}");
    println!("{p}{YLW}let{RST} a = {GRN}Rc{RST}::new({MAG}42{RST});          {GRY}// alloc + refs=1{RST}");
    println!("{p}{YLW}let{RST} b = {GRN}Rc{RST}::clone(&a);      {GRY}// refs=2  (retain){RST}");
    println!("{p}println!(\"{{a}}, count={{}}\", {GRN}Rc{RST}::strong_count(&a));");
    println!("{p}{GRY}// a and b drop here — freed exactly once{RST}");
    code_close();

    press_enter("4 lines vs ~15 in C. Does it behave identically at runtime?");
    section_result();

    let a = Rc::new(42i32);
    let b = Rc::clone(&a);
    println!("  a={GRN}{BOLD}{a}{RST}  b={GRN}{BOLD}{b}{RST}  strong_count={GRN}{BOLD}{}{RST}",
             Rc::strong_count(&a));
    drop(b);
    println!("  Dropped b → count={YLW}{}{RST}", Rc::strong_count(&a));
    drop(a);
    println!("  Dropped a → count=0 — freed\n");

    println!("  {BOLD}Lines of boilerplate to safely share one heap value:\n{RST}");
    println!("  {RED}C (manual refcount){RST}  ~15 lines  {RED}███████████████{RST}");
    println!("  {GRN}Rust (Rc<T>){RST}          4 lines  {GRN}████{RST}");
    println!("  {BLU}TypeScript (GC){RST}        0 lines  {DIM}(GC tracks references invisibly){RST}");

    section_explain();
    println!("  {GRN}Rc<T>{RST} wraps your value with a heap-allocated reference counter.");
    println!("  {GRN}Rc::clone(){RST} increments it ({YLW}retain{RST}); {GRN}drop(){RST} decrements it ({YLW}release{RST}).");
    println!("  When count hits 0, {GRN}Drop::drop(){RST} is called — exactly once.\n");
    println!("  You write 4 lines. Rust writes the other 11 for you.");
    println!("  The {YLW}trade-off{RST}: sharing is {BOLD}explicit{RST} in the type ({GRN}Rc<T>{RST} vs {YLW}T{RST}).");
    println!("  {BLU}TypeScript{RST}: zero lines, but {YLW}when{RST} memory frees is not your decision.");
}

// ══════════════════════════════════════════════════════════════════
// Demo 2 — allocation latency benchmark
// ══════════════════════════════════════════════════════════════════
const BATCH_SIZE:  usize = 1000;
const BATCH_COUNT: usize = 500;

struct Node { a: i32, b: i32, c: i32, d: i32, e: i32, f: i32 }

fn demo_alloc_latency() {
    let p = p();

    println!("  Each batch: {YLW}{BATCH_SIZE}{RST} {GRN}Box::new(){RST} allocations, then drop.");
    println!("  Total: {YLW}500,000{RST} allocations across {YLW}{BATCH_COUNT}{RST} batches.\n");
    println!("  {YLW}For accurate numbers run with:{RST}  {DIM}cargo run --release --bin part3{RST}\n");

    code_open_rust();
    println!("{p}{YLW}let mut{RST} nodes: {GRN}Vec<Box<Node>>{RST} = {GRN}Vec{RST}::with_capacity(BATCH_SIZE);");
    println!("{p}{YLW}for{RST} i {YLW}in{RST} 0..BATCH_SIZE {{");
    println!("{p}    nodes.{CYN}push{RST}({GRN}Box{RST}::new({GRN}Node{RST} {{ a: i, {GRY}/* ... */{RST} }}));");
    println!("{p}}}");
    println!("{p}{GRY}// nodes drops here — all freed deterministically{RST}");
    code_close();

    press_enter("Box::new/drop uses the same allocator as C malloc/free. Will numbers match?");
    section_result();

    let mut times: Vec<u128> = Vec::with_capacity(BATCH_COUNT);
    let mut checksum: i64 = 0;

    for b in 0..BATCH_COUNT {
        let filled = (b + 1) * 38 / BATCH_COUNT;
        print!("\r  {DIM}[{RST}");
        for x in 0..38usize {
            print!("{}", if x < filled { format!("{GRN}▓{RST}") } else { format!("{DIM}░{RST}") });
        }
        print!("{DIM}]{RST}  {}/{BATCH_COUNT}   ", b + 1);
        io::stdout().flush().unwrap();

        let t0 = Instant::now();
        let mut nodes: Vec<Box<Node>> = Vec::with_capacity(BATCH_SIZE);
        for i in 0..BATCH_SIZE {
            let i = i as i32;
            nodes.push(black_box(Box::new(Node {
                a: i, b: i*2, c: i*3, d: i*4, e: i*5, f: i*6,
            })));
        }
        for n in &nodes { checksum += n.a as i64; }
        drop(nodes); /* deterministic free */
        times.push(t0.elapsed().as_nanos() / 1000); /* µs */
    }
    println!("\r  {GRN}Complete.{RST}                                              \n");

    times.sort_unstable();
    let p50  = times[BATCH_COUNT / 2];
    let p95  = times[BATCH_COUNT * 95 / 100];
    let tmax = *times.last().unwrap();
    let sc   = tmax.max(1);

    let bar = |n: u128| "█".repeat((n * 30 / sc) as usize);
    println!("  {BOLD}Batch time ({BATCH_COUNT} batches × {BATCH_SIZE} allocs):{RST}\n");
    println!("   p50  {GRN}{BOLD}{p50:>5}{RST}µs  {GRN}{}{RST}", bar(p50));
    println!("   p95  {YLW}{BOLD}{p95:>5}{RST}µs  {YLW}{}{RST}", bar(p95));
    println!("   max  {RED}{BOLD}{tmax:>5}{RST}µs  {RED}{}{RST}", bar(tmax));
    println!("\n  {DIM}(checksum={checksum} — prevents dead-code elimination){RST}");

    section_explain();
    println!("  {GRN}drop(){RST} is {BOLD}deterministic{RST} — it runs the instant nodes goes out of scope.");
    println!("  Rust uses the system allocator (same as C). Numbers should be close.\n");
    println!("  Compare with the TypeScript CLI part 3:");
    println!("    {YLW}If TS p95 >> p50{RST}: you're seeing a GC pause.");
    println!("    {GRN}If TS p95 ≈ p50{RST}:  JSC collected during your keypress gaps.");
    println!("                  Run non-interactively to remove that slack.\n");
    println!("  Neither C nor Rust can produce a GC spike — there is no GC.");
}

// ─── menu ────────────────────────────────────────────────────────────
fn print_menu() {
    println!("{GRN}{BOLD}\n  ┌──────────────────────────────────────┐");
    println!("  │   Rust — Performance & Verbosity     │");
    println!("  └──────────────────────────────────────┘{RST}");
    println!("{DIM}  Part 3: memory management cost in code and latency.\n{RST}");
    println!("  {CYN}1){RST} Shared ownership — verbosity comparison");
    println!("  {CYN}2){RST} Allocation latency — deterministic free vs GC");
    println!("  {DIM}q) Quit{RST}");
    print!("\n  {BOLD}> {RST}");
    io::stdout().flush().unwrap();
}

fn main() {
    let stdin = io::stdin();
    loop {
        print_menu();
        let mut line = String::new();
        let n = stdin.lock().read_line(&mut line).unwrap_or(0);
        if n == 0 { break; }
        println!();
        match line.trim() {
            "1" => demo_shared_ownership(),
            "2" => demo_alloc_latency(),
            "q" | "Q" => {
                println!("{DIM}\n  Exiting. All memory freed deterministically.\n{RST}");
                break;
            }
            _ => println!("{RED}  Unknown option.{RST}"),
        }
    }
}
