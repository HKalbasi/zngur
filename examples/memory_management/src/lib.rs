mod generated;

#[derive(Clone)]
pub struct PrintOnDrop(&'static str);

pub struct PrintOnDropPair {
    pub first: PrintOnDrop,
    pub second: PrintOnDrop,
}

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

fn consume_and_panic(p: PrintOnDrop, do_panic: bool) -> PrintOnDrop {
    if do_panic {
        panic!("consume_and_panic executed with value {}", p.0);
    }
    p
}
