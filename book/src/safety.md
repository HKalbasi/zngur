# Safety

Rust is a pretty strict language about its rules, and its rules are not necessarily equivalent to the C++ rules that C++ developers
know. So, extra care is needed in using C++ and Rust together in a project. By using Zngur, Rust side remains a normal, idiomatic and
safe code which can't cause undefined behavior (UB), but nothing prevents the C++ side to break Rust expectations. This page
lists the ways that it can break the rules and things to consider for preventing it.

## C++ functions exposed into Rust

Zngur supports multiple ways of calling C++ functions in Rust, including free functions in `rust::exported_functions` namespace, writing `impl` block
for Rust types, and converting a `std::function` into a `Box<dyn FnX()>`. In all of these, your C++ function should behave like a safe Rust function
that avoids UB in all cases. This property is called _soundness_, and if there exists some inputs and conditions that cause UB, then your
function is _unsound_. You can assume these about your parameters:

- `&mut T` is a valid reference at least until the end of your function to a `T`, and you have exclusive access to it.
- `&T` is a valid reference at least until the end of your function to a `T`, but there may exist other threads that are
  reading it at the same time (and even writing on it, if it is an interior mutable type).
- For `*mut T` and `*const T`, there is no guarantee. They may be dangling, null, unaligned, and anything. A safe Rust function basically can't
  do anything useful with a raw pointer in its arguments.

If your function is unsound and requires some specific preconditions to be called, consider making it `unsafe` to prevent Rust code from calling
it carelessly.

## Lifetimes

## Mutability XOR aliasing

## Calling unsafe Rust code
