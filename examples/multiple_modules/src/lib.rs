pub struct MainType(pub i64);

impl MainType {
    pub fn do_something(&self, _v: std::option::Option<&i32>) {
        println!("do_something called!");
    }
}

#[rustfmt::skip]
#[path = "main.zng.rs"]
mod main_generated;
#[rustfmt::skip]
#[path = "module.zng.rs"]
mod module_generated;
