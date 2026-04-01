#include <iostream>
#include <packet.zng.h>
#include <receiver.zng.h>
#include <aggregation.zng.h>
#include <processor.zng.h>

int main() {
    // Using default rust::crate namespace
    auto processor = rust::crate::Processor::new_();
    auto receiver = rust::crate::Receiver::new_();
    auto stats = rust::Impl<rust::crate::StatsAccumulator, rust::Inherent>::create();

    std::cout << "Starting LatencyAnalysis simulation..." << std::endl;

    processor.run(receiver, stats, 5);

    rust::Impl<rust::crate::StatsAccumulator, rust::Inherent>::print_report(stats);

    return 0;
}
