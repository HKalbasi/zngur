#include <cstdint>
#include <iostream>
#include <vector>

#include "./generated.h"

using namespace std;

template <typename T> using Vec = ::rust::std::vec::Vec<T>;

int main() {
  auto s = Vec<int32_t>::new_();
  s.push(2);
  Vec<int32_t>::push(s, 5);
  s.push(7);
  Vec<int32_t>::push(s, 3);
  cout << s.clone().into_iter().sum() << endl;
  int state = 0;
  auto f = ::rust::Box<::rust::Dyn<::rust::Fn<int32_t, int32_t>>>::build(
      [&](int32_t x) {
        state += x;
        std::cout << "hello " << x << " " << state << "\n";
        return x * 2;
      });
  auto x = s.into_iter().map(std::move(f)).sum();
  std::cout << x << " " << state << "\n";
}

template <>
class ::rust::Impl<::std::vector<int32_t>>
    : ::rust::std::iter::Iterator<int32_t>::Impl {

public:
  ::std::vector<int32_t> self;

  ::rust::std::option::Option<int32_t> next() {
    if (self.empty()) {
      return ::rust::std::option::Option<int32_t>::None();
    } else {
      auto r = self.back();
      self.pop_back();
      return ::rust::std::option::Option<int32_t>::Some(r);
    }
  }
};
