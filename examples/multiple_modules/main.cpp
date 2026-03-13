#include "main.zng.h"
#include <iostream>

using namespace main_ns::std::vec;

int main() {
    auto v = Vec<int32_t>::new_();
    std::cout << "Created Vec!" << std::endl;
    return 0;
}
