mod generated;

#[derive(Clone)]
struct PrintOnDrop(&'static str);

impl Drop for PrintOnDrop {
    fn drop(&mut self) {
        println!("PrintOnDrop({}) has been dropped", self.0);
    }
}

trait PrintOnDropConsumer {
    fn consume(&mut self, p: PrintOnDrop);
}

fn consume_n_times(consumer: &mut dyn PrintOnDropConsumer, name: &'static str, times: usize) {
    for _ in 0..times {
        consumer.consume(PrintOnDrop(name));
    }
}
