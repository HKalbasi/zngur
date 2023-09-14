mod generated;

#[derive(Clone)]
struct PrintOnDrop(&'static str);

impl Drop for PrintOnDrop {
    fn drop(&mut self) {
        println!("PrintOnDrop({}) has been dropped", self.0);
    }
}
