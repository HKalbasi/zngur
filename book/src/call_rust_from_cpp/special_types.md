# Types with special support

## bool

The `rust::Bool` type has an `operator bool()` so you can use this type directly in if statements and ternary expressions. This type
also has a constructor from C++ `bool` so you can pass `true` and `false` to functions that take `rust::Bool` in input.

## literals

In Rust there are many kind of literal expressions, some of them are natively supported in C++, like integer literals. For the rest, Zngur
tries to support them using C++ feature called User-defined Literals.

| Syntax         | Rust Equivalent | Output Type                       | Status          | Enabled With        |
| -------------- | --------------- | --------------------------------- | --------------- | ------------------- |
| `'a'_rs`       | `'a'`           | `rust::Char`                      | Not Implemented | `char`              |
| `"hello"_rs`   | `"hello"`       | `rust::Ref<rust::Str>`            | Implemented     | `str`               |
| `'a'_rs_b`     | `b'a'`          | `uint8_t`                         | Not Implemented | unconditionally     |
| `"hello"_rs_b` | `b"hello"`      | `rust::Ref<rust::Slice<uint8_t>>` | Not Implemented | `[u8]`              |
| `"hello"_rs_c` | `c"hello"`      | `rust::Ref<rust::ffi::CStr>`      | Not Implemented | `::rust::ffi::CStr` |
