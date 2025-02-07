#include <iostream>
#include <vector>

#include "./generated.h"

int main() {
  // Test dbg works for Ref and RefMut
  rust::Ref<rust::Str> v1 = rust::Str::from_char_star("foo");
  zngur_dbg(v1);
  rust::std::string::String v2 = v1.to_owned();
  zngur_dbg(v2);
  rust::Ref<rust::std::string::String> v3 = v2;
  zngur_dbg(v3);
  rust::std::string::String v4 = std::move(zngur_dbg(v2));
  zngur_dbg(v4);
  rust::RefMut<rust::std::string::String> v5 = v4;
  zngur_dbg(v5);
  v5.push_str(zngur_dbg(rust::Str::from_char_star("bar")));
  zngur_dbg(v4);
}
