#pragma once

#include "task.h"

namespace rust {

namespace crate {
class RustTask;
}

struct Task;

template <typename Type, typename Trait> class Impl;

template <> class Impl<crate::RustTask, ::task::Task> : public ::rust::ImplBase<crate::RustTask, ::task::Task> {
public:
  ::task::Poll poll() override;
};

template <>
inline ::task::Task *upcast<::task::Task, ::rust::Task>(Raw<::rust::Task> r) {
  return reinterpret_cast<::task::Task *>(r.__zngur_data);
}

} // namespace rust
