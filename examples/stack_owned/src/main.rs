#[rustfmt::skip]
mod generated;

pub use generated::cpp::MyCppWrapper;

fn main() {
    let c = MyCppWrapper::new(5, 6);
    c.print();
}
