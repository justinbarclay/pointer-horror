// Demonstrates: Rust prevents double-free via ownership.
// Moving a value invalidates the original binding — there is always
// exactly one owner, so there is always exactly one drop.
//
// Try:      rustc double_free.rs
// Expected: error[E0382] use of moved value: `a`

fn main() {
    let a = Box::new(42);
    let b = a;         // ownership moves to b — a is now invalid
    println!("{a}");   // error: a was moved
    drop(b);           // b is freed exactly once
}
