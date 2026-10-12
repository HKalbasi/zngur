# Opaque C++ types

There are currently 3 kinds of opaque types that `zngur` is able to represent

- Reference
- Heap allocated
- Stack allocated

These types are made available to Rust with varying sets of restrictions and tradeoffs imposed
based on your choice

Types in the `c++::` namespace are normal types: they use the normal
layout policies, and the backing C++ type is derived from the `c++::`
segments (e.g. `c++::osmium::Way` means the C++ type `::osmium::Way`):

```
type c++::osmium::Way {
    #only_by_ref;
}
```

`#only_by_ref` declares a reference-only type, `#heap_allocated` a heap
allocated type, and `#layout` / `#layout_conservative` a stack allocated type.

## C++ template types

A `c++::` path can carry template arguments, so each concrete instantiation
is declared as its own type with its own layout:

```
type c++::std::unique_ptr<c++::CppType> {
    #layout(size = 8, align = 8);
}
```

On the C++ side this names the real instantiation (`::std::unique_ptr<::CppType>`)
everywhere: method signatures, `Impl` specializations, `.cpp()`. On the Rust
side the template becomes one generic struct per template, bounded by a
generated trait, with the storage plus trait impl per instantiation:

```rust
pub struct unique_ptr<T: unique_ptrSpecializations>(T::Data);

impl unique_ptrSpecializations for CppType {
    type Data = unique_ptr_CppType_Data; // 8 bytes, align 8, from `#layout`
}
```

so Rust code writes `unique_ptr<CppType>::make_unique(1, 2)`, while the
`extern "C"` symbols and the C++ side keep working with the real
instantiation. `Drop`, `ZngCppDestruct`, and the object markers are
implemented once, generically; each instantiation only contributes its
sized-and-aligned `Data` buffer and the destructor hook that runs it.

Three consequences follow from that:

- Every instantiation needs its own `type` block, because sizes, methods,
  and the trivial-relocatability opt-in below are per-instantiation.
  (`unique_ptr` is pointer-sized for every `T`, but that is not true of
  templates in general.) Each block generates its own `Data` plus trait
  impl; the generic struct and trait are shared.
- `zngur` never forward-declares a `c++::` type: the definition always comes
  from your own headers, so `#include` them (e.g. `<memory>`) via
  `#cpp_additional_includes` or your own header. Two different
  instantiations that would share one `Data` name are rejected instead of
  silently merged.
- Nested templates like `shared_ptr<vector<i32>>` work the same way, but
  every level must be declared: the argument `vector<i32>` needs its own
  `type` block with its own layout, so the Rust path `vector<i32>` resolves
  and the trait impl has somewhere to hang its `Data`.

## Template layouts

`Data` is whatever the layout needs the instantiation to own inline, and
the shared `Drop` destroys it through the trait's hook exactly once:

- `#layout` / `#layout_conservative`: `Data` is a sized-and-aligned byte
  buffer holding the object itself.
- `#heap_allocated`: `Data` is the heap box (pointer plus stored
  destructor). The constructor returns the object by value and `zngur`
  boxes it, exactly like a non-template heap type.
- `#only_by_ref`: `Data` is `()`. Nothing is owned and destruction is a
  no-op; the handle only travels back into C++ (raw pointers spell as
  plain C++ pointers, `Ref` exposes `.cpp()`).

One template serves every layout at once: only stack instantiations get
the object markers, while `Drop` stays generic over the hook.

## Mixed templates

Both directions mix freely. A C++ template can hold a Rust type, naming
the generated mirror on the C++ side:

```
mod crate {
    type RustPoint {
        #layout(size = 8, align = 4);
        constructor(i32, i32);
    }
}

type c++::Holder<crate::RustPoint> {
    #layout(size = 12, align = 4);
}
```

This works like any other stack object as long as the Rust type stays
plain data: relocating it is a memcpy, and the mirror's move constructor
transfers the drop flag.

`Box<T>` mixes too, in both directions. `Box<dyn Trait>` gets its
definition automatically; `Box` over a concrete type is declared with the
`template_types` shorthand, where the template supplies the layout and
each named instantiation inherits it:

```
mod ::std {
    type<T> boxed::Box<T> {
        #layout(size = 8, align = 8);
    }
}

type ::std::boxed::Box<crate::RustPoint> {
}

type c++::std::vector<::std::boxed::Box<crate::RustPoint>> {
    #layout(size = 24, align = 8);
}
```

On the C++ side the box spells `rust::std::boxed::Box<...>`, built and
consumed by value with the mirror protocol; on the Rust side it is the
real `Box`, constructed with `Box::new` and read through `Deref`.

Conversely, a Rust template instantiates over a C++ type, with the real
methods moving the instantiation's bytes:

