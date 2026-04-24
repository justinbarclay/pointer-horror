use std::hint::black_box;
use std::io::{self, BufRead, Write};
use std::panic;

const RST:  &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM:  &str = "\x1b[2m";
const GRN:  &str = "\x1b[92m";
const YLW:  &str = "\x1b[93m";
const MAG:  &str = "\x1b[95m";
const CYN:  &str = "\x1b[96m";
const GRY:  &str = "\x1b[90m";
const RED:  &str = "\x1b[91m";

fn code_open() {
    println!("{DIM}  ╭─ {RST}{BOLD}{GRN}rust{RST} {DIM}────────────────────────────────────────────────────");
    println!("  │{RST}");
}

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
    println!("{BOLD}{YLW}  ║{RST}  {DIM}Press Enter to see what actually happens...{RST}");
    println!("{BOLD}{YLW}  ╚════════════════════════════════════════════════════════════╝{RST}");
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    let n = io::stdin().lock().read_line(&mut buf).unwrap_or(0);
    if n == 0 { std::process::exit(0); }
    println!();
}

fn cl() -> String {
    format!("  {DIM}│{RST}  ")
}

// ============================================================
// Demo 1: Slices carry length -- no sizeof decay
// ============================================================
fn demo_slice_carries_length() {
    let p = cl();
    code_open();
    println!("{p}{DIM}// &[i32] is a fat pointer: (data_ptr, length){RST}");
    println!("{p}{YLW}fn{RST} print_length(arr: {GRN}&[i32]{RST}) {{");
    println!("{p}    println!({CYN}\"arr.len = {{}}\"{RST}, arr.{YLW}len(){RST});");
    println!("{p}}}");
    println!("{DIM}  │{RST}");
    println!("{p}{YLW}let{RST} data = [{MAG}1i32{RST}, {MAG}2{RST}, {MAG}3{RST}, {MAG}4{RST}, {MAG}5{RST}, {MAG}6{RST}, {MAG}7{RST}, {MAG}8{RST}, {MAG}9{RST}, {MAG}10{RST}];");
    println!("{p}println!({CYN}\"data.len = {{}}\"{RST}, data.{YLW}len(){RST});  {GRY}// ← in main{RST}");
    println!("{p}print_length({GRN}&data{RST});                  {GRY}// ← inside function{RST}");
    code_close();

    press_enter("In C, sizeof decays to a pointer size inside functions. What does Rust do?");
    section_result();

    let data = [1i32, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("  data.len() in main            : {GRN}{BOLD}{}{RST}", data.len());
    print_length(&data);

    let partial = &data[2..5];
    println!("  &data[2..5].len()             : {GRN}{BOLD}{}{RST}", partial.len());
    print_length(partial);

    section_explain();
    println!("  {BOLD}&[i32]{RST} is a {BOLD}\"fat pointer\"{RST}: it carries {GRN}(data_ptr, length){RST} together.");
    println!("  Length is part of the type — it {BOLD}cannot be separated{RST} from the slice.");
    println!("  There is no decay. print_length(&data) receives the full length.\n");
    println!("  This is why Rust functions take {GRN}&[T]{RST} instead of {RED}*const T{RST}.");
    println!("  The C convention of passing {RED}(arr, len){RST} separately is unnecessary.");
}

fn print_length(arr: &[i32]) {
    println!("  arr.len() inside function     : {GRN}{BOLD}{}{RST}", arr.len());
}

// ============================================================
// Demo 2: Bounds-checked indexing -- loud failure, not silent
// ============================================================
fn demo_bounds_check_panic() {
    let p = cl();
    code_open();
    println!("{p}{YLW}let{RST} arr = [{MAG}10i32{RST}, {MAG}20{RST}, {MAG}30{RST}];");
    println!("{p}{YLW}let{RST} _ = arr[{RED}5{RST}];  {GRY}// index 5, array has 3 elements{RST}");
    code_close();

    println!("  {DIM}Note: Rust also catches literal out-of-bounds at {BOLD}compile time{RST}{DIM}.");
    println!("  This demo uses black_box to force a runtime bounds check.{RST}");

    press_enter("What happens when you index past the end of a Rust array?");
    section_result();

    let arr = [10i32, 20, 30];
    println!("  arr[0] = {GRN}{BOLD}{}{RST}  (valid)", arr[0]);
    println!("  arr[2] = {GRN}{BOLD}{}{RST}  (valid)", arr[2]);
    println!();

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let idx = black_box(5usize);
    let result = panic::catch_unwind(|| arr[idx]);
    panic::set_hook(prev_hook);

    match result {
        Ok(_) => unreachable!(),
        Err(payload) => {
            let msg = payload
                .downcast_ref::<String>()
                .map(|s| s.as_str())
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("index out of bounds");
            println!("  arr[5] → {RED}{BOLD}Panic:{RST} \"{msg}\"");
        }
    }

    section_explain();
    println!("  Rust checks bounds on {BOLD}every index operation{RST} at runtime.");
    println!("  When the check fails, it {GRN}panics immediately{RST} at the bad access:\n");
    println!("    {GRN}•{RST} Clear error message with actual vs expected length");
    println!("    {GRN}•{RST} Stack unwinds cleanly — {BOLD}no memory is corrupted{RST}");
    println!("    {GRN}•{RST} The failure is {BOLD}loud and localized{RST}, not silent and distant\n");
    println!("  Compare: in C, arr[5] {RED}silently reads garbage memory{RST}.");
    println!("  In Rust, the program {GRN}refuses to continue{RST} with invalid data.");
}

// ============================================================
// Demo 3: Safe access with .get() -> Option<&T>
// ============================================================
fn demo_safe_access() {
    let p = cl();
    code_open();
    println!("{p}{YLW}let{RST} arr = [{MAG}10i32{RST}, {MAG}20{RST}, {MAG}30{RST}];");
    println!("{DIM}  │{RST}");
    println!("{p}arr.{YLW}get({GRN}0{YLW}){RST}   {GRY}// in-bounds{RST}");
    println!("{p}arr.{YLW}get({RED}5{YLW}){RST}   {GRY}// out-of-bounds{RST}");
    println!("{p}arr.{YLW}get({RED}99{YLW}){RST}  {GRY}// way out of bounds{RST}");
    code_close();

    println!("  {DIM}.get() returns {GRN}Option<&T>{RST}{DIM} instead of panicking.{RST}");

    press_enter("What do you think arr.get(0) and arr.get(5) will return?");
    section_result();

    let arr = [10i32, 20, 30];
    for i in [0, 1, 2, 5, 99] {
        match arr.get(i) {
            Some(val) => println!("  arr.get({i:>2}) = {GRN}{BOLD}Some({val}){RST}"),
            None      => println!("  arr.get({i:>2}) = {RED}{BOLD}None{RST}"),
        }
    }

    section_explain();
    println!("  .get() returns {BOLD}Option<&T>{RST}:");
    println!("    {GRN}Some(&val){RST}  if the index is valid");
    println!("    {RED}None{RST}        if the index is out of bounds\n");
    println!("  The compiler {BOLD}rejects{RST} code that uses the value without handling None.");
    println!("  Out-of-bounds is not a crash — it is a {BOLD}value you must handle{RST}.\n");
    println!("  Use {YLW}[]{RST} when you are certain the index is valid (accepts panic risk).");
    println!("  Use {GRN}.get(){RST} when the index comes from untrusted or computed input.");
}

// ============================================================
// Demo 4: Length is part of the type -- no convention needed
// ============================================================
fn demo_length_in_type() {
    let p = cl();
    code_open();
    println!("{p}{DIM}// no separate 'len' parameter{RST}");
    println!("{p}{YLW}fn{RST} sum(arr: {GRN}&[i32]{RST}) -> {YLW}i32{RST} {{");
    println!("{p}    arr.{YLW}iter(){RST}.{CYN}sum(){RST}");
    println!("{p}}}");
    println!("{DIM}  │{RST}");
    println!("{p}{YLW}let{RST} data = [{MAG}1i32{RST}, {MAG}2{RST}, {MAG}3{RST}, {MAG}4{RST}, {MAG}5{RST}];");
    println!("{p}sum({GRN}&data{RST})");
    println!("{p}sum({GRN}&data[0..3]{RST})");
    println!("{p}sum({GRN}&data[1..]{RST})");
    code_close();

    press_enter("No 'len' argument. How does sum() know the length? Can the caller lie about it?");
    section_result();

    fn sum(arr: &[i32]) -> i32 { arr.iter().sum() }
    let data = [1i32, 2, 3, 4, 5];
    println!("  sum(&data)        = {GRN}{BOLD}{}{RST}", sum(&data));
    println!("  sum(&data[0..3])  = {GRN}{BOLD}{}{RST}", sum(&data[0..3]));
    println!("  sum(&data[1..])   = {GRN}{BOLD}{}{RST}", sum(&data[1..]));

    section_explain();
    println!("  {GRN}&[i32]{RST} encodes {BOLD}(ptr, len) as a fat pointer{RST}.");
    println!("  Length is part of the value — there is {BOLD}no separate argument to get wrong{RST}.\n");
    println!("  The compiler ensures slices are always created from valid memory.");
    println!("  {YLW}data[0..100]{RST} on a 5-element array {RED}panics at slice creation{RST},");
    println!("  before the bad slice can ever reach sum().\n");
    println!("  The C pattern of {RED}(ptr, len) held together by convention{RST}");
    println!("  is replaced by a type that makes the invariant {GRN}unbreakable{RST}.");
}

fn print_menu() {
    println!("{GRN}{BOLD}\n  ┌──────────────────────────────────────┐");
    println!("  │         Rust Array Safety            │");
    println!("  └──────────────────────────────────────┘{RST}");
    println!("{DIM}  Demonstrates how Rust eliminates C's array pitfalls.\n{RST}");
    println!("  {CYN}1){RST} Slices carry length — no decay");
    println!("  {CYN}2){RST} Bounds-checked indexing (panic)");
    println!("  {CYN}3){RST} Safe access with .get() -> Option");
    println!("  {CYN}4){RST} Length is part of the type");
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
            "1" => demo_slice_carries_length(),
            "2" => demo_bounds_check_panic(),
            "3" => demo_safe_access(),
            "4" => demo_length_in_type(),
            "q" | "Q" => {
                println!("{DIM}\n  Exiting. No undefined behavior was invoked.\n{RST}");
                break;
            }
            _ => println!("{RED}  Unknown option.{RST}"),
        }
    }
}
