#include <iostream>
#include "./generated.h"

using rust::crate::new_current_runtime;

template <typename T>
using BoxFuture = rust::Box<rust::Dyn<rust::std::future::Future<T>, rust::Send, rust::Sync>>;

BoxFuture<int32_t> cpp_coro_ready_5() {
  co_return 5;
}

BoxFuture<uint64_t> cpp_sleep(uint64_t ms) {
  auto dur = rust::std::time::Duration::from_millis(ms);
  co_await rust::tokio::time::sleep(std::move(dur));
  std::cout << "tokio_sleep " << ms << " done" << std::endl;
  co_return ms;
}

BoxFuture<int32_t> cpp_coro_sleep_3x(uint64_t ms) {
  auto x = co_await cpp_sleep(ms);
  x += co_await cpp_sleep(ms);
  x += co_await cpp_sleep(ms);
  co_return x;
}

BoxFuture<rust::Unit> join_tasks(rust::tokio::runtime::Runtime& runtime) {
  auto h1 = runtime.spawn(cpp_coro_sleep_3x(60).into_pin());
  auto h2 = runtime.spawn(cpp_coro_sleep_3x(100).into_pin());
  std::cout << "spawned 2 tasks" << std::endl;
  auto r1 = co_await rust::crate::into_box_future(std::move(h1));
  auto r2 = co_await rust::crate::into_box_future(std::move(h2));
  std::cout << "joined " << r1.unwrap() << " " << r2.unwrap() << std::endl;
  co_return rust::Unit{};
}

int main() {
  std::cout << "=== test ready futures ===" << std::endl;
  auto f1 = cpp_coro_ready_5();
  auto f2 = cpp_coro_sleep_3x(5);
  std::cout << "Futures are lazy and do nothing until polled" << std::endl;
  
  auto runtime = new_current_runtime();
  std::cout << "f1 = " << runtime.block_on(f1.into_pin()) << std::endl;
  std::cout << "f2 = " << runtime.block_on(f2.into_pin()) << std::endl;

  runtime.block_on(join_tasks(runtime).into_pin());
}
