#[rustfmt::skip]
mod generated;

pub use generated::cpp::MyCppWrapper;
use zngur_lib::{ZngCppDefaultConstruct, ZngCppStackObject};

// SAFETY: Constructor calls the actual C++ constructor. Object is initialized
// after calling `.construct()`
unsafe impl ZngCppDefaultConstruct for MyCppWrapper {
    unsafe fn construct(this: &mut std::mem::MaybeUninit<Self>) {
        // SAFETY: It's fine to assume-init a C++ type that hasn't been constructed
        // yet for the purpose of calling the constructor.
        unsafe {
            this.assume_init_mut().constructor();
        }
    }
}

fn main() {
    let c = MyCppWrapper::new();
    println!("Default constructed CppType:");
    c.print();

    let c = MyCppWrapper::custom_constructor(5, 6);
    c.print();
}
