use std::fmt::Debug;

#[rustfmt::skip]
mod generated;

fn argument_position_impl_trait(arg: impl Debug) {
    println!("Print debug -- {arg:?}");
}

fn return_position_impl_trait() -> impl Debug {
    4
}

