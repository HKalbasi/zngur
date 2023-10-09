#include <iostream>
#include <vector>

#include "./generated.h"

// Rust values are available in the `::rust` namespace from their absolute path
// in Rust
template <typename T> using Vec = rust::std::vec::Vec<T>;
template <typename T> using Option = rust::std::option::Option<T>;
template <typename T> using BoxDyn = rust::Box<rust::Dyn<T>>;
template <typename T> using RmDyn = rust::RefMut<rust::Dyn<T>>;
using rust::crate::consume_and_panic;
using rust::crate::consume_n_times;
using rust::crate::PrintOnDrop;
using rust::crate::PrintOnDropConsumer;
using rust::crate::PrintOnDropPair;

class CppPrintOnDropHolder : public PrintOnDropConsumer {
  rust::Unit consume(PrintOnDrop p) override {
    item = std::move(p);
    return {};
  }

  PrintOnDrop item;
};

int main() {
  auto p1 = PrintOnDrop(rust::Str::from_char_star("A"));
  auto p2 = PrintOnDrop(rust::Str::from_char_star("B"));
  auto p3 = PrintOnDrop(rust::Str::from_char_star("C"));
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
    vec1.emplace_back(rust::Str::from_char_star("cpp_V1"));
    vec1.emplace_back(rust::Str::from_char_star("cpp_V2"));
    vec1.emplace_back(rust::Str::from_char_star("cpp_V3"));
    std::cout << "Checkpoint 5" << std::endl;
    vec1.pop_back();
    std::cout << "Checkpoint 6" << std::endl;
  }
  {
    std::cout << "Checkpoint 7" << std::endl;
    Vec<PrintOnDrop> vec2 = Vec<PrintOnDrop>::new_();
    vec2.push(PrintOnDrop(rust::Str::from_char_star("rust_V1")));
    vec2.push(PrintOnDrop(rust::Str::from_char_star("rust_V2")));
    std::cout << "Checkpoint 8" << std::endl;
    vec2.clone(); // Clone and drop immediately
    std::cout << "Checkpoint 9" << std::endl;
  }
  {
    CppPrintOnDropHolder c;
    {
      std::cout << "Checkpoint 10" << std::endl;
      auto holder = BoxDyn<PrintOnDropConsumer>::make_box<CppPrintOnDropHolder>(
          std::move(c));
      std::cout << "Checkpoint 11" << std::endl;
      consume_n_times(holder.deref_mut(), rust::Str::from_char_star("P"), 3);
      std::cout << "Checkpoint 12" << std::endl;
      consume_n_times(holder.deref_mut(), rust::Str::from_char_star("Q"), 2);
      std::cout << "Checkpoint 13" << std::endl;
    }
    std::cout << "Checkpoint 14" << std::endl;
  }
  {
    CppPrintOnDropHolder c;
    {
      std::cout << "Checkpoint 15" << std::endl;
      auto holder = RmDyn<PrintOnDropConsumer>(c);
      std::cout << "Checkpoint 16" << std::endl;
      consume_n_times(holder, rust::Str::from_char_star("P2"), 3);
      std::cout << "Checkpoint 17" << std::endl;
      consume_n_times(holder, rust::Str::from_char_star("Q2"), 2);
      std::cout << "Checkpoint 18" << std::endl;
    }
    std::cout << "Checkpoint 19" << std::endl;
  }
  std::cout << "Checkpoint 20" << std::endl;
  try {
    PrintOnDrop a{rust::Str::from_char_star("A")};
    std::cout << "Checkpoint 21" << std::endl;
    consume_and_panic(a.clone(), false);
    std::cout << "Checkpoint 22" << std::endl;
    consume_and_panic(rust::Str::from_char_star("B"), true);
    std::cout << "Checkpoint 23" << std::endl;
  } catch (rust::Panic e) {
    std::cout << "Checkpoint 24" << std::endl;
  }
  {
    std::cout << "Checkpoint 25" << std::endl;
    PrintOnDropPair p{rust::Str::from_char_star("first"),
                      rust::Str::from_char_star("second")};
    std::cout << "Checkpoint 26" << std::endl;
  }
  {
    std::cout << "Checkpoint 27" << std::endl;
    Vec<PrintOnDrop> vec2 = Vec<PrintOnDrop>::new_();
    vec2.push(PrintOnDrop(rust::Str::from_char_star("elem1")));
    vec2.push(PrintOnDrop(rust::Str::from_char_star("elem2")));
    std::cout << "Checkpoint 28" << std::endl;
    vec2.get(0).unwrap().clone();
    {
      auto vec_slice = vec2.deref();
      auto tmp = vec_slice.get(0).unwrap().clone();
      std::cout << "Checkpoint 29" << std::endl;
    }
    std::cout << "Checkpoint 30" << std::endl;
  }
  std::cout << "Checkpoint 31" << std::endl;
  {
    auto p1 = zngur_dbg(PrintOnDrop(rust::Str::from_char_star("dbg_A")));
    std::cout << "Checkpoint 32" << std::endl;
    auto p2 = zngur_dbg(std::move(p1));
    std::cout << "Checkpoint 33" << std::endl;
    zngur_dbg(p2);
    std::cout << "Checkpoint 34" << std::endl;
  }
  std::cout << "Checkpoint 35" << std::endl;
  {
    auto p1 = Option<PrintOnDrop>::Some(
        PrintOnDrop(rust::Str::from_char_star("option_A")));
    std::cout << "Checkpoint 36" << std::endl;
    auto p2 = Option<PrintOnDrop>::Some(
        PrintOnDrop(rust::Str::from_char_star("option_B")));
    std::cout << "Checkpoint 37" << std::endl;
    p2.take();
    std::cout << "Checkpoint 38" << std::endl;
    p2.take();
    std::cout << "Checkpoint 39" << std::endl;
  }
  std::cout << "Checkpoint 40" << std::endl;
  {
    const char *elems[3] = {"elem1", "elem2", "elem3"};
    int i = 0;
    auto iter = rust::std::iter::from_fn(
        ::rust::Box<::rust::Dyn<::rust::Fn<::rust::std::option::Option<
            ::rust::crate::PrintOnDrop>>>>::make_box([&] {
          if (i == 3) {
            return Option<PrintOnDrop>::None();
          }
          return Option<PrintOnDrop>::Some(
              PrintOnDrop(rust::Str::from_char_star(elems[i++])));
        }));
    std::cout << "Checkpoint 41" << std::endl;
    iter.for_each(
        ::rust::Box<
            ::rust::Dyn<::rust::Fn<::rust::crate::PrintOnDrop, rust::Unit>>>::
            make_box([](PrintOnDrop p) -> rust::Unit {
              std::cout << "Checkpoint 42" << std::endl;
              return {};
            }));
  }
  std::cout << "Checkpoint 43" << std::endl;
}
