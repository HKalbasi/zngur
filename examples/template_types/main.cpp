#include <cstddef>

#include "generated.h"


int main() {
  auto vec_a = rust::std::vec::Vec<rust::crate::TypeA>::new_();
  vec_a.push(rust::crate::TypeA(1));
  vec_a.push(rust::crate::TypeA(2));
  vec_a.push(rust::crate::TypeA(3));
  zngur_dbg(vec_a);

  auto vec_b = rust::std::vec::Vec<rust::crate::TypeB>::new_();
  vec_b.push(rust::crate::TypeB(1));
  vec_b.push(rust::crate::TypeB(2));
  vec_b.push(rust::crate::TypeB(3));
  for (std::size_t i = 0; i < vec_b.len(); i++) {
    auto opt = vec_b.get(i);
    if (auto r = rust::std::option::Option<rust::Ref<rust::crate::TypeB>>::Some::match(opt)) {
        zngur_dbg(*rust::Ref(r->f0));
    }
  }

  auto box = rust::crate::get_box();
  box.as_ref().say_hello();
}
