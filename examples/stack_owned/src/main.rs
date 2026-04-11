#[rustfmt::skip]
mod generated;

pub use generated::cpp::MyCppWrapper;
use znglib::ZngCppStackObject;

fn main() {
    let c = MyCppWrapper::new();
    println!("Default constructed CppType:");
    c.print();

    let c = MyCppWrapper::custom_constructor(5, 6);
    c.print();
}
