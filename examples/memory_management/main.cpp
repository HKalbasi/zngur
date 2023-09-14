#include <iostream>
#include <vector>

#include "./generated.h"

// Rust values are available in the `::rust` namespace from their absolute path
// in Rust
template <typename T> using Vec = ::rust::std::vec::Vec<T>;
template <typename T> using Option = ::rust::std::option::Option<T>;
template <typename T> using BoxDyn = ::rust::Box<::rust::Dyn<T>>;
using rust::crate::PrintOnDrop;

// You can implement Rust traits for your classes
template <typename T>
class VectorIterator : public rust::Impl<::rust::std::iter::Iterator<T>> {
  std::vector<T> vec;
  size_t pos;

public:
  VectorIterator(std::vector<T> &&v) : vec(v), pos(0) {}
  ~VectorIterator() {
    std::cout << "vector iterator has been destructed" << std::endl;
  }

  Option<T> next() override {
    if (pos >= vec.size()) {
      return Option<T>::None();
    }
    T value = vec[pos++];
    // You can construct Rust enum with fields in C++
    return Option<T>::Some(value);
  }
};

int main() {
  auto p1 = PrintOnDrop(::rust::Str::from_char_star("A"));
  auto p2 = PrintOnDrop(::rust::Str::from_char_star("B"));
  auto p3 = PrintOnDrop(::rust::Str::from_char_star("C"));
  std::cout << "Checkpoint 1" << std::endl;
  p2 = std::move(p1);
  std::cout << "Checkpoint 2" << std::endl;
  {
    std::cout << "Checkpoint 3" << std::endl;
    PrintOnDrop p{std::move(p2)};
    std::cout << "Checkpoint 4" << std::endl;
  }
  {
    std::vector<PrintOnDrop> vec1;
    vec1.emplace_back(::rust::Str::from_char_star("cpp_V1"));
    vec1.emplace_back(::rust::Str::from_char_star("cpp_V2"));
    vec1.emplace_back(::rust::Str::from_char_star("cpp_V3"));
    std::cout << "Checkpoint 5" << std::endl;
    vec1.pop_back();
    std::cout << "Checkpoint 6" << std::endl;
  }
  {
    std::cout << "Checkpoint 7" << std::endl;
    Vec<PrintOnDrop> vec2 = Vec<PrintOnDrop>::new_();
    vec2.push(PrintOnDrop(::rust::Str::from_char_star("rust_V1")));
    vec2.push(PrintOnDrop(::rust::Str::from_char_star("rust_V2")));
    std::cout << "Checkpoint 8" << std::endl;
    vec2.clone(); // Clone and drop immediately
    std::cout << "Checkpoint 9" << std::endl;
    std::cout << "Checkpoint 10" << std::endl;
  }
  std::cout << "Checkpoint 11" << std::endl;
  std::cout << "Checkpoint 12" << std::endl;
  std::cout << "Checkpoint 13" << std::endl;
}
