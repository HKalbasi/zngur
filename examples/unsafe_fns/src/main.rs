#[rustfmt::skip]
mod generated;

pub struct RustStruct;

impl RustStruct {
    pub unsafe fn unsafe_rust_fn() {
        println!("Called unsafe Rust function!");
    }

    // To test that signature verification works:
    // 1. Comment out the safe definition below.
    // 2. Uncomment the unsafe definition below.
    // 3. Run `cargo build` and verify it fails because it is marked as safe in .zng.

    pub fn safe_rust_fn() {
        println!("Called safe Rust function!");
    }

    // pub unsafe fn safe_rust_fn() {
    //     println!("Called unsafe Rust function!");
    // }
}

fn main() {
    let result = unsafe { generated::dangerous_function() };
    println!("Dangerous result: {}", result);
}
