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

## Fields as underlying types

When you declare fields in a tuple or struct using `field name (offset = X, type = T);`, the generated C++ exposes helper wrapper types:

- `rust::FieldOwned<T, OFFSET>` for fields on owning types
- `rust::FieldRef<T, OFFSET>` for fields on `Ref<Ty>`
- `rust::FieldRefMut<T, OFFSET>` for fields on `RefMut<Ty>`

These wrappers now act as their underlying type `T` in many contexts:

- `Ref<T>` construction from any `Field*<T, OFFSET>`
- Implicit read via `operator T()` and `.read()` for value-like access
- Method calls are forwarded when applicable

Example:

```C++
rust::Tuple<int32_t, rust::std::string::String> t{42, "hi"_rs.to_owned()};

// Read value
int32_t v = t.f0; // operator T() on FieldOwned<int32_t, 0>

// Get a Ref<T> from a field
rust::Ref<int32_t> r = t.f0;

// Access methods through Ref from Field wrappers
rust::Ref<rust::std::string::String> sref = t.f1;
auto len = sref.len();

// From references to container, fields become FieldRef/FieldRefMut
rust::Ref<decltype(t)> rt = t;
auto l1 = rt.f1.len();

rust::RefMut<decltype(t)> mt = t;
mt.f1.push_str("!"_rs);
```

See `examples/regression_test1` for a runnable demonstration.
