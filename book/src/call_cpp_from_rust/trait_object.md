# Creating trait objects from C++ types

In addition to opaque C++ objects, you can convert your C++ types into `&dyn Trait` or `Box<dyn Trait>`.
You could do that with opaque types, but you needed to:

- Create an opaque borrowed (or owned for `Box<dyn Trait>`) type for the C++ type.
- Implement the `Trait` for that type inside C++.
- Cast `&Opaque` to `&dyn Trait` when needed.

This feature is a shortcut for that use case.
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
