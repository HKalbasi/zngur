#include <iostream>
#include <vector>

#include "./generated.h"

void test_dbg_works_for_ref_and_refmut() {
  auto scope = rust::crate::Scoped::new_("Test dbg works for Ref and RefMut"_rs);
  
  rust::Ref<rust::Str> v1 = "foo"_rs;
  zngur_dbg(v1);
  rust::std::string::String v2 = v1.to_owned();
  zngur_dbg(v2);
  rust::Ref<rust::std::string::String> v3 = v2;
  zngur_dbg(v3);
  rust::std::string::String v4 = std::move(zngur_dbg(v2));
  zngur_dbg(v4);
  rust::RefMut<rust::std::string::String> v5 = v4;
  zngur_dbg(v5);
  v5.push_str(zngur_dbg("bar"_rs));
  zngur_dbg(v4);
}

template<typename T>
concept has_push_str = requires(T v, rust::Ref<rust::Str> s) { 
  v.push_str(s); 
};


void test_fields_and_constructor() {
  auto scope = rust::crate::Scoped::new_("Test fields and constructor work"_rs);

  rust::crate::Foo v1 = rust::crate::Foo{1, "bar"_rs.to_owned()};
  zngur_dbg(v1);
  zngur_dbg(v1.field2);
  zngur_dbg(v1.field2.len());
  v1.field2.push_str("baz"_rs);
  zngur_dbg(v1);

  rust::Tuple<rust::std::string::String, rust::crate::Foo> v2{"kkk"_rs.to_owned(), std::move(v1)};
  zngur_dbg(v2);
  zngur_dbg(v2.f0);
  zngur_dbg(v2.f1);
  zngur_dbg(v2.f1.field2);
  v2.f1.field2.push_str("xxx"_rs);
  
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
  v4.f1.field2.push_str("yyy"_rs);
  zngur_dbg(v4.f1.field2.len());
}

void test_field_underlying_conversions() {
  auto scope = rust::crate::Scoped::new_("Test Field* underlying conversions"_rs);

  rust::Tuple<int32_t, rust::std::string::String> pair{42, "hi"_rs.to_owned()};

  // FieldOwned conversion to Ref and value
  rust::Ref<int32_t> r0 = pair.f0;
  int32_t v0 = pair.f0;
  zngur_dbg(v0);
  // TODO: Add support for conversion to T for all fields.
  // rust::std::string::String v1 = pair.f1;

  // FieldOwned<String> to Ref<String> and call a method
  rust::Ref<rust::std::string::String> sref = pair.f1;
  zngur_dbg(sref.len());

  rust::Ref<rust::Tuple<int32_t, rust::std::string::String>> pref = pair;
  zngur_dbg(int32_t(pref.f0));
  zngur_dbg(pref.f1.len());

  rust::RefMut<rust::Tuple<int32_t, rust::std::string::String>> pmut = pair;
  zngur_dbg(int32_t(pmut.f0));
  pmut.f1.push_str("!"_rs);
  zngur_dbg(pmut.f1.len());
}

int main() {
  test_dbg_works_for_ref_and_refmut();
  test_fields_and_constructor();
  test_field_underlying_conversions();
}
