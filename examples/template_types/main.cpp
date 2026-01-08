#include <cstddef>

#include "generated.h"


int main() {
  auto vec_a = rust::std::vec::Vec<rust::crate::TypeA>::new_();
  vec_a.push(rust::crate::TypeA(1));
  vec_a.push(rust::crate::TypeA(2));
  vec_a.push(rust::crate::TypeA(3));
  // vec_a.get(0); // Does not exist because [TypeA] is not defined in main.zng
  zngur_dbg(vec_a);

  auto vec_b = rust::std::vec::Vec<rust::crate::TypeB>::new_();
  vec_b.push(rust::crate::TypeB(1));
  vec_b.push(rust::crate::TypeB(2));
  vec_b.push(rust::crate::TypeB(3));
  for (std::size_t i = 0; i < vec_b.len(); i++) {
    zngur_dbg(vec_b.get(i).unwrap());
  }

  auto box = rust::crate::get_box();
  box.as_ref().say_hello();
}
