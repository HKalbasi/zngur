#include "./generated.h"

using ::operator""_rs;

int main() {
    // Pass a char literal directly using the _rs suffix
    rust::crate::CharPrinter::print(U'A'_rs);

    // Pass a char from a char32_t variable
    char32_t c = U'\u00E9'; // é
    rust::crate::CharPrinter::print(operator""_rs(c));

    // Use a char in a predicate
    if (rust::crate::CharPrinter::is_alphabetic(U'z'_rs)) {
        rust::crate::CharPrinter::print(rust::crate::CharPrinter::to_uppercase(U'z'_rs));
    }

    // Invalid chars are replaced with U+FFFD
    char32_t invalid = 0xD800; // Surrogate code point
    rust::crate::CharPrinter::print(operator""_rs(invalid));

    return 0;
}
