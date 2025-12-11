#include <iostream>
#include <vector>

#include "./generated.h"

using DynDebug = rust::Dyn<rust::std::fmt::Debug>;

int main() {
  rust::crate::argument_position_impl_trait(5);
  rust::crate::argument_position_impl_trait("hello"_rs.to_owned());
  rust::crate::argument_position_impl_trait(
      rust::crate::return_position_impl_trait());
  rust::Box<DynDebug> elem = rust::crate::both_impl_trait("foo"_rs);
  rust::crate::argument_position_impl_trait(elem.deref());
  rust::crate::argument_position_impl_trait(std::move(elem));

  auto future = rust::crate::async_func1();
  rust::crate::argument_position_impl_trait(
      "Rust futures are lazy"_rs.to_owned());
  rust::crate::busy_wait_future(std::move(future));
  rust::crate::busy_wait_future(rust::crate::async_func2());

  rust::crate::argument_position_impl_trait(
      "Before calling impl_future"_rs.to_owned());
  future = rust::crate::impl_future();
  rust::crate::argument_position_impl_trait(
      "Before polling impl_future"_rs.to_owned());
  rust::crate::busy_wait_future(std::move(future));
}
