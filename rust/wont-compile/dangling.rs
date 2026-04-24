// Demonstrates: Rust prevents returning references to local variables.
//
// Try:      rustc dangling.rs
// Expected: error[E0106] missing lifetime specifier
//           error[E0515] cannot return reference to local variable `local`

fn bad_get() -> &i32 {
    let local = 42;
    &local
}

fn main() {
    let p = bad_get();
    println!("{p}");
}
