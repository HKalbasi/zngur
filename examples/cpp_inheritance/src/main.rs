#![allow(non_camel_case_types)]

#[rustfmt::skip]
mod generated;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use cpp_inherit::CppInherit;
use zngur_lib::{ZngCppDefaultConstruct, ZngCppStackObject as _};

use crate::generated::cpp::{CppTask, Dispatcher};

pub struct RustTask<'a>(pub Pin<&'a mut dyn Future<Output = ()>>);

// SAFETY: C++ object will be initialized after calling .construct().
unsafe impl ZngCppDefaultConstruct for CppTask {
    unsafe fn construct(&mut self) {
        self.constructor();
    }
}

// SAFETY: C++ object will be initialized after calling .construct().
unsafe impl ZngCppDefaultConstruct for Dispatcher {
    unsafe fn construct(&mut self) {
        // SAFETY: Object is uninitialized at the time we call the constructor.
        self.constructor();
    }
}

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
    let mut task: CppInherit<CppTask, RustTask> = CppInherit::new(fut, CppTask::new());
    let dispatcher = Dispatcher::new();
    dispatcher.run_task(&mut task.inner);
}
