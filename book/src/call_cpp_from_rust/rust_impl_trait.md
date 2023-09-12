# Implementing Rust traits for Rust types

You can write `impl Trait for Type` blocks for type defined in your crate (this is not a Zngur restriction, it's the orphan rule) in C++. First, you need to
declare the functions of that block in the `main.zng` file:

```Rust
extern "C++" {
    impl ::std::ops::Index<usize, Output = crate::Node> for crate::WayNodeList {
        fn index(&self, usize) -> &crate::Node;
    }
}
```

Then, some class like this will be generated in the `generated.h` file:

```C++
namespace rust {
template <>
class Impl<::rust::crate::WayNodeList,
           ::rust::std::ops::Index<::size_t, ::rust::crate::Node>> {
public:
  static ::rust::Ref<::rust::crate::Node>
  index(::rust::Ref<::rust::crate::WayNodeList> self, ::size_t i0);
};
}
```

And you need to implement that in a `.cpp` file, and link it to the crate:

```C++
rust::Ref<Node>
rust::Impl<WayNodeList, rust::std::ops::Index<size_t, Node>>::index(
    rust::Ref<WayNodeList> self, size_t i) {
  // Your code here
}
```
