#include "generated.h"

// Creates and returns a Rust Vec<i32> with sample data
rust::std::vec::Vec<int32_t> foo() {
    // Create a new Rust Vec<i32>
    auto numbers = rust::std::vec::Vec<int32_t>::new_();

    // Add some numbers
    numbers.push(10);
    numbers.push(20);
    numbers.push(30);

    return numbers;
}
