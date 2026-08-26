use std::{
    future::Future,
    sync::atomic::AtomicU32,
    task::{Context, Poll, Waker},
};

#[rustfmt::skip]
mod generated;

pub fn into_box_future<T>(
    f: impl Future<Output = T> + Send + Sync + 'static,
) -> Box<dyn Future<Output = T> + Send + Sync> {
    Box::new(f)
}

struct PendingCounter(u64, u64);

impl Future for PendingCounter {
    type Output = u64;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 == self.1 {
            Poll::Ready(self.0)
        } else {
            cx.waker().clone().wake();
            self.1 += 1;
            Poll::Pending
        }
    }
}

pub fn pend_x(x: u64) -> impl Future<Output = u64> {
    PendingCounter(x, 0)
}

pub fn block_on<T>(f: Box<dyn Future<Output = T>>) -> T {
    static JOB_ID: AtomicU32 = AtomicU32::new(0);
    let id = JOB_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut f = Box::into_pin(f);
    let mut f = f.as_mut();
    let mut cx = Context::from_waker(Waker::noop());
    println!("job {id} started");
    loop {
        match f.as_mut().poll(&mut cx) {
            Poll::Ready(t) => return t,
            Poll::Pending => println!("job {id} is pending"),
        }
    }
}
