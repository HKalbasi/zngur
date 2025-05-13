#include <iostream>
#include <vector>

#include "./generated.h"

void test_dbg_works_for_ref_and_refmut() {
  auto scope = rust::crate::Scoped::new_(rust::Str::from_char_star("Test dbg works for Ref and RefMut"));
  
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

template<typename T>
concept has_push_str = requires(T v, rust::Ref<rust::Str> s) { 
  v.push_str(s); 
};


void test_fields_and_constructor() {
  auto scope = rust::crate::Scoped::new_(rust::Str::from_char_star("Test fields and constructor work"));

  rust::crate::Foo v1 = rust::crate::Foo{1, rust::Str::from_char_star("bar").to_owned()};
  zngur_dbg(v1);
  zngur_dbg(v1.field2);
  zngur_dbg(v1.field2.len());
  v1.field2.push_str(rust::Str::from_char_star("baz"));
  zngur_dbg(v1);

  rust::Tuple<rust::std::string::String, rust::crate::Foo> v2{rust::Str::from_char_star("kkk").to_owned(), std::move(v1)};
  zngur_dbg(v2);
  zngur_dbg(v2.f0);
  zngur_dbg(v2.f1);
  zngur_dbg(v2.f1.field2);
  v2.f1.field2.push_str(rust::Str::from_char_star("xxx"));
  
  rust::Ref<rust::Tuple<rust::std::string::String, rust::crate::Foo>> v3 = v2;
  zngur_dbg(v3.f0);
  zngur_dbg(v3.f1);
  zngur_dbg(v3.f1.field2);
  static_assert(has_push_str<decltype(v2.f1.field2)>);
  static_assert(!has_push_str<decltype(v3.f1.field2)>);
  zngur_dbg(v3.f1.field2.len());

  rust::RefMut<rust::Tuple<rust::std::string::String, rust::crate::Foo>> v4 = v2;
  zngur_dbg(v4.f0);
  zngur_dbg(v4.f1);
  zngur_dbg(v4.f1.field2);
  v4.f1.field2.push_str(rust::Str::from_char_star("yyy"));
  zngur_dbg(v4.f1.field2.len());
}

int main() {
  test_dbg_works_for_ref_and_refmut();
  test_fields_and_constructor();
}
