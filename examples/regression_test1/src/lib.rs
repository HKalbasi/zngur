#[rustfmt::skip]
mod generated;

#[allow(unused)]
#[derive(Debug)]
struct Foo {
    field1: i32,
    field2: String,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone)]
struct FieldTypeA {
    pub fizz: FieldTypeC,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone)]
// heap allocated
struct FieldTypeB {
    pub fizz: FieldTypeC,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone)]
// auto field offset
struct FieldTypeC {
    pub buzz_1: i32,
    pub buzz_2: i32,
    pub buzz_3: i32,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone)]
struct TypeA {
    pub foo: i32,
    pub bar: FieldTypeA,
    pub baz: FieldTypeB,
}

#[allow(unused)]
#[derive(Debug, Copy, Clone)]
// heap allocated
struct TypeB {
    pub foo: i32,
    pub bar: FieldTypeA,
    pub baz: FieldTypeB,
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
