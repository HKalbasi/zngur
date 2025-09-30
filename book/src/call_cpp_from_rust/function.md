# Calling C++ free functions

You can call C++ functions that operate on Rust types.
First, you need to add that function signature in the `main.zng` file inside an `extern "C++"` block:

```Rust
extern "C++" {
    fn new_blob_store_client(crate::Flags) -> crate::Reader;
}
```

Then, in the `generated.h` file, the function signature is declared like this:

```C++
namespace rust {
namespace exported_functions {
::rust::crate::Reader new_blob_store_client(::rust::crate::Flags i0);
}
} // namespace rust
```

You need to implement it in a `.cpp` file and link it to the final binary.
