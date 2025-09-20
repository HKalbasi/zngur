#include <iostream>
#include <vector>

#include "./generated.h"

template <typename T> using Vec = rust::std::vec::Vec<T>;

int main() {
  Vec<int32_t> s = Vec<int32_t>::new_();
  s.push(2);
  s.push(7);
  zngur_dbg(s);
  s.reserve(3);
  int32_t* s_ptr = s.as_mut_ptr();
  s_ptr[2] = 3;
  s_ptr[3] = 10;
  s_ptr[4] = 2;
  s.set_len(5);
  zngur_dbg(s);

  Vec<Vec<int32_t>> ss = Vec<Vec<int32_t>>::new_();
  ss.push(s.clone());
  ss.push(Vec<int32_t>::new_());
  zngur_dbg(ss);
  ss.reserve(3);
  rust::RawMut<Vec<int32_t>> ss_ptr = ss.as_mut_ptr();
  ss.set_len(5);
  zngur_dbg(ss);
}
