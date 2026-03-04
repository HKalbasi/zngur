#![allow(unused)]

#[rustfmt::skip]
mod generated;

#[derive(Debug)]
struct TypeA(i32);

#[derive(Debug)]
struct TypeB(i32);

trait MyTrait {
    fn say_hello(&self) {
        println!("Hello");
    }
}

impl MyTrait for TypeA {}

fn get_box() -> Box<dyn MyTrait> {
    Box::new(TypeA(1))
}
