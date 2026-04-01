#[rustfmt::skip]
#[path = "aggregation.zng.rs"]
mod generated;

pub use packet_crate::Packet;

pub struct StatsAccumulator(pub generated::ZngurCppOpaqueOwnedObject);
