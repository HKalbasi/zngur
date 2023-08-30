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
`drop_flag`, which is `0` if the value doesn't need drop (it is uninitialized or moved out from) and otherwise `1`. So a C++ wrapper
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
