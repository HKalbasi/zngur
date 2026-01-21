#[rustfmt::skip]
mod generated;

#[allow(unused)]
#[derive(Debug)]
struct Foo {
    field1: i32,
    field2: String,
}

struct Scoped(&'static str);

impl Scoped {
    fn new(message: &'static str) -> Self {
        println!("{message} -- started");
        Self(message)
    }
}

impl Drop for Scoped {
    fn drop(&mut self) {
        println!("{} -- finished", self.0);
        println!();
    }
}

fn call_dyn_fn_multi_args(func: Box<dyn Fn(i32, crate::Scoped, &str)>) {
    let scope = Scoped::new("scope passed to dyn Fn");
    func(2, scope, "hello");
    println!("End of call_dyn_fn_multi_args");
}

#[derive(Debug)]
struct ZeroSizedType;

impl ZeroSizedType {
    fn new() -> Self {
        Self
    }

    fn method(&self) {
        println!("Method call on ZST");
    }
}
