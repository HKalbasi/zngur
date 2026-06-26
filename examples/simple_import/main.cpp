#include <iostream>
#include "generated.h"

template<class T> using Vec = rust::std::vec::Vec<T>;
template<class T> using Option = rust::std::option::Option<T>;
using String = rust::std::string::String;

// External function declarations from foo.cpp and bar.cpp
extern Vec<int32_t> foo();
extern Option<String> bar();

int main() {
    // Get Vec<i32> from foo() and demonstrate both imported and extended APIs
    std::cout << "foo(): Creating a Rust Vec<i32>" << std::endl;
    auto numbers = foo();
    std::cout << "  Vec contents: ";
    zngur_dbg(numbers);

    // Use extended APIs defined in main.zng
    std::cout << "  Vec is_empty(): " << numbers.is_empty() << std::endl;
    numbers.clear();  // Clear the vector using main.zng API
    std::cout << "  After clear(), is_empty(): " << numbers.is_empty() << std::endl;
    std::cout << std::endl;

    // Get Option<String> from bar() and demonstrate both imported and extended APIs
    std::cout << "bar(): Creating Rust Option<String>" << std::endl;
    auto some_value = bar();
    Option<String> none_value = Option<String>::None();

    // Use imported APIs from bar.zng
    std::cout << "  some_value.is_some(): " << some_value.is_some() << std::endl;
    std::cout << "  none_value.is_none(): " << none_value.is_none() << std::endl;

    // Use extended API defined in main.zng
    auto unwrapped_string = some_value.unwrap();  // Use main.zng API
    std::cout << "  Unwrapped string: ";
    zngur_dbg(unwrapped_string);
    return 0;
}
