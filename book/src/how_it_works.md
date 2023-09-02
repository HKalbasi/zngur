# How it works?

Rust, as a language, only can talk with other languages in C. So special care should
be taken in transferring Rust types between Rust and C++.

## How a Rust type is represented in C++

A normal rust type ABI is undefined, so passing it directly in a cross language function call is undefined behavior. The
thing that is guaranteed is that the size and alignment of a type won't change during a compile session. By adding static assertions
against the user provided size and align in the `main.zng` file, Zngur ensures that it knows the correct size and align of the
type for this compile session. Knowing the size and align of a type enables `std::ptr::read` and `std::ptr::write`. These functions
only need the pointer to be valid (which basically means `ptr..ptr+size` should belong to a single live chunk of memory,
[read more](https://doc.rust-lang.org/std/ptr/index.html#safety)) and
aligned. So Zngur can use the pointer to the below `data` in those functions:

```C++
alignas(align_value) ::std::array<uint8_t, size_value> data;
```

To support running destructors of Rust types in C++, Zngur uses `std::ptr::drop_in_place` which has similar constraints to `read` and
`write`. But to prevent double free, Zngur needs to track if a Rust type is moved out. It does this using a boolean field called
`drop_flag`, which is `false` if the value doesn't need drop (it is uninitialized or moved out from) and otherwise `true`. So a C++ wrapper
for a typical Rust type will look like this:

```C++
struct MultiBuf {
private:
  alignas(8)::std::array<uint8_t, 32> data;
  bool drop_flag;

public:
  MultiBuf() : drop_flag(false) {}
  ~MultiBuf() {
    if (drop_flag) {
      __zngur_crate_MultiBuf_drop_in_place_s13e22(&data[0]);
    }
  }
  MultiBuf(const MultiBuf &other) = delete;
  MultiBuf &operator=(const MultiBuf &other) = delete;
  MultiBuf(MultiBuf &&other) : data(other.data), drop_flag(true) {
    if (!other.drop_flag) {
      ::std::terminate();
    }
    other.drop_flag = false;
  }
};
```

Note that the drop flag [also exists in Rust](https://doc.rust-lang.org/stable/nomicon/drop-flags.html). It is not stored inside
the type, but in the stack of the owner, and the compiler generates them only if necessary.

## Calling Rust functions from C++

For exposing a function or method from Rust to C++, an `extern "C"` function is generated that takes all arguments as `*mut u8`, and
takes output as an output parameter `o: *mut u8`. It then reads arguments using `ptr::read`, calls the underlying function, and write
the result in `o` using `ptr::write`. So for example for `Option<i32>::unwrap` some code like this will be generated:

```Rust
#[no_mangle]
pub extern "C" fn __zngur___std_option_Option_i32__unwrap___x8s9s13s20m27y31n32m39y40(
    i0: *mut u8,
    o: *mut u8,
) {
    unsafe {
        ::std::ptr::write(
            o as *mut i32,
            <::std::option::Option<i32>>::unwrap(::std::ptr::read(
                i0 as *mut ::std::option::Option<i32>,
            )),
        )
    }
}
```

In the C++ side, this code will be generated for that function:

```C++
::rust::std::string::String rust::rustyline::Result<::rust::std::string::String>::unwrap(::rust::rustyline::Result<::rust::std::string::String> i0)
{
    ::rust::std::string::String o;
    ::rust::__zngur_internal_assume_init(o);
    __zngur___rustyline_Result__std_string_String__unwrap___x8s9s19m26s27s31s38y45n46m53y54(::rust::__zngur_internal_data_ptr(i0), ::rust::__zngur_internal_data_ptr(o));
    ::rust::__zngur_internal_assume_deinit(i0);
    return o;
}
```

`::rust::std::string::String o;` creates an uninitialized `String`. `__zngur_internal_assume_init` sets its drop flag to `true` so that it will become
freed after being returned by this function. Then it will call the underlying Rust function, and by `__zngur_internal_assume_deinit` it will ensure
that the destructor for `i0` is not called. `i0` is now semantically moved in Rust, and it's Rust responsibility to destruct it.

## Calling C++ functions from Rust

Similarly, for exposing a C++ function to Rust, a function will be generated that takes all inputs and output by `uint8_t*`.

```C++
extern "C" {
void __zngur_new_blob_store_client_(uint8_t *o) {
  ::rust::Box<::rust::Dyn<::rust::crate::BlobStoreTrait>> oo =
      ::rust::exported_functions::new_blob_store_client();
  ::rust::__zngur_internal_move_to_rust(o, oo);
}
}
```

Where `::rust::__zngur_internal_move_to_rust` is just this function:

```C++
template <typename T>
inline void __zngur_internal_move_to_rust(uint8_t *dst, T &t) {
  {
    memcpy(dst, ::rust::__zngur_internal_data_ptr(t),
           ::rust::__zngur_internal_size_of<T>());
    ::rust::__zngur_internal_assume_deinit(t);
  }
}
```

And that function is called in Rust by a function like this:

```Rust
pub(crate) fn new_blob_store_client() -> Box<dyn crate::BlobStoreTrait> {
    unsafe {
        let mut r = ::core::mem::MaybeUninit::uninit();
        __zngur_new_blob_store_client_(r.as_mut_ptr() as *mut u8);
        r.assume_init()
    }
}
```

This could be a free function like the above example, a function in an inherent impl block, or a trait impl block. All of them
are implemented in this way.

## Implementing Rust traits for C++ classes

C++ types can't exist in Rust by value, since it might need a nontrivial move constructor incompatible with Rust moves. So for representing
them in Rust, Zngur uses the following struct:

```Rust
struct ZngurCppOpaqueObject {
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
}

impl Drop for ZngurCppOpaqueObject {
    fn drop(&mut self) {
        (self.destructor)(self.data)
    }
}
```

Where `data` is a `new`ed pointer in C++, and `destructor` is a function pointer that can `delete` that data, i.e. `[](uint8_t *d) { delete (T *)d; }`. It's
basically a type erased `unique_ptr`.

For converting a C++ class into a `Box<dyn Trait>`, Zngur generates a code like this in the Rust side:

```Rust
#[no_mangle]
pub extern "C" fn __zngur_crate_BlobStoreTrait_s13(
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
    f_put: extern "C" fn(data: *mut u8, i0: *mut u8, o: *mut u8),
    f_tag: extern "C" fn(data: *mut u8, i0: *mut u8, i1: *mut u8, o: *mut u8),
    f_metadata: extern "C" fn(data: *mut u8, i0: *mut u8, o: *mut u8),
    o: *mut u8,
) {
    struct Wrapper {
        value: ZngurCppOpaqueObject,
        f_put: extern "C" fn(data: *mut u8, i0: *mut u8, o: *mut u8),
        f_tag: extern "C" fn(data: *mut u8, i0: *mut u8, i1: *mut u8, o: *mut u8),
        f_metadata: extern "C" fn(data: *mut u8, i0: *mut u8, o: *mut u8),
    }
    impl crate::BlobStoreTrait for Wrapper {
        fn put(&self, i0: &mut crate::MultiBuf) -> u64 {
            unsafe {
                let data = self.value.data;
                let mut i0 = ::core::mem::MaybeUninit::new(i0);
                let mut r = ::core::mem::MaybeUninit::uninit();
                (self.f_put)(data, i0.as_mut_ptr() as *mut u8, r.as_mut_ptr() as *mut u8);
                r.assume_init()
            }
        }
        fn tag(&self, i0: u64, i1: &::core::primitive::str) -> () {
            unsafe {
                let data = self.value.data;
                let mut i0 = ::core::mem::MaybeUninit::new(i0);
                let mut i1 = ::core::mem::MaybeUninit::new(i1);
                let mut r = ::core::mem::MaybeUninit::uninit();
                (self.f_tag)(
                    data,
                    i0.as_mut_ptr() as *mut u8,
                    i1.as_mut_ptr() as *mut u8,
                    r.as_mut_ptr() as *mut u8,
                );
                r.assume_init()
            }
        }
        fn metadata(&self, i0: u64) -> crate::BlobMetadata {
            unsafe {
                let data = self.value.data;
                let mut i0 = ::core::mem::MaybeUninit::new(i0);
                let mut r = ::core::mem::MaybeUninit::uninit();
                (self.f_metadata)(data, i0.as_mut_ptr() as *mut u8, r.as_mut_ptr() as *mut u8);
                r.assume_init()
            }
        }
    }
    let this = Wrapper {
        value: ZngurCppOpaqueObject { data, destructor },
        f_put,
        f_tag,
        f_metadata,
    };
    let r: Box<dyn crate::BlobStoreTrait> = Box::new(this);
    unsafe { std::ptr::write(o as *mut _, r) }
}
```

Which constructs a `Wrapper` around `ZngurCppOpaqueObject` which also contains the function pointers to the method implementation, and
implements the trait for it. Inside of each trait function is very similar to a normal `C++` function used in Rust and contains the
similar `MaybeUninit`s.

Using that, `make_box` can be defined:

```C++
template <typename T, typename... Args> static Box make_box(Args &&...args) {
  auto data = new T(::std::forward<Args>(args)...);
  Box o;
  ::rust::__zngur_internal_assume_init(o);
  __zngur_crate_BlobStoreTrait_s13(
      (uint8_t *)data, [](uint8_t *d) { delete (T *)d; },

      [](uint8_t *d, uint8_t *i0, uint8_t *o) {
        T *dd = (T *)d;
        ::rust::Ref<::rust::crate::MultiBuf> ii0;
        ::rust::__zngur_internal_assume_init(ii0);
        memcpy(::rust::__zngur_internal_data_ptr(ii0), i0,
                ::rust::__zngur_internal_size_of<
                    ::rust::Ref<::rust::crate::MultiBuf>>());
        ::uint64_t oo = dd->put(ii0);
        ::rust::__zngur_internal_move_to_rust(o, oo);
      },

      [](uint8_t *d, uint8_t *i0, uint8_t *i1, uint8_t *o) {
        T *dd = (T *)d;
        ::uint64_t ii0;
        ::rust::__zngur_internal_assume_init(ii0);
        memcpy(::rust::__zngur_internal_data_ptr(ii0), i0,
                ::rust::__zngur_internal_size_of<::uint64_t>());
        ::rust::Ref<::rust::core::primitive::str> ii1;
        ::rust::__zngur_internal_assume_init(ii1);
        memcpy(::rust::__zngur_internal_data_ptr(ii1), i1,
                ::rust::__zngur_internal_size_of<
                    ::rust::Ref<::rust::core::primitive::str>>());
        ::rust::Unit oo = dd->tag(ii0, ii1);
        ::rust::__zngur_internal_move_to_rust(o, oo);
      },

      [](uint8_t *d, uint8_t *i0, uint8_t *o) {
        T *dd = (T *)d;
        ::uint64_t ii0;
        ::rust::__zngur_internal_assume_init(ii0);
        memcpy(::rust::__zngur_internal_data_ptr(ii0), i0,
                ::rust::__zngur_internal_size_of<::uint64_t>());
        ::rust::crate::BlobMetadata oo = dd->metadata(ii0);
        ::rust::__zngur_internal_move_to_rust(o, oo);
      },

      ::rust::__zngur_internal_data_ptr(o));
  return o;
}
```
