#include "generated.h"

#include "impls.h"

namespace rust {

::task::Poll Impl<crate::RustTask, ::task::Task>::poll() {
  // Call Rust poll using the new downcast_ref abstraction
  auto rust_poll = ::rust::crate::RustTask::poll(this->downcast_ref_mut());

  if (rust_poll.matches_Ready()) {
    return ::task::Poll::kReady;
  } else {
    return ::task::Poll::kPending;
  }
}

namespace exported_functions {
::rust::Unit poll(::rust::Raw<::rust::Task> t) {
  // Use the new upcast abstraction
  auto *task = t.upcast<::task::Task>();
  task->poll();
  return {};
}
} // namespace exported_functions

} // namespace rust
