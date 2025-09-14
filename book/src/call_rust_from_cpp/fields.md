# Fields as underlying types

When you declare fields in a tuple or struct using `field name (offset = X, type = T);`,
the generated C++ exposes helper wrapper types:

- `rust::FieldOwned<T, OFFSET>` for fields on owning types
- `rust::FieldRef<T, OFFSET>` for fields on `Ref<Ty>`
- `rust::FieldRefMut<T, OFFSET>` for fields on `RefMut<Ty>`

These wrappers now act as their underlying type `T` in many contexts:

- `Ref<T>` construction from any `Field*<T, OFFSET>`
- Implicit read via `operator T()` for value-like access
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
