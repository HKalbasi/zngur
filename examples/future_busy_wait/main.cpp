#include <iostream>
#include "./generated.h"

using rust::crate::block_on;
using rust::crate::pend_x;

template <typename T>
using BoxFuture = rust::Box<rust::Dyn<rust::std::future::Future<T>>>;

BoxFuture<int32_t> cpp_coro_ready_5() {
  std::cout << "cpp_coro_ready_5" << std::endl;
  co_return 5;
}

BoxFuture<int32_t> cpp_coro_pend_3x(uint64_t ms) {
  std::cout << "cpp_coro_pend_3x " << ms << std::endl;
  auto x = co_await pend_x(ms);
  std::cout << "first done" << std::endl;
  x += co_await pend_x(ms);
  std::cout << "second done" << std::endl;
  x += co_await pend_x(ms);
  std::cout << "third done" << std::endl;
  co_return x;
}

int main() {
  std::cout << "=== test ready futures ===" << std::endl;
  auto f1 = cpp_coro_ready_5();
  auto f2 = cpp_coro_pend_3x(3);
  std::cout << "Futures are lazy and do nothing until polled" << std::endl;
  
  block_on(std::move(f1));
  block_on(std::move(f2));
}
