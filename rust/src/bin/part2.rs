use std::io::{self, BufRead, Write};

const RST:  &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM:  &str = "\x1b[2m";
const GRN:  &str = "\x1b[92m";
const YLW:  &str = "\x1b[93m";
const MAG:  &str = "\x1b[95m";
const CYN:  &str = "\x1b[96m";
const GRY:  &str = "\x1b[90m";
const RED:  &str = "\x1b[91m";

fn code_open(lang: &str, color: &str) {
    println!("{DIM}  ╭─ {RST}{BOLD}{color}{lang}{RST} {DIM}────────────────────────────────────────────────────");
    println!("  │{RST}");
}

fn code_open_c()    { code_open("c",    RED); }
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

fn section_compiler() {
    println!("{BOLD}{YLW}\n  ━━ Compiler says {RST}{DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RST}\n");
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

fn p() -> String { format!("  {DIM}│{RST}  ") }

// ============================================================
// Demo 1: No dangling — lifetimes prevent returning &local
// ============================================================
fn demo_no_dangling() {
    let p = p();
    println!("  In C this compiles fine and produces a dangling pointer:\n");

    code_open_c();
    println!("{p}{YLW}int{RST} *bad_get() {{");
    println!("{p}    {YLW}int{RST} local = {MAG}42{RST};");
    println!("{p}    {RED}return &local;{RST}  {GRY}// stack frame destroyed on return{RST}");
    println!("{p}}}");
    code_close();

    println!("  Now the equivalent Rust — note the borrow {RED}('&'){RST} on the return type:\n");

    code_open_rust();
    println!("{p}{YLW}fn{RST} bad_get() -> {RED}&i32{RST} {{   {GRY}// ← missing lifetime specifier{RST}");
    println!("{p}    {YLW}let{RST} local = {MAG}42{RST};");
    println!("{p}    {RED}&local{RST}             {GRY}// ← cannot return reference to local{RST}");
    println!("{p}}}");
    code_close();

    section_compiler();
    println!("  {RED}error[E0106]{RST}: missing lifetime specifier");
    println!("   {GRY}-->{RST} src/bin/part2.rs:4:17");
    println!("    {GRY}|{RST}");
    println!("  {GRY}4 |{RST} fn bad_get() -> {RED}&i32{RST} {{");
    println!("    {GRY}|{RST}                 {RED}^^^^{RST} expected named lifetime parameter");
    println!();
    println!("  {RED}error[E0515]{RST}: cannot return reference to local variable `local`");
    println!("    {GRY}|{RST}");
    println!("  {GRY}6 |{RST}     {RED}&local{RST}");
    println!("    {GRY}|{RST}     {RED}^^^^^^{RST} returns a reference to data owned by the current function");
    println!("\n  {DIM}Try it yourself:{RST}  {CYN}rustc rust/wont-compile/dangling.rs{RST}");

    press_enter("The C code compiles and creates a dangling pointer. Rust won't compile it. What do you do instead?");
    section_result();

    println!("  Return the {GRN}value itself{RST} (ownership transfers to the caller):\n");

    code_open_rust();
    println!("{p}{YLW}fn{RST} get_value() -> {GRN}i32{RST} {{    {GRY}// owned — no lifetime needed{RST}");
    println!("{p}    {YLW}let{RST} local = {MAG}42{RST};");
    println!("{p}    local                  {GRY}// move: caller now owns it{RST}");
    println!("{p}}}");
    println!("{DIM}  │{RST}");
    println!("{p}{YLW}let{RST} v = get_value();");
    println!("{p}println!({CYN}\"{{v}}\"{RST});   {GRY}// safe: v is an i32 on our stack{RST}");
    code_close();

    let v = {
        let local = 42;
        local  // value copied/moved out
    };
    println!("  v = {GRN}{BOLD}{v}{RST}  (caller owns this i32 — no pointer, no dangling)");

    section_explain();
    println!("  {BOLD}Lifetimes{RST} are Rust's compile-time proof that references never");
    println!("  outlive the data they point to. The rule:");
    println!("    {GRN}•{RST} A reference {GRN}(&T){RST} can only be returned if it borrows from");
    println!("      something the {BOLD}caller already owns{RST}");
    println!("    {RED}•{RST} A reference to a {RED}local variable{RST} cannot escape the function\n");
    println!("  The solution: return an {GRN}owned value{RST} ({YLW}i32{RST}, {YLW}String{RST}, {YLW}Box<T>{RST}).");
    println!("  The concept of a dangling pointer does not exist in safe Rust.\n");
    println!("  Every {GRN}&T{RST} reference is {BOLD}guaranteed valid{RST} by the borrow checker.");
}

// ============================================================
// Demo 2: Drop runs exactly once — no use-after-free
// ============================================================
struct Tracked(i32);

impl Drop for Tracked {
    fn drop(&mut self) {
        println!("  {DIM}[Drop] Tracked({}) freed — memory returned to OS{RST}", self.0);
    }
}

fn demo_drop_once() {
    let p = p();
    println!("  The C equivalent: malloc, free, then read the freed pointer:\n");

    code_open_c();
    println!("{p}{YLW}int{RST} *p = {CYN}malloc{RST}({YLW}sizeof{RST}({YLW}int{RST}));");
    println!("{p}*p = {MAG}42{RST};");
    println!("{p}{CYN}free{RST}(p);");
    println!("{p}{CYN}printf{RST}({GRN}\"%%d\\n\"{RST}, *p); {RED}// ← use-after-free, UB{RST}");
    code_close();

    println!("  In Rust, attempting this is a compile error:\n");

    code_open_rust();
    println!("{p}{YLW}let{RST} s = {GRN}String{RST}::from({GRN}\"hello\"{RST});");
    println!("{p}{YLW}drop{RST}(s);           {GRY}// explicit drop = free{RST}");
    println!("{p}println!({CYN}\"{{s}}\"{RST}); {RED}// error: use of moved value: `s`{RST}");
    code_close();

    section_compiler();
    println!("  {RED}error[E0382]{RST}: borrow of moved value: `s`");
    println!("   {GRY}-->{RST} src/bin/part2.rs");
    println!("    {GRY}|{RST}");
    println!("    {GRY}|{RST}  {YLW}let{RST} s = String::from(\"hello\");");
    println!("    {GRY}|{RST}      {GRY}- move occurs because `s` has type `String`{RST}");
    println!("    {GRY}|{RST}  {YLW}drop{RST}(s);");
    println!("    {GRY}|{RST}         {GRY}- value moved here{RST}");
    println!("    {GRY}|{RST}  println!(\"{{s}}\");");
    println!("    {GRY}|{RST}           {RED}^^^ value borrowed here after move{RST}");
    println!("\n  {DIM}Try it yourself:{RST}  {CYN}rustc rust/wont-compile/use_after_free.rs{RST}");

    press_enter("Rust won't compile use-after-drop. What does 'drop' actually look like at runtime?");
    section_result();

    println!("  Creating a Tracked value with a custom Drop implementation:\n");
    {
        let t = Tracked(42);
        println!("  t.0 = {GRN}{BOLD}{}{RST}  (alive)", t.0);
        println!("  Explicitly dropping t...");
        drop(t);
        println!("  (t is gone — compiler won't let us mention it now)");
    }

    section_explain();
    println!("  {BOLD}Ownership{RST} means every value has exactly one owner.");
    println!("  When the owner goes out of scope (or is passed to drop()),");
    println!("  Rust calls {GRN}Drop::drop(){RST} and the memory is freed.\n");
    println!("  After drop(), the variable is {RED}moved{RST} — the compiler tracks");
    println!("  this statically and refuses to compile any code that accesses");
    println!("  it. This makes use-after-free {BOLD}impossible to write{RST} in safe Rust.\n");
    println!("  No garbage collector. No runtime checks. Pure {GRN}compile-time{RST} proof.");
}

// ============================================================
// Demo 3: Ownership = single owner = single free
// ============================================================
fn demo_single_owner() {
    let p = p();
    println!("  The C double-free: free the same pointer twice:\n");

    code_open_c();
    println!("{p}{YLW}int{RST} *p = {CYN}malloc{RST}({YLW}sizeof{RST}({YLW}int{RST}));");
    println!("{p}*p = {MAG}42{RST};");
    println!("{p}{CYN}free{RST}(p);   {GRY}// correct first free{RST}");
    println!("{p}{CYN}free{RST}(p);   {RED}// double-free: UB / heap corruption{RST}");
    code_close();

    println!("  In Rust, the ownership rules make this impossible to write:\n");

    code_open_rust();
    println!("{p}{YLW}let{RST} a = {GRN}Box{RST}::new({MAG}42{RST});  {GRY}// a owns heap allocation{RST}");
    println!("{p}{YLW}let{RST} b = a;           {GRY}// ownership MOVES to b{RST}");
    println!("{p}println!({CYN}\"{{a}}\"{RST});   {RED}// error: a was moved{RST}");
    println!("{p}{GRY}// b dropped here — freed exactly once{RST}");
    code_close();

    section_compiler();
    println!("  {RED}error[E0382]{RST}: use of moved value: `a`");
    println!("    {GRY}|{RST}");
    println!("    {GRY}|{RST}  {YLW}let{RST} a = Box::new(42);");
    println!("    {GRY}|{RST}      {GRY}- move occurs: `a` has type `Box<i32>`{RST}");
    println!("    {GRY}|{RST}  {YLW}let{RST} b = a;");
    println!("    {GRY}|{RST}          {GRY}- value moved here{RST}");
    println!("    {GRY}|{RST}  println!(\"{{a}}\");");
    println!("    {GRY}|{RST}           {RED}^ value used here after move{RST}");
    println!("\n  {DIM}Try it yourself:{RST}  {CYN}rustc rust/wont-compile/double_free.rs{RST}");

    press_enter("Single ownership prevents double-free. Show me what safe single-owner access actually looks like.");
    section_result();

    println!("  Ownership transfer and scoped drop:\n");
    {
        let a = Tracked(100);
        println!("  [a owns Tracked(100)]");
        let b = a;  // a is moved into b
        println!("  [moved a → b, a no longer exists]");
        println!("  b.0 = {GRN}{BOLD}{}{RST}", b.0);
        println!("  [b goes out of scope...]");
    }
    println!("  [scope ended — b was freed exactly once, above]\n");

    section_explain();
    println!("  {BOLD}Move semantics{RST}: assignment transfers ownership.");
    println!("  After {YLW}let b = a{RST}, {RED}a is gone{RST} from the compiler's perspective.");
    println!("  There is {BOLD}one owner{RST} at all times → {BOLD}one drop{RST} at all times.\n");
    println!("  This means:");
    println!("    {GRN}•{RST} Memory is freed when the owner goes out of scope");
    println!("    {GRN}•{RST} No GC needed — the scope boundary is the free point");
    println!("    {GRN}•{RST} Double-free is a type error, not a runtime crash");
    println!("    {GRN}•{RST} Memory leaks are prevented without runtime overhead");
}

// ============================================================
// Demo 4: Option<T> — the compile-enforced nullable type
// ============================================================
fn find_value(arr: &[i32], target: i32) -> Option<i32> {
    arr.iter().find(|&&x| x == target).copied()
}

fn demo_option() {
    let p = p();
    println!("  In C, find_value returns NULL when not found — type is just int *:\n");

    code_open_c();
    println!("{p}{YLW}int{RST} *find_value({YLW}int{RST} *arr, {YLW}size_t{RST} len, {YLW}int{RST} target) {{");
    println!("{p}    {YLW}for{RST} (...) {YLW}if{RST} (arr[i] == target) {YLW}return{RST} &arr[i];");
    println!("{p}    {YLW}return{RST} {RED}NULL{RST};");
    println!("{p}}}");
    println!("{DIM}  │{RST}");
    println!("{p}{YLW}int{RST} *p = find_value(arr, n, {RED}99{RST}); {GRY}// NULL — 99 not found{RST}");
    println!("{p}{CYN}printf{RST}({GRN}\"%%d\\n\"{RST}, *p);            {RED}// ← forgot NULL check: SIGSEGV{RST}");
    code_close();

    println!("  In Rust, the return type is {GRN}Option<i32>{RST} — the absent case is in the type:\n");

    code_open_rust();
    println!("{p}{YLW}fn{RST} find_value(arr: {GRN}&[i32]{RST}, target: {YLW}i32{RST}) -> {GRN}Option<i32>{RST} {{");
    println!("{p}    arr.{YLW}iter(){RST}.{CYN}find{RST}(|&&x| x == target).{YLW}copied{RST}()");
    println!("{p}}}");
    println!("{DIM}  │{RST}");
    println!("{p}{GRY}// Compiler forces you to handle None:{RST}");
    println!("{p}{YLW}match{RST} find_value(&arr, {RED}99{RST}) {{");
    println!("{p}    {GRN}Some(v){RST} => println!({CYN}\"found: {{v}}\"{RST}),");
    println!("{p}    {RED}None{RST}    => println!({CYN}\"not found\"{RST}),");
    println!("{p}}}");
    code_close();

    press_enter("What happens when Rust code tries to use an Option<i32> without handling None?");
    section_result();

    let arr = [1i32, 2, 3, 4, 5];

    for target in [3, 99] {
        let result = find_value(&arr, target);
        match result {
            Some(v) => println!("  find_value(&arr, {target:>2}) = {GRN}{BOLD}Some({v}){RST}"),
            None    => println!("  find_value(&arr, {target:>2}) = {RED}{BOLD}None{RST}   (not found — no crash)"),
        }
    }

    println!();
    println!("  Trying to use result without matching is a compile error:");
    println!("  {RED}error[E0308]{RST}: mismatched types");
    println!("    expected {YLW}i32{RST}, found {GRN}Option<i32>{RST}\n");
    println!("  Short-circuit helpers:");
    let found   = find_value(&arr, 3).unwrap_or(0);
    let missing = find_value(&arr, 99).unwrap_or(0);
    println!("  find_value(&arr, 3).unwrap_or(0)  = {GRN}{BOLD}{found}{RST}");
    println!("  find_value(&arr, 99).unwrap_or(0) = {YLW}{BOLD}{missing}{RST}  (default, not a crash)");

    section_explain();
    println!("  {GRN}Option<T>{RST} is an enum: {GRN}Some(value){RST} or {RED}None{RST}.");
    println!("  There is no NULL in Rust — there is no way to have an {YLW}i32{RST}");
    println!("  that secretly might be null. The absent case is {BOLD}explicit in the type{RST}.\n");
    println!("  If you try to use an {GRN}Option<i32>{RST} as an {YLW}i32{RST} without unwrapping,");
    println!("  the {BOLD}compiler refuses{RST}. You cannot forget the None check.\n");
    println!("    {GRN}match{RST}          — exhaustive, must handle both arms");
    println!("    {GRN}.unwrap_or(x){RST}  — provide a fallback value");
    println!("    {GRN}.map(|v| ...){RST}  — transform if Some, propagate None");
    println!("    {GRN}if let Some(v){RST} — branch on presence");
    println!("    {GRN}?{RST}              — propagate None to the caller");
}

fn print_menu() {
    println!("{GRN}{BOLD}\n  ┌──────────────────────────────────────┐");
    println!("  │     Rust Pointer Lifetime Safety     │");
    println!("  └──────────────────────────────────────┘{RST}");
    println!("{DIM}  Part 2: lifetimes, ownership, drop, Option<T>.\n{RST}");
    println!("  {CYN}1){RST} No dangling — lifetimes prevent &local");
    println!("  {CYN}2){RST} No use-after-free — borrow checker + Drop");
    println!("  {CYN}3){RST} No double-free — single owner, single drop");
    println!("  {CYN}4){RST} No null — Option<T> forces None handling");
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
            "1" => demo_no_dangling(),
            "2" => demo_drop_once(),
            "3" => demo_single_owner(),
            "4" => demo_option(),
            "q" | "Q" => {
                println!("{DIM}\n  Exiting. All memory already freed by the borrow checker.\n{RST}");
                break;
            }
            _ => println!("{RED}  Unknown option.{RST}"),
        }
    }
}
