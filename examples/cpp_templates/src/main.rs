#[rustfmt::skip]
mod generated;

use generated::std::{shared_ptr, vector};
use generated::{CppType, Holder};

// Plain data: two ints, no `Drop`, so C++ may memcpy it freely.
struct RustPoint(i32, i32);

fn main() {
    assert_eq!(std::mem::size_of::<vector<i32>>(), 24);
    assert_eq!(std::mem::size_of::<shared_ptr<vector<i32>>>(), 16);

    let mut v = vector::<i32>::make_numbers();
    println!("Numbers sum {}", v.sum());
    v.push(4);
    println!("Numbers sum {}", v.sum());

    let mut s = shared_ptr::<vector<i32>>::make_shared_numbers();
    println!("Shared sum {}", s.sum());
    s.push(30);
    println!("Shared sum {}", s.sum());

    let mut h = Holder::<CppType>::make_holder(1, 2);
    println!("Holder first {}", h.first());
    h.set_first(10);
    println!("Holder first {}", h.first());

    let d = Holder::<i32>::default_holder();
    unsafe {
        println!("Default holder {}", (*d).value());
    }

    // Mixed: a C++ template holding a Rust value.
    let mut rh = Holder::<RustPoint>::make_rust_holder(3, 4);
    let inner = rh.take_inner();
    assert_eq!((inner.0, inner.1), (3, 4));
    println!("Rust holder {} {}", inner.0, inner.1);

    // Mixed: a Rust `Vec` holding C++ instantiations. The `.zng`
    // declares the instantiation so the method shims resolve; the calls
    // below are the real `Vec` methods, moving and dropping the heap
    // boxes like any other element.
    let mut hv: Vec<Holder<CppType>> = Vec::new();
    hv.push(Holder::<CppType>::make_holder(5, 6));
    hv.push(Holder::<CppType>::make_holder(7, 8));
    assert_eq!(hv.len(), 2);
    println!("Holder vec len {}", hv.len());

    // Mixed the other way: a C++ `vector` owning Rust `Box`es. The boxes
    // are real `Box`es (no mirror: Rust types cross as themselves),
    // constructed on the Rust side and moved across by value.
    let mut vb = vector::<Box<RustPoint>>::make_boxed_points();
    vb.push_box(Box::new(RustPoint(1, 2)));
    vb.push_box(Box::new(RustPoint(3, 4)));
    assert_eq!(vb.len(), 2);
    println!("Boxed vec len {}", vb.len());
    let first = vb.take_first();
    assert_eq!((first.0, first.1), (1, 2));
    println!("First boxed {} {}", first.0, first.1);
}
