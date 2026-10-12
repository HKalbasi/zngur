#include <aggregation.zng.h>
#include <packet.zng.h>
#include <stats.h>

// Method bodies for the real C++ type. There is no rust:: mirror wrapper:
// `Impl` is specialized directly on cpp_stats::StatsAccumulator.
cpp_stats::StatsAccumulator
rust::Impl<cpp_stats::StatsAccumulator>::create() {
  return cpp_stats::StatsAccumulator();
}

rust::Unit rust::Impl<cpp_stats::StatsAccumulator>::add_packet(
    rust::RefMut<cpp_stats::StatsAccumulator> self, rust::packet::Packet p) {
  self.cpp().add_packet(p);
  return {};
}

rust::Unit rust::Impl<cpp_stats::StatsAccumulator>::print_report(
    rust::Ref<cpp_stats::StatsAccumulator> self) {
  self.cpp().print_report();
  return {};
}
