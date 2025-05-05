#include <generated.h>

template <typename T>
using Vec = rust::std::vec::Vec<T>;
using u64 = uint64_t;

namespace rust::exported_functions {
    auto build_vec_by_push_cpp(u64 n) -> Vec<u64> {
        auto v = Vec<u64>::new_();
        for (u64 i = 0; i < n; ++i) {
            v.push(i);
        }
        return v;
    }
}