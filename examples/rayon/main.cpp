#include <cstddef>
#include <cstdint>
#include <iostream>
#include <numeric>
#include <vector>

#include "./generated.h"

uint32_t is_prime(uint64_t v) {
  if (v < 2)
    return 0;
  for (int i = 2; i * i <= v; i += 1) {
    if (v % i == 0) {
      return 0;
    }
  }
  return 1;
}

int main() {
  std::vector<uint64_t> v(10000000);
  std::iota(v.begin(), v.end(), 1);
  auto slice = ::std::slice::from_raw_parts(v.data(), v.size());
  auto f = ::rust::Box<
      ::rust::Dyn<::rust::Fn<::rust::Ref<uint64_t>, int32_t>, ::rust::Sync,
                  ::rust::Send>>::build([&](::rust::Ref<uint64_t> x) {
    return is_prime(*x);
  });
  std::cout << slice.par_iter().map(std::move(f)).sum() << std::endl;
}
