#[rustfmt::skip]
mod generated;

pub use generated::cpp::MyCppWrapper;
use znglib::{ZngCppDefaultConstruct, ZngCppStackObject};

// SAFETY: Constructor calls the actual C++ constructor. Object is initialized
// after calling `.construct()`
unsafe impl ZngCppDefaultConstruct for MyCppWrapper {
    unsafe fn construct(&mut self) {
        self.constructor();
    }
}

fn main() {
    let c = MyCppWrapper::new();
    println!("Default constructed CppType:");
    c.print();

    let c = MyCppWrapper::custom_constructor(5, 6);
    c.print();
}
