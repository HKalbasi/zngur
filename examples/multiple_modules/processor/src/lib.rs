pub use receiver_crate::Receiver;
pub use aggregation_crate::StatsAccumulator;

pub struct Processor;

impl Processor {
    pub fn new() -> Self {
        Processor
    }

    pub fn run(
        &self,
        receiver: &mut receiver_crate::Receiver,
        stats: &mut aggregation_crate::StatsAccumulator,
        count: u32,
    ) {
        for _ in 0..count {
            let packet = receiver.next_packet();
            stats.add_packet(packet);
        }
    }
}

#[rustfmt::skip]
#[path = "processor.zng.rs"]
mod generated;
