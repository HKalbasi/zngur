#![allow(non_camel_case_types)]

mod generated;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use example_cpp_inheritance::CppInherit;
pub use generated::cpp::CppTask;
pub use generated::cpp::Dispatcher;

pub struct RustTask<'a>(pub Pin<&'a mut dyn Future<Output = ()>>);

impl<'a> RustTask<'a> {
    pub fn poll(&mut self) -> Poll<()> {
        let waker = unsafe { Waker::from_raw(dummy_raw_waker()) };
        let mut cx = Context::from_waker(&waker);
        self.0.as_mut().poll(&mut cx)
    }
}

unsafe fn dummy_raw_waker() -> std::task::RawWaker {
    static DUMMY_WAKER_VTABLE: std::task::RawWakerVTable =
        std::task::RawWakerVTable::new(|_| unsafe { dummy_raw_waker() }, |_| (), |_| (), |_| ());
    std::task::RawWaker::new(std::ptr::null(), &DUMMY_WAKER_VTABLE)
}

fn main() {
    let mut fut = async {
        println!("Hello from Rust async!");
    };

    let fut = RustTask(unsafe { Pin::new_unchecked(&mut fut) });
    let mut task: CppInherit<CppTask, RustTask> = CppInherit::new_with(fut, |base: &mut CppTask| base.init());
    let dispatcher = Dispatcher::new();
    dispatcher.run_task(&mut task.inner);
}
