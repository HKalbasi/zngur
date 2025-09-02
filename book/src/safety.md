# Safety

Rust is a pretty strict language, and its rules are not necessarily equivalent to the C++ rules that C++ developers know.
So, extra care is needed when using C++ and Rust together.
By using Zngur, the Rust side remains normal, idiomatic and safe code that can't cause undefined behavior (UB),
but nothing prevents the C++ side from breaking Rust expectations.
This page lists the ways that it can break the rules and things to consider for preventing it.

## C++ functions exposed into Rust

Zngur supports multiple ways of calling C++ functions in Rust, including free functions in the `rust::exported_functions` namespace,
writing `impl` blocks for Rust types, and converting a `std::function` into a `Box<dyn FnX()>`.
In all of these, your C++ function should behave like a safe Rust function that avoids UB in all cases.
This property is called *soundness*, and if there exist some inputs and conditions that cause UB, then your function is *unsound*.
You can assume these things about your parameters:

- `&mut T` is a valid reference at least until the end of your function to a `T`, and you have exclusive access to it.
- `&T` is a valid reference at least until the end of your function to a `T`, but there may exist other threads that are
  reading it at the same time (and even writing on it, if it is an interior mutable type).
- For `*mut T` and `*const T`, there is no guarantee. They may be dangling, null, unaligned, and anything. A safe Rust function basically can't
  do anything useful with a raw pointer in its arguments.

If your function is unsound and requires some specific preconditions to be called, consider making it `unsafe` to prevent Rust
code from calling it carelessly.

## Lifetimes

In Rust, references have lifetime, and the borrow checker enforces that references are used correctly. In C++
there is no borrow checker, so you need to do the borrow checker job manually yourself.

For example, this code is invalid:

```C++
auto v = Vec<int32_t>::new_();
v.push(1);
v.push(2);
auto r = v.get(0);
some_function_that_consume_vector(std::move(v));
zngur_dbg(r);
```

the `Vec::get` function has signature `Vec::get(&self) -> Option<&i32>` and if we un-elide the lifetimes
it will become `Vec::get<'a>(&'a self) -> Option<&'a i32>` so the reference to the input should live
at least as long as the output. The output is used in the last line, but the reference to the input
is killed by the move happened in the above line, so this code is invalid.

You need to do this manual work in C++ codes as well, and in most cases doing it with Zngur generated code
is easier, since you don't need to know about the internal implementations and you can just
decide by seeing function signatures.

You should also handle lifetimes in functions exposed to Rust, and use correct
and explicit lifetime annotations in the `main.zng` if the elided version does not reflect your requirements.

## Mutability XOR aliasing

This is another job of the borrow checker that you need to do manually. You should either have
a single mutable reference, or some immutable references. This restriction does not exist in
C++, and in C++ it is perfectly valid to create multiple mutable references, so extra care
is needed by C++ developers to adhere to this rule.

The motivation behind this rule is, every single safe Rust function should work in a way such that
calling it multiple times in multiple threads doesn't introduce data races, which necessarily implies
that mutable references should be unique. In C++, not all functions are expected to be called in
multiple threads, so there is no such requirement. That said, you should adhere to this rule even
in completely single threaded programs, as it can also affect memory safety. For example:

```C++
auto v = Vec<int32_t>::new_();
v.push(1);
v.push(2);
auto r = v.get(0);
v.push(3);
zngur_dbg(r);
```

this program (and its C++ equivalent) may have UB as the vector may reallocate on the `v.push(3)`, invalidating
the `r` reference. But by adhering to the Rust borrow checker rules we can avoid this kind of UB. This code
is violating the mutability XOR aliasing rule since the immutable reference `r` and the mutable reference created
by `v.push(3)` are alive at the same time.

## Calling unsafe Rust code

Zngur allows you to call `unsafe` Rust functions just like normal functions. You need to read the documentation
of such functions carefully to satisfying their preconditions.
