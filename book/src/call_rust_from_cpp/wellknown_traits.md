# Wellknown traits

Some Rust traits from the standard library have special support or effect in Zngur. They are listed on this page.
You can use them by adding `wellknown_traits(Trait1, Trait2);` statement for your type in the `main.zng` file.

## Copy

Adding this trait enables the copy constructor for the type on the C++ side,
and removes the drop flag and destructor-related generated code, so it will improve performance.

## ?Sized

Marks the type as unsized and the pointers to them as fat. Unlike other wellknown traits, this is mandatory and missing it
for a type will result in a compile error.

## Debug

Enables using `zngur_dbg` macro for a type. `zngur_dbg` is a macro similar to `dbg!` in Rust which prints the location, name
and value of expression inside it, like this:

```
[main.cpp:64] t = [
    10,
    20,
    60,
]
```

and returns it for further use.

## PartialEq, PartialOrd, Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor, Shl, Shr

{{#include ../unimplemented_begin.md}}2{{#include ../unimplemented_end.md}}

These traits add operator overloading for binary operators `==`, `<=>`, `+`, `-`, `*`, `/`, `%`, `&`, `|`, `^`, `<<` and `>>` respectively.

## IntoIterator

{{#include ../unimplemented_begin.md}}2{{#include ../unimplemented_end.md}}

Enables converting the type into a C++ iterable object, so you can use it in `for (auto x: iterator)` and standard library functions
that expect an iterator.
