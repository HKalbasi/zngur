use std::{
    fmt::Debug,
    task::{Context, Waker},
};

#[rustfmt::skip]
mod generated;

fn argument_position_impl_trait(arg: impl Debug) {
    println!("Print debug -- {arg:?}");
}

fn return_position_impl_trait() -> impl Debug {
    4
}

fn both_impl_trait(input: impl Debug) -> impl Debug {
    input
}

async fn async_func1() -> i32 {
    println!("Async func 1");
    43
}

fn busy_wait_future<T: Debug>(fut: Box<dyn Future<Output = T>>) -> T {
    let mut fut = Box::into_pin(fut);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(r) => {
                println!("Future done with result {r:?}");
                return r;
            }
            std::task::Poll::Pending => (),
        }
    }
}
