use std::future::Future;

#[rustfmt::skip]
mod generated;

pub fn new_current_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

pub fn into_box_future<T>(
    f: impl Future<Output = T> + Send + Sync + 'static,
) -> Box<dyn Future<Output = T> + Send + Sync> {
    Box::new(f)
}
