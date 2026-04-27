#[rustfmt::skip]
mod generated;

fn main() {
    let result = unsafe { generated::dangerous_function() };
    println!("Dangerous result: {}", result);
}
