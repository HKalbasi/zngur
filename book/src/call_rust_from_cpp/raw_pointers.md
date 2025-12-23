# Raw pointers

Raw pointers `*mut T` and `*const T` are Rust primitve types used in `unsafe` Rust code.

## Raw pointers to primitive types

A raw pointer to a primitive type available in both C++ and Rust becomes a normal pointer in C++.
For example, `*const i32` becomes a `const int32_t*` and you can use it like a normal pointer.

## Raw pointers to Rust specific types

For a Rust type `T`, Zngur generates `rust::Raw<rust::T>` for `*const T` and `rust::RawMut<rust::T>` for `*mut T`.
We need `rust::Raw` to handle fat raw pointers and differences between memory layout of `rust::T` in C++ and `T` in Rust.
These types also provide some convenient methods for working with raw pointers.

Members available in both `rust::Raw<T>` and `rust::RawMut<T>`:

- Constructor from `uint8_t*` or `rust::zngur_fat_pointer` based on whether `T` is sized.
- Constructor from `Ref<T>` or `RefMut<T>`. Semantically equivalent to `&T as *const T` in Rust.
- `offset(n)`: advances the pointer `n` elements of `T`. Semantically equivalent to [`<*const T>::offset`] in Rust.
- `read_ref()`: Creates a `rust::Ref<T>`. Semantically equivalent to `&*ptr` in Rust.

Members available only in `rust::RawMut<T>`:

- `read_mut()`: Creates a `rust::RefMut<T>`. Semantically equivalent to `&mut *ptr` in Rust.
- `read()`: Moves out of the raw pointer and creates a `T`. Semantically equivalent to [`std::ptr::read`] in Rust.
- `write(T)`: Writes into the pointer. Semantically equivalent to [`std::ptr::write`] in Rust.

## Safety

Working with raw pointers is extremely unsafe. Here is a non-exhaustive list of unexpected things that may happen:

- You should derive raw pointers from Rust functions or references. Creating raw pointers out of thin air and casting
  integers to pointers is considered UB. See [strict provenance] apis for constructing raw pointers.
- All `read` and `write` methods require the pointer to be aligned. Doing unaligned read and writes is UB.
- Using write does not invoke the destructor of the old value. Use [`std::ptr::drop_in_place`] for destructing the
  old value if needed.
- Zngur exposes the metadata part of fat pointers, but it is considered unstable by Rust and exact data in the
  metadata part may change in future compiler versions.
- General UBs of using pointers in C++ (e.g. use after free) are also UB in Rust raw pointers.

[`std::ptr::read`]: https://doc.rust-lang.org/std/ptr/fn.read.html
[`std::ptr::write`]: https://doc.rust-lang.org/std/ptr/fn.write.html
[`std::ptr::drop_in_place`]: https://doc.rust-lang.org/std/ptr/fn.drop_in_place.html
[`<*const T>::offset`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.offset
[strict provenance]: https://doc.rust-lang.org/std/ptr/index.html#strict-provenance
