#![allow(non_camel_case_types)]

mod generated;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use generated::CppInherit;
use generated::Task;

pub struct RustTask {
    pub fut: *mut dyn Future<Output = ()>,
}

impl RustTask {
    pub fn poll(&mut self) -> Poll<()> {
        let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
        let mut cx = Context::from_waker(&waker);
        let fut = unsafe { &mut *self.fut };
        let pinned_fut = unsafe { Pin::new_unchecked(fut) };
        pinned_fut.poll(&mut cx)
    }
}

unsafe fn dummy_raw_waker() -> std::task::RawWaker {
    std::task::RawWaker::new(
        std::ptr::null(),
        &std::task::RawWakerVTable::new(|_| unsafe { dummy_raw_waker() }, |_| (), |_| (), |_| ()),
    )
}

pub fn test_inheritance() {
    let mut fut = async {
        println!("Hello from Rust async!");
    };
    let fut_ptr = &mut fut as *mut dyn Future<Output = ()>;
    let rust_obj = RustTask { fut: fut_ptr };
    let inherited = CppInherit::<Task, RustTask>::new(rust_obj);
    let cpp_ptr = inherited.as_cpp_ptr();
    generated::poll(cpp_ptr);
}

fn main() {
    test_inheritance();
}
