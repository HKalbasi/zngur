pub use packet_crate::Packet;

pub struct Receiver {
    count: u64,
}

impl Receiver {
    pub fn new() -> Self {
        Receiver { count: 0 }
    }

    pub fn next_packet(&mut self) -> Packet {
        self.count += 1;
        // Simulate packet: timestamp = count * 100, size = count * 10
        Packet::new(self.count * 100, (self.count * 10) as u32)
    }
}

#[rustfmt::skip]
#[path = "receiver.zng.rs"]
mod generated;
