#[rustfmt::skip]
#[path = "aggregation.zng.rs"]
mod generated;

pub use packet::Packet;

pub struct StatsAccumulator(pub generated::ZngurCppOpaqueOwnedObject);
