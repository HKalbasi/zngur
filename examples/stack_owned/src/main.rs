#[rustfmt::skip]
mod generated;

use generated::std::unique_ptr;
use generated::{CppConservativeType, CppRefType, CppType};

fn main() {
    let c = CppType::new(5, 6);
    c.print();

    assert_eq!(std::mem::size_of::<CppConservativeType>(), 32);
    assert_eq!(std::mem::align_of::<CppConservativeType>(), 16);

    let mut vec = Vec::new();
    let cons = CppConservativeType::new(10, 20);
    vec.push(cons);
    vec[0].print();

    let r = CppRefType::get_global();
    unsafe {
        (*r).print();
    }

    assert_eq!(std::mem::size_of::<unique_ptr<CppType>>(), 8);
    assert_eq!(std::mem::align_of::<unique_ptr<CppType>>(), 8);
    let p = unique_ptr::<CppType>::make_unique(1, 2);
    p.print();
}
