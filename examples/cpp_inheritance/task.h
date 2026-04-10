#pragma once
#include <cstdint>

namespace task {

enum class Poll {
  kPending,
  kReady,
};

class Task {
public:
  virtual ~Task() {}
  virtual Poll poll() = 0;
};

} // namespace task

using Task = task::Task;
