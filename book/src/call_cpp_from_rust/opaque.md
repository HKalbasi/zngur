# Opaque C++ types

## Opaque borrowed C++ type

The generated Rust code contains a `ZngurCppOpaqueBorrowedObject`
which you can use to create opaque Rust equivalent for C++ types.
To do that, you first need to create a newtype wrapper around it in Rust:

```Rust
struct Way(generated::ZngurCppOpaqueBorrowedObject);
```

Then you need to add this type to your `main.zng` as a `#cpp_ref`:

```
type crate::Way {
    #cpp_ref "::osmium::Way";
}
```

Note that `#cpp_ref` types don't need manual layout policy.
This enables creating `rust::Ref<rust::crate::Way>` from a `const osmium::Way&` in C++
and you can pass it to the Rust side.
Rust side can't do anything meaningful with it, except passing it again to the C++ side.
In the C++ side `rust::Ref<rust::crate::Way>` has a `.cpp()` method
which will return the `osmium::Way&` back to you.
If you want to use the methods on your C++ type in the Rust side,
you can write `impl` and `impl trait` blocks for the newtype wrapper `crate::Way` inside C++.
See the [`examples/osmium`](https://github.com/HKalbasi/zngur/blob/main/examples/osmium) for a full working example.

## Opaque owned C++ type

Owning C++ types on the Rust stack is impossible without a huge amount of acrobatics,
since Rust assumes that every type is memcpy-movable,
which doesn't work for C++ types with non-trivial move constructors.
It is not entirely impossible,
for example the `moveit` crate achieves it by hiding the binding of the stack owner in a macro:

```Rust
moveit! {
    let mut stack_obj = ffi::A::new();
}
// here, type of `stack_obj` is `Pin<MoveRef<ffi::A>>`, not `ffi::A` itself.
stack_obj.as_mut().set(42);
assert_eq!(stack_obj.get(), 42);
```

Keeping the Rust side clean and idiomatic is one of the design goals of Zngur,
and such a macro is not clean and idiomatic Rust.
So, storing C++ objects on the Rust stack is not supported.
If you really need to store things on the Rust stack,
consider moving the type definition into Rust.

Keeping C++ objects in Rust using heap allocation
is supported with `ZngurCppOpaqueOwnedObject`.

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

## Semantics of the opaque types

The `ZngurCppOpaqueBorrowedObject` and newtype wrappers around it don't represent a C++ object,
but they represent an imaginary ZST Rust object at the first byte of a C++ object.
This can sometimes cause behavior that is safe and sound,
but surprising and counterintuitive for someone that expects them to represent the whole C++ object.
Some examples (assume `RustType` is a newtype wrapper around `ZngurCppOpaqueBorrowedObject`
that refers to a `CppType` class in the C++):

- `std::mem::sizeof::<RustType>()` is 0, not the size of `CppType`
- `std::mem::alignof::<RustType>()` is 1, not the align of `CppType`
- `std::mem::swap::<RustType>(a, b)` only swaps the first zero bytes of those, i.e. does nothing.

Those problem might be solved by the `extern type` language feature.
Using that, we can define the `ZngurCppOpaqueBorrowedObject` as a proper extern type
instead of an imaginary zero sized type at the first of C++ objects.
