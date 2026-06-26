#include <cstdio>
#include <iostream>
#include <vector>

#include "./generated.h"

template<class... Ts> struct overloaded : Ts... { using Ts::operator()...; };
template<class... Ts> overloaded(Ts...) -> overloaded<Ts...>;

template<class T> using Option = rust::std::option::Option<T>;
using Merged = rust::crate::Merged;

int main() {
    Option<int> v = Option<int>::Some(42);
    std::visit(overloaded{
        [](Option<int>::None) { std::cout << "none" << std::endl; },
        [](Option<int>::Some s) { std::cout << "some " << s.f0 << std::endl; },
    }, v.match());

    if (auto r = Option<int>::Some::match_mut(v)) {
      *rust::RefMut(r->f0) = 67;
    }

    if (auto r = Option<int>::Some::match(v)) {
      std::cout << r->f0 << "!!" << std::endl;
    }

    if (Option<int>::Some::check(v)) {
        std::cout << "is some" << std::endl;
    }

    std::visit(overloaded{
        [](rust::RefMut<Option<int>::None>) {  },
        [](rust::RefMut<Option<int>::Some> s) { *rust::RefMut(s.f0) = 1337; },
    }, v.match_mut());

    std::visit(overloaded{
        [](rust::Ref<Option<int>::None>) { std::cout << "none" << std::endl; },
        [](rust::Ref<Option<int>::Some> s) { std::cout << "some " << s.f0 << std::endl; },
    }, v.match_ref());

    // Test that we can use all the merged info.
    Merged m = Merged::first();
    if (auto r = Merged::First::match(m)) {
        std::cout << "first: " << r->f0 << std::endl;
    }
    m = Merged::second();
    if (auto r = Merged::Second::match(m)) {
        std::cout << "second: " << r->f0 << std::endl;
    }
}
