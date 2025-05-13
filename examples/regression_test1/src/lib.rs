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