```
mod ::std {
    type<T> vec::Vec<T> {
        #layout(size = 24, align = 8);
        fn new() -> vec::Vec<T>;
        fn push(&mut self, T);
        fn len(&self) -> usize;
    }
}

type ::std::vec::Vec<c++::Holder<c++::CppType>> {
}
```

See `examples/cpp_templates/` for the full working example: `vector<i32>`
and nested `shared_ptr<vector<i32>>` on the stack, a heap-allocated
`Holder<CppType>`, a borrowed `Holder<i32>`, a `Holder<RustPoint>`, a Rust
`Vec` of `Holder<CppType>`, and a C++ `vector` of Rust `Box`es.

## Opaque Borrowed C++ Type

For example, you define a reference-only opaque type in `main.zng` as follows
(the backing C++ type is derived from the `c++::` path, here `::osmium::Way`):

```
type c++::osmium::Way {
    #only_by_ref;
}
```

Note that `#only_by_ref` types don't need manual layout policy.
This enables creating `rust::Ref<::osmium::Way>` from a `const ::osmium::Way&` in C++
and you can pass it to the Rust side.
Rust side can't do anything meaningful with it, except passing it again to the C++ side.
In the C++ side `rust::Ref<::osmium::Way>` has a `.cpp()` method
which will return the `osmium::Way&` back to you.
If you want to use the methods on your C++ type in the Rust side,
you can write `impl` and `impl trait` blocks for the C++ type `::osmium::Way` inside C++.
See the [`examples/osmium`](https://github.com/HKalbasi/zngur/blob/main/examples/osmium) for a full working example.

### Semantics of Opaque Borrowed Types

`#only_by_ref` types are implemented as ZSTs that can only be obtained by reference.
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

Keeping C++ objects in Rust using heap allocation is supported with `#heap_allocated` types,
with the C++ path derived from the `c++::` path:

```
type c++::cpp_inventory::Inventory {
    #heap_allocated;
}
```

Look at the example `tutorial_cpp` for a full usage example.

### Semantics of Opaque Heap Allocated Types

Heap allocated types are generated as a #[repr(C)] struct holding a heap allocated pointer and destructor.

At construction, your `Impl` method returns the real C++ object by value and
`zngur` heap-boxes it natively (`new` plus a move); there is no `rust::`
mirror wrapper on the C++ side.
The only thing you can do with that object is call C++ methods or `Drop` it
which calls the destructor and frees up the allocation

> **NOTE**: In the future, we may replace C++ owned objects with a
> `unique_ptr`-like type that wraps a cpp_ref instead.

### The C++ side: no mirror types

For all three kinds, the C++ side works directly with the real C++ type:
no `rust::` mirror wrapper class is generated. `Ref`/`RefMut` handles and
`rust::Impl` specializations name the real type (e.g.
`rust::Impl<::cpp_stats::StatsAccumulator>`), and `.cpp()` on a handle
returns a reference to the real object.

This imposes three rules on the C++ types:

- Values crossing to Rust by value are move-constructed (heap types), or
  move/copy-constructed in place (stack types). The type must support that;
  `zngur` never `memcpy`s a C++ object it doesn't own the layout of.
- A value returned to Rust by value is destroyed once in transit (the C++
  temporary) and once by Rust `Drop`. Keep destructors correct under that:
  each construction is still destroyed exactly once, but you will observe
  the transit destruction (e.g. an extra destructor print in test output).
- C++-held objects carry no use-after-move guard. Misuse is ordinary C++
  undefined behavior, not a loud `SIGSEGV` like `rust::`-owned wrappers.

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

type c++::CppType {
    #layout(size = 8, align = 4);
}
```

Similarly to how we can define opaque C++ objects with `#only_by_ref` and `#heap_allocated`,
a `c++::` type with `#layout` is stored directly in the Rust stack, with the backing C++ type
derived from the `c++::` path (here `::CppType`). The memory layout is specified separately
using the `#layout` or `#layout_conservative` directives.

#### Conservative Layout

When exact type size or alignment is not known ahead of time (e.g. across platforms or compiler
versions), you can specify an upper bound using `#layout_conservative`:

```zng
type c++::CppConservativeType {
    #layout_conservative(size = 32, align = 16);
}
```

With `#layout_conservative`, `zngur` generates compile-time static assertions ensuring that
`sizeof(T) <= size` and `alignof(T) <= align`, while allocating the specified upper-bound size and
alignment in the Rust wrapper.

Just like with C++ opaque objects, we can define functions on an `extern C++` block

```zng
extern "C++" {
    fn create_cpp_type(i32, i32) -> c++::CppType;
    fn print_cpp_type(&c++::CppType);
}
```

The C++ code has access to the `.cpp()` method used to access the inner type.

```rust
// main.rs
mod generated;

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
