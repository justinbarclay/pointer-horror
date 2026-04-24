// Demonstrates: Rust prevents using a value after it has been dropped.
//
// Try:      rustc use_after_free.rs
// Expected: error[E0382] borrow of moved value: `s`

fn main() {
    let s = String::from("hello");
    drop(s);           // explicit free — s is moved into drop()
    println!("{s}");   // use after free
}
