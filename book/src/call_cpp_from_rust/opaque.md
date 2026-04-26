# Opaque C++ types

There are currently 3 kinds of opaque types that `zngur` is able to represent

- Reference
- Heap allocated
- Stack allocated

These types are made available to Rust with varying sets of restrictions and tradeoffs imposed
based on your choice

For each of these types (aka. marked `#cpp_ref`, `#cpp_value`, or
`#cpp_stack_owned`), `zngur` will generate a new type within `pub mod cpp {}`.

This module is where all generated opaque types live

## Opaque Borrowed C++ Type

For example, you define a reference-only opaque type in `main.zng` as a `#cpp_ref`:

```
type crate::Way {
    #cpp_ref "::osmium::Way";
}
```

The generated Rust code will contain a `cpp::Way`, which you have to re-export into
`crate::Way`.

> **NOTE**: Since you told `zngur` that `Way` would be defined in
> `crate::Way`, you have to re-export the generated wrapper to the
> correct location (e.g. `pub use generated::cpp::Way;`). This is
> required for any generated opaque type.

Note that `#cpp_ref` types don't need manual layout policy.
This enables creating `rust::Ref<rust::crate::Way>` from a `const osmium::Way&` in C++
and you can pass it to the Rust side.
Rust side can't do anything meaningful with it, except passing it again to the C++ side.
In the C++ side `rust::Ref<rust::crate::Way>` has a `.cpp()` method
which will return the `osmium::Way&` back to you.
If you want to use the methods on your C++ type in the Rust side,
you can write `impl` and `impl trait` blocks for the newtype wrapper `crate::Way` inside C++.
See the [`examples/osmium`](https://github.com/HKalbasi/zngur/blob/main/examples/osmium) for a full working example.

### Semantics of Opaque Borrowed Types

`#cpp_ref` types are implemented as ZSTs that can only be obtained by reference.
While the Rust representation is a ZST, when the reference flows into C++, the reference
is pointing to the C++ object.

This can sometimes cause behavior that is safe and sound,
but surprising and counterintuitive for someone that expects them to represent the whole C++ object.
If you assume that `RustType` is a newtype wrapper around a reference to a
`CppType` class in C++, instead of a ZST, you will find it surprising that:

- `std::mem::sizeof::<RustType>()` is 0, not the size of `CppType`
- `std::mem::alignof::<RustType>()` is 1, not the align of `CppType`
- `std::mem::swap::<RustType>(a, b)` only swaps the first zero bytes of those, i.e. does nothing.

Those problem might be solved by the `extern type` language feature.

## Opaque Heap Allocated C++ Type

Keeping C++ objects in Rust using heap allocation is supported with `#cpp_value` types.
This is similar to `#cpp_ref` types. Look at the example `tutorial_cpp` for a usage example

## Creating trait objects from C++ types

By the above infrastructure you can convert your C++ types into `&dyn Trait` or `Box<dyn Trait>`.
To do that, you need to:

- Create an opaque borrowed (or owned for `Box<dyn Trait>`) type for the C++ type.
- Implement the `Trait` for that type inside C++.
- Cast `&Opaque` to `&dyn Trait` when needed.

There is a shortcut provided by Zngur.
You can define the trait in your `main.zng`:

```
trait iter::Iterator::<Item = i32> {
    fn next(&mut self) -> ::std::option::Option<i32>;
}
```

and inherit in your C++ type from it:

```
template <typename T>
class VectorIterator : public rust::std::iter::Iterator<T> {
  std::vector<T> vec;
  size_t pos;

public:
  VectorIterator(std::vector<T> &&v) : vec(v), pos(0) {}

  Option<T> next() override {
    if (pos >= vec.size()) {
      return Option<T>::None();
    }
    T value = vec[pos++];
    return Option<T>::Some(value);
  }
};
```

Then you can construct a `rust::Box<rust::Dyn>` or `rust::Ref<rust::Dyn>` from it.

```
auto vec_as_iter = rust::Box<rust::Dyn<rust::std::iter::Iterator<int32_t>>>::make_box<
      VectorIterator<int32_t>>(std::move(vec));
```

If you need to call the trait methods on the result, you need to add a `dyn Trait` or `Box<dyn Trait>` in your zng file as well:

```
trait iter::Iterator::<Item = i32> {
    fn next(&mut self) -> ::std::option::Option<i32>;
}

type dyn iter::Iterator::<Item = i32> {
    wellknown_traits(?Sized);

    fn next(&mut self) -> ::std::option::Option<i32>;
    fn map<i32, Box<dyn Fn(i32) -> i32>>(self, Box<dyn Fn(i32) -> i32>)
                -> ::std::iter::Map<::std::vec::IntoIter<i32>, Box<dyn Fn(i32) -> i32>>;
}

type Box<dyn iter::Iterator<Item = i32>> {
    #layout(size = 16, align = 8);
    fn deref(&self) -> &dyn dyn iter::Iterator<Item = i32> use ::core::ops::Deref;
    fn collect<::std::vec::Vec<i32>>(self) -> ::std::vec::Vec<i32>;
}
```

Now you can call collect and map on the resulting iterator defined in C++.
Note that you don't need the `trait` declaration in the zng file if you just need working with trait objects exposed from Rust code.
In that case, just declaring the `type dyn Trait` is enough, and it works like any other type.
The `trait` declaration in the zng file is only needed if you want to use this feature.

### Semantics of Opaque Heap Allocated Types

Heap allocated types are generated as a #[repr(C)] struct to a heap allocated pointer and destructor.

At construction with `::build` in C++, `zngur` allocates an object of the
correct size and a alignment, and initializes it with the forwarded arguments.
The only thing you can do with that object is call C++ methods or `Drop` it
which calls the destructor and frees up the allocation

> **NOTE**: In the future, we may replace C++ owned objects with a
> `unique_ptr`-like type that wraps a cpp_ref instead.

## Trivially Relocatable C++ Types

If we want to have C++ objects directly within the Rust stack, then things
become a bit more complicated. C++'s move semantics are not compatible with
Rust's move semantics.

### C++ Move Semantics

For better or for worse, C++ has very complex machinery to enable efficient
movement of data in memory. In the general case, it uses [`rvalue` references](https://en.cppreference.com/cpp/language/value_category) in order
to convey that the object behind it is "going to expire soon-ish..." and their
resources may be reused. When you construct an `rvalue` reference (specifically
an `xvalue`) with `std::move`, you may pass this reference around to, for
instance, a a move constructor which can use the fact that the provided
reference will expire soon, so instead of copying the underlying data, the
constructor can efficiently steal the underlying data.

Note that the C++ compiler will not prevent you from using the moved-from object
again, and it **will** run the moved-from object's destructor. This means that
moved-from objects must remain in a valid state at all times.

One consequence of this design is that C++'s move semantics are significantly
broader than Rust's since C++ enables library authors to run custom logic when
moving a value in memory.

Furthermore, C++ allows you to `= delete` a move constructor which effectively
prohibits a value from moving in memory once constructed

### Rust's Move Semantics

Meanwhile, Rust's move semantics are very simple. When you move a value, the
Rust compiler will effectively copy the struct's bits and invalidate the moved
from object. The Rust compiler will not run the `Drop` implementation on the
moved-from value.

### Trivial Relocatability

Rust's move semantics can be summarized as "all objects in Rust are trivially
relocatable", and for the purpose of `zngur` interop, a C++ object is considered
trivially relocatable if performing a trivial relocatable move on the object
has the same observable effect as move constructing the C++ object in the new
memory location and running the moved-from objects destructor.

A lot of C++ objects likely follow this guarantee. A few examples likely include:

- Plain old data types
- any `T` with a `T(T&&) = default;` implementation where all members are
  trivially relocatable.
- `std::vector<T>` regardless of `T`'s trivial relocatability.
  - Most other collections
- `std::unique_ptr` and `std::shared_ptr`
- `std::future`

A few **negative** examples are:

- Self referential types
- `libstdc++`'s `std::string` since small-string optimization makes it self-referential
  - NOTE: `stdc++`'s implementation of `std::string` does not do small-string
    optimization. That is to say, this is very tricky stuff...

### C++ Trivially Relocatable in Rust

`zngur` allows you to own a C++ object in the Rust stack so long as the C++
object meets the trivial relocatability guarantees. Let's go over an example
(you can follow along the full example in `examples/stack_owned/`)

```zng
// main.zng
#cpp_additional_includes "
#include <cpp_type.h>
"

type crate::MyCppWrapper {
    #cpp_stack_owned "::CppType" (size = 8, align = 4);
}
```

Similarly to how we can define opaque C++ objects with `#cpp_ref`, we instead
use `#cpp_stack_owned` which instructs `zngur` that this type will be stored
directly in the Rust stack. This requires telling `zngur` the layout information
of the type.

Just like with C++ opaque objects, we can define functions on an `extern C++` block

```zng
extern "C++" {
    fn create_cpp_type(i32, i32) -> crate::MyCppWrapper;
    fn print_cpp_type(&crate::MyCppWrapper);
}
```

The C++ code has access to the `.cpp()` method used to access the inner type.

```rust
// main.rs
mod generated;

pub use generated::cpp::MyCppWrapper;

fn main() {
    println!("Hello from Rust");
    let c = generated::create_cpp_type(10, 20);
    println!("Rust got CppType");
    generated::print_cpp_type(&c);
    println!("Rust dropping CppType");
}
```

The wrapper unconditionally implements the following traits defined in the
`zngur-lib` crate.

- The marker `ZngCppObject`
- The marker `ZngCppStackObject`
- The `ZngCppDestruct` trait with an `unsafe fn destruct(&mut self)`.

In the future, we may add generic functionality around these traits such as safe
in-place construction.

### Type trait

In order to verify the safety requirements of trivial relocatability of a C++ type,
you have to manually specialize the `::rust::is_trivially_relocatable<T> : ::std::true_type {};`
for your type.

```c++
namespace rust {
template <> struct is_trivially_relocatable<CppType> : std::true_type {};
} // namespace rust
```

Failure to do so will cause a static assertion failure in C++.

This is already specialized for all `std::is_trivially_copyable` types and in a future where
`std::is_trivially_relocatable` exists in the standard, we can specialize it for all types
