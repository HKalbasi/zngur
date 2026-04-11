#[rustfmt::skip]
mod generated;

pub use generated::cpp::MyCppWrapper;

fn main() {
    println!("Hello from Rust");
    let c = generated::create_cpp_type(10, 20);
    println!("Rust got CppType");
    generated::print_cpp_type(&c);
    println!("Rust dropping CppType");
}
