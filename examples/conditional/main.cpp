
#include <cmath>
#include <iostream>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include "generated.h"

// Rust values are available in the `::rust` namespace from their absolute path
// in Rust
template <typename T> using Vec = rust::std::vec::Vec<T>;
template <typename T> using Option = rust::std::option::Option<T>;

using KVPair = rust::crate::KeyValuePair;

#ifdef ZNGUR_CFG_FEATURE_FLOAT_VALUES
using KVPairValue_T = ::std::double_t;
#else
using KVPairValue_T = ::std::int32_t;
#endif

int main() {
  auto s = Vec<KVPair>::new_();

  std::vector<std::pair<rust::Ref<rust::Str>, int>> pairs = {
      {"foo"_rs, 1}, {"bar"_rs, 2}, {"baz"_rs, 3}, {"abc"_rs, 4}};

  for (const auto &[key, value] : pairs) {
    s.push(KVPair{key.to_owned(), static_cast<KVPairValue_T>(value)});
  }
  auto it = s.iter();
  for (auto next = it.next(); next.matches_Some(); next = it.next()) {
    auto pair = next.unwrap();
    rust::Ref<rust::std::string::String> key = pair.key;
    KVPairValue_T value = pair.value;
    std::cout << "KVPair(size = " << KVPair::self_size()
              << ", align = " << KVPair::self_align() << "){"
              << std::string_view(
                     reinterpret_cast<const char *>(key.as_str().as_ptr()),
                     key.len())
              << " : " << std::to_string(value) << "}\n";
    zngur_dbg(pair);
  }
}
