pub struct Packet(pub u64, pub u32);

impl Packet {
    pub fn new(timestamp: u64, size: u32) -> Self {
        Packet(timestamp, size)
    }

    pub fn timestamp(&self) -> u64 {
        self.0
    }

    pub fn size(&self) -> u32 {
        self.1
    }
}

#[rustfmt::skip]
#[path = "packet.zng.rs"]
mod generated;
