# Types with special support

## bool

The `rust::Bool` type has an `operator bool()` so you can use this type directly in if statements and ternary expressions. This type
also has a constructor from C++ `bool` so you can pass `true` and `false` to functions that take `rust::Bool` in input.

## str

The `rust::Str` has an static method `from_char_star` which creates a non owning `rust::Ref<rust::Str>` from a `const char *` so
you can create rust `str` from a string literal, `std::string` or other C strings in the C++ side. For example, for converting
a C++ `std::string` into a Rust `String`, you can use `rust::Str::from_char_star(s.c_str()).to_string()`.

For the Rust to C++ side, no special function is provided. You can use `as_ptr` method for getting a pointer to the beginning and `len` method
for the number of bytes. For example for converting a `&str` to a C++ `std::string` you can use `std::string((char*)s.as_ptr(), s.len())`. Note
that Rust `&str`s are not NUL terminated and passing `as_ptr` pointer to functions that expect a C string is invalid.

The `from_char_star` method is just a shortcut and is fairly limited. To fully control the process, consider using one of the more specialized
Rust string types like `CStr` or `OsString`.
