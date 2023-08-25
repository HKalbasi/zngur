#include <cstddef>
#include <cstdint>
#include <iostream>
#include <vector>

#include "./generated.h"

using namespace std;

template <typename T> using Vec = ::rust::std::vec::Vec<T>;
template <typename T> using Option = ::rust::std::option::Option<T>;
template <typename T> using BoxDyn = ::rust::Box<::rust::Dyn<T>>;

template <typename T>
class VectorIterator : public rust::Impl<::rust::std::iter::Iterator<T>> {
  std::vector<T> vec;
  size_t pos;

public:
  VectorIterator(std::vector<T> &&v) : vec(v), pos(0) {}

  Option<T> next() override {
    if (pos >= vec.size()) {
      return Option<T>::None();
    }
    T value = vec[pos++];
    return Option<T>::Some(value);
  }
};

int main() {
  auto s = Vec<int32_t>::new_();
  s.push(2);
  Vec<int32_t>::push(s, 5);
  s.push(7);
  Vec<int32_t>::push(s, 3);
  cout << s.clone().into_iter().sum() << endl;
  int state = 0;
  auto f = BoxDyn<::rust::Fn<int32_t, int32_t>>::build([&](int32_t x) {
    state += x;
    std::cout << "hello " << x << " " << state << "\n";
    return x * 2;
  });
  auto x = s.into_iter().map(std::move(f)).sum();
  std::cout << x << " " << state << "\n";
  std::vector<int32_t> vec{10, 20, 60};
  auto vec_as_iter = BoxDyn<::rust::std::iter::Iterator<int32_t>>::make_box<
      VectorIterator<int32_t>>(std::move(vec));
  auto t = ::crate::collect_vec(std::move(vec_as_iter));
  zngur_dbg(t);
}
