# Name mapping

Not all Rust types have the equivalent syntax in C++,
so Zngur uses a name mapping that is explained in this table:

| Zngur type                               | C++ Type                                       |
| ---------------------------------------- | ---------------------------------------------- |
| `uX`                                     | `uintX_t`                                      |
| `iX`                                     | `intX_t`                                       |
| `bool`                                   | `rust::Bool`                                   |
| `char`                                   | `rust::Char`                                   |
| `some_crate::some_mod::SomeAdt<A, B, C>` | `rust::some_crate::some_mod::SomeAdt<A, B, C>` |
| `&T`                                     | `rust::Ref<T>`                                 |
| `&mut T`                                 | `rust::RefMut<T>`                              |
| `*const T`                               | `rust::Raw<T>` or `const T*` (depends on `T`)  |
| `*mut T`                                 | `rust::RawMut<T>` or `T*` (depends on `T`)     |
| `[T]`                                    | `rust::Slice<T>`                               |
| `dyn T`                                  | `rust::Dyn<T>`                                 |
| `dyn T + Marker1 + Marker2`              | `rust::Dyn<T, rust::Marker1, rust::Marker2>`   |
| `()`                                     | `rust::Unit` or `rust::Tuple<>`                |
| `(A, B, C)`                              | `rust::Tuple<A, B, C>`                         |

## Why `rust::Ref<T>` instead of C++ references?

Because they are very different:

- Using two Rust `&mut` or one `&` and one `&mut` in Rust is UB.
- It is extremely easy to move out of a C++ reference,
  or use a method that takes ownership on it.
  By using `Ref<T>` such code will be rejected at compile time.
- Not all Rust references are one word,
  for example `&str`, `&[T]` or `&dyn Trait` are two words, incompatible with C++ references.
- C++ representation of a Rust type may have additional fields,
  such as a drop flag (see [how it works](../how_it_works.html)),
  so a Rust reference coming from Rust code is not necessarily a valid C++ reference for the C++ representation.
- You can't have a reference to reference, pointer to reference,
  vector of references and similar things with C++ references,
  but with `Ref<T>` those would be possible.

But Zngur tries to keep convenience of C++ references when using Rust `Ref<T>`.
For example, `Ref<T>` is constructible from `T` so you can pass `T` to a function that expect a `Ref<T>`,
similar to a `T&`.
Or Zngur tries to emulate Rust's auto dereference rules on methods
so you can call a method directly on the `Ref<T>` without needing to use `->` or `*` operator.

## Why `rust::Bool` instead of C++ `bool`?

C++ `bool` size is implementation defined and not necessarily 1,
and `true` and `false` are not necessarily encoded using `1` and `0`.
That is, the only valid way to create a `bool` is using it's standard constructors.
These are all valid:

```C++
bool b = 1;
bool b = true;
bool b = 5;
```

But this is UB:

```C++
bool b = true;
char* bc = (char*)&b;
*bc = 0;
```

So Zngur can't use `bool` the way it uses `uint32_t` and friends.
But it adds some special codes to `bool`
so that you can use it in an `if` statement or cast it to and from C++ `bool`.
