#include <cstdint>
#include "./generated.h"

namespace rust {
namespace exported_functions {
    int32_t dangerous_function() {
        rust::crate::RustStruct::unsafe_rust_fn();
        rust::crate::RustStruct::safe_rust_fn();
        return 42;
    }
}
}
