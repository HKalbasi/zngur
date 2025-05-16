#include <cstdint>
#include <iostream>
#include <vector>

#include "./generated.h"

template <typename T> using Vec = rust::std::vec::Vec<T>;
template <typename T> using Option = rust::std::option::Option<T>;
template <typename T> using BoxDyn = rust::Box<rust::Dyn<T>>;
template <typename T> using RmDyn = rust::RefMut<rust::Dyn<T>>;
using namespace rust::crate;

class CppPrintOnDropHolder : public PrintOnDropConsumer {
  rust::Unit consume(PrintOnDrop p) override {
    item = std::move(p);
    return {};
  }

  PrintOnDrop item;
};

int main() {
  auto p1 = PrintOnDrop("A"_rs);
  auto p2 = PrintOnDrop("B"_rs);
  auto p3 = PrintOnDrop("C"_rs);
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
    vec1.emplace_back("cpp_V1"_rs);
    vec1.emplace_back("cpp_V2"_rs);
    vec1.emplace_back("cpp_V3"_rs);
    std::cout << "Checkpoint 5" << std::endl;
    vec1.pop_back();
    vec1.pop_back();
    std::cout << "Checkpoint 6" << std::endl;
  }
  {
    std::cout << "Checkpoint 7" << std::endl;
    Vec<PrintOnDrop> vec2 = Vec<PrintOnDrop>::new_();
    vec2.push(PrintOnDrop("rust_V1"_rs));
    vec2.push(PrintOnDrop("rust_V2"_rs));
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
      consume_n_times(holder.deref_mut(), "P"_rs, 3);
      std::cout << "Checkpoint 12" << std::endl;
      consume_n_times(holder.deref_mut(), "Q"_rs, 2);
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
      consume_n_times(holder, "P2"_rs, 3);
      std::cout << "Checkpoint 17" << std::endl;
      consume_n_times(holder, "Q2"_rs, 2);
      std::cout << "Checkpoint 18" << std::endl;
    }
    std::cout << "Checkpoint 19" << std::endl;
  }
  std::cout << "Checkpoint 20" << std::endl;
  try {
    PrintOnDrop a{"A"_rs};
    std::cout << "Checkpoint 21" << std::endl;
    consume_and_panic(a.clone(), false);
    std::cout << "Checkpoint 22" << std::endl;
    consume_and_panic("B"_rs, true);
    std::cout << "Checkpoint 23" << std::endl;
  } catch (rust::Panic e) {
    std::cout << "Checkpoint 24" << std::endl;
  }
  {
    std::cout << "Checkpoint 25" << std::endl;
    PrintOnDropPair p{"first"_rs,
                      "second"_rs};
    std::cout << "Checkpoint 26" << std::endl;
  }
  {
    std::cout << "Checkpoint 27" << std::endl;
    Vec<PrintOnDrop> vec2 = Vec<PrintOnDrop>::new_();
    vec2.push(PrintOnDrop("elem1"_rs));
    vec2.push(PrintOnDrop("elem2"_rs));
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
    auto p1 = zngur_dbg(PrintOnDrop("dbg_A"_rs));
    std::cout << "Checkpoint 32" << std::endl;
    auto p2 = zngur_dbg(std::move(p1));
    std::cout << "Checkpoint 33" << std::endl;
    zngur_dbg(p2);
    std::cout << "Checkpoint 34" << std::endl;
  }
  std::cout << "Checkpoint 35" << std::endl;
  {
    auto p1 = Option<PrintOnDrop>::Some(
        PrintOnDrop("option_A"_rs));
    std::cout << "Checkpoint 36" << std::endl;
    auto p2 = Option<PrintOnDrop>::Some(
        PrintOnDrop("option_B"_rs));
    std::cout << "Checkpoint 37" << std::endl;
    p2.take();
    std::cout << "Checkpoint 38" << std::endl;
    p2.take();
    std::cout << "Checkpoint 39" << std::endl;
  }
  std::cout << "Checkpoint 40" << std::endl;
  {
    rust::Ref<rust::Str> elems[3] = {"elem1"_rs, "elem2"_rs, "elem3"_rs};
    int i = 0;
    auto iter = rust::std::iter::from_fn(
        rust::Box<rust::Dyn<rust::Fn<Option<PrintOnDrop>>>>::make_box([&] {
          if (i == 3) {
            return Option<PrintOnDrop>::None();
          }
          return Option<PrintOnDrop>::Some(
              PrintOnDrop(elems[i++]));
        }));
    std::cout << "Checkpoint 41" << std::endl;
    iter.for_each(
        rust::Box<rust::Dyn<rust::Fn<PrintOnDrop, rust::Unit>>>::make_box(
            [](PrintOnDrop p) -> rust::Unit {
              std::cout << "Checkpoint 42" << std::endl;
              return {};
            }));
  }
  std::cout << "Checkpoint 43" << std::endl;
  {
    auto tuple = rust::Tuple<PrintOnDrop, int32_t, PrintOnDrop>(
        PrintOnDrop("field_0"_rs), 5,
        PrintOnDrop("field_2"_rs));
    std::cout << "Checkpoint 44" << std::endl;
  }
  std::cout << "Checkpoint 45" << std::endl;
}
