#include <iostream>

#include <aggregation.zng.h>
#include <packet.zng.h>
#include <processor.zng.h>
#include <receiver.zng.h>

int main() {
  auto processor = rust::std::option::Option<rust::crate::Processor>::Some(
      rust::crate::Processor::new_());
  auto receiver = rust::std::option::Option<rust::crate::Receiver>::Some(
      rust::crate::Receiver::new_());
  auto stats =
      rust::Impl<rust::crate::StatsAccumulator, rust::Inherent>::create();

  std::cout << "Starting LatencyAnalysis simulation..." << std::endl;

  processor.unwrap().run(receiver.unwrap(), stats, 5);

  rust::Impl<rust::crate::StatsAccumulator, rust::Inherent>::print_report(
      stats);

  return 0;
}
