# Enum variants

Enum variants can be declared with `variant name { }` syntax.
Inside the block you can specify the fields of a variant,
similarly to how struct fields are declared:

```
type ::std::option::Option<i32> {
    variant None { }
    variant Some {
        field 0 (offset = auto, type = i32);
    }
}
```

Note that due to technical limitations only `auto` offsets are supported.

By default, if any variants are declared, zngur will verify at compile time
that declarations are exhaustive, i.e. no variants and no variant fields are missing. `non_exhaustive` declaration can be used to disable this check.

```
type ::std::option::Option<i32> {
    non_exhaustive; // Some variants might be missing.
    variant Some {
        non_exhaustive; // Some variant fields might be missing.
    }
}
```

For every declared variant zngur will generate a nested class.
If a variant is exhaustive, this class will have a constructor to build it from fields:

```C++
template<class T> using Option = rust::std::option::Option<T>;

Option<int> opt = Option<int>::Some(42);
```

Each variant class has a `::match()` static method returning `std::optional` to check if the enum contains that variant:

```C++
if (auto r = Option<int>::Some::match(v)) {
    std::cout << r->f0 << std::endl;
}
```

`::match_ref()` and `::match_mut()` static methods can similarly be used for by-ref matching,
returning `std::optional<Ref<_>>` and `std::optional<RefMut<_>>` respectively.

These methods are only available on C++17 or higher, since they require `std::optional`.

If you don't need the fields, you can also use the `::check()` static method:

```C++
if (Option<int>::None::check(v)) {
    std::cout << "is none" << std::endl;
}
```

Alternatively, `.match()` methods on the enum class can be used to convert the enum into `std::variant`
of variant classes (or refs to variant classes):

```C++
std::visit(overloaded{
    [](rust::Ref<Option<int>::None>) { std::cout << "none" << std::endl; },
    [](rust::Ref<Option<int>::Some> s) { std::cout << "some " << s.f0 << std::endl; },
}, v.match_ref());
```

This is also only available starting with C++17.

See `examples/enums` for a runnable demonstration.
