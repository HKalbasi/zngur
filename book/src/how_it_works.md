# How it works?

Rust, as a language, only can talk with other languages in C. So special care should
be taken in transferring Rust types between Rust and C++.

## How a Rust type is represented in C++

A normal rust type ABI is undefined, so passing it directly in a cross language function call is undefined behavior. The
thing that is guaranteed, is that size and align of a type won't change during a compile session. By adding static assertions
against the user provided size and align in the `main.zng` file, Zngur ensures that it knows the correct size and align of the
type for this compile session. Knowing size and align of a type enables `std::ptr::read` and `std::ptr::write`. These functions
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
      // TODO: call drop in place glue code
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

Note that drop flag [also exists in Rust](https://doc.rust-lang.org/stable/nomicon/drop-flags.html). It is not stored inside
the type, but in the stack of the owner, and compiler generate them only if necessary.

## Calling Rust functions from C++

For exposing a function or method from Rust to C++, an `extern "C"` function is generated that takes all arguments as `*mut u8`, and
takes output as an output parameter `o: *mut u8`. It then read arguments using `ptr::read`, calls the underlying function, and write
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

`::rust::std::string::String o;` creates an uninitialized `String`. `__zngur_internal_assume_init` sets it's drop flag to `true` so that it will become
freed after being returned by this function. Then it will call the underlying Rust function, and by `__zngur_internal_assume_deinit` it will ensure
that the destructor for `i0` is not called. `i0` is now semantically moved in Rust, and it's Rust responsibility to destruct it.

## Calling C++ functions from Rust

Similarly, for exposing a C++ to Rust, a function will be generated that take all inputs and output by `uint8_t*`.

```C++
222
```
