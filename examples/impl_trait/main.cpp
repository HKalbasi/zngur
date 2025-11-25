#include <iostream>
#include <vector>

#include "./generated.h"

int main() {
    rust::crate::argument_position_impl_trait(5);
    rust::crate::argument_position_impl_trait("hello"_rs.to_owned());
}
