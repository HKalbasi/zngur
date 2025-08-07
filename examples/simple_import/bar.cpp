#include "generated.h"

// Creates and returns a Rust Option<String> with Some(String)
rust::std::option::Option<rust::std::string::String> bar() {
    // Create Some(String)
    return rust::std::option::Option<rust::std::string::String>::Some(
        rust::std::string::String::new_()
    );
}
