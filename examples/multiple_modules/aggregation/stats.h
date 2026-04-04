#pragma once
#include <iostream>
#include <numeric>
#include <vector>

// We include packet.zng.h to use Packet methods
#include <packet.zng.h>

namespace cpp_stats {

class StatsAccumulator {
  std::vector<uint64_t> timestamps;
  std::vector<uint32_t> sizes;

public:
  StatsAccumulator() = default;

  void add_packet(const rust::packet::Packet &p) {
    timestamps.push_back(p.timestamp());
    sizes.push_back(p.size());
  }

  void print_report() const {
    uint64_t total_size = std::accumulate(sizes.begin(), sizes.end(), 0ULL);
    double latency_avg = 0;
    for (int i = 1; i < timestamps.size(); ++i) {
      latency_avg += static_cast<double>(timestamps[i] - timestamps[i - 1]) /
                     (timestamps.size() - 1);
    }

    std::cout << "LatencyAnalysis Report:" << std::endl;
    std::cout << "Total Packets: " << timestamps.size() << std::endl;
    std::cout << "Total Bytes: " << total_size << std::endl;
    if (!timestamps.empty()) {
      std::cout << "Average Latency: " << latency_avg << std::endl;
    }
  }
};

} // namespace cpp_stats
