# Generated Code Structure

Zngur generates both Rust and C++ files from a `.zng` interface definition file. There are two generation modes for the C++ side:

## Generation Modes

### Split-Header Mode (Default)

By default, Zngur generates one Rust file and three C++ files:

**Rust:**

1. **`generated.rs`** - Rust bridge code and FFI declarations

**C++:**

1. **`zngur.h`** - Foundation header with core utilities
2. **`generated.h`** - Main header with Rust type wrappers and inline functions
3. **`generated.cpp`** - Implementation file with method bodies and C++ bridge code

This mode provides better modularity and allows `zngur.h` to be shared across multiple independent Zngur-generated libraries.

**Usage:**

```bash
zngur g main.zng -o output_dir
```

The `-o` flag specifies where `zngur.h` should be generated. You must add this directory to your C++ include path (e.g., with `-I` flag).

### Single-Header Mode

For backward compatibility and simpler build setups, you can use the `--single-header` flag to merge `zngur.h` into `generated.h`:

**Usage:**

```bash
zngur g main.zng --single-header
```

This generates one Rust file and two C++ files:

**Rust:**

1. **`generated.rs`** - Rust bridge code and FFI declarations

**C++:**

1. **`generated.h`** - Contains both foundation utilities and type wrappers
2. **`generated.cpp`** - Implementation file

This mode emulates the old behavior of Zngur before the header split and is useful for simple projects or when migrating from older versions.

## File Structure

The rest of this chapter describes the generated files. For C++, split-header mode is described; in single-header mode, the content of `zngur.h` is merged directly into `generated.h`.

## 1. `generated.rs` - Rust Bridge Code

**Purpose**: Contains Rust-side FFI bridge code that enables C++ to call Rust functions and implement Rust traits.

This file should be included in your Rust project with `mod generated;`.

### Opaque Type Definitions

The file begins with helper types for representing C++ objects in Rust:

```rust
pub struct ZngurCppOpaqueOwnedObject {
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
}

pub struct ZngurCppOpaqueBorrowedObject(());
```

- `ZngurCppOpaqueOwnedObject` wraps a C++ object with ownership, calling the destructor when dropped
- `ZngurCppOpaqueBorrowedObject` represents a borrowed C++ object (zero-sized marker type)

### Static Assertions

For each Rust type declared in the `.zng` file, generates compile-time size and alignment checks:

```rust
const _: [(); SIZE] = [(); ::std::mem::size_of::<Type>()];
const _: [(); ALIGN] = [(); ::std::mem::align_of::<Type>()];
```

These ensure that the sizes specified in `#layout(size=X, align=Y)` match the actual Rust type sizes.
If there's a mismatch, compilation fails with a clear error.

For Copy types, an additional assertion verifies the `Copy` trait:

```rust
const _: () = {
    const fn static_assert_is_copy<T: Copy>() {}
    static_assert_is_copy::<Type>();
};
```

### Function Bridge Implementations

For each Rust function and method declared in the `.zng` file, generates an `extern "C"` wrapper:

```rust
#[no_mangle]
pub extern "C" fn _zngur_mangled_name(i0: *mut u8, i1: *mut u8, o: *mut u8) {
    unsafe {
        let arg0 = std::ptr::read(i0 as *mut ArgType0);
        let arg1 = std::ptr::read(i1 as *mut ArgType1);
        let result = rust_function(arg0, arg1);
        std::ptr::write(o as *mut _, result);
    }
}
```

These functions:

- Use mangled names to avoid symbol conflicts
- Accept arguments and return value as raw `*mut u8` pointers
- Use `ptr::read` to move values from C++ to Rust
- Use `ptr::write` to move the result back to C++
- Are marked `#[no_mangle]` so C++ can link to them

### Drop in Place Functions

For non-Copy types, generates destructor bridge functions:

```rust
#[no_mangle]
pub extern "C" fn _zngur_Type_drop_in_place(v: *mut u8) {
    unsafe {
        ::std::ptr::drop_in_place(v as *mut Type);
    }
}
```

These allow C++ destructors to properly drop Rust values by calling Rust's `drop_in_place`.

### Trait Object Builders

For `Box<dyn Trait>` types declared in `extern "C++"` blocks, generates trait implementations that wrap C++ objects:

```rust
#[no_mangle]
pub extern "C" fn _zngur_build_trait_object(
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
    o: *mut u8,
) {
    struct Wrapper {
        value: ZngurCppOpaqueOwnedObject,
    }

    impl Trait for Wrapper {
        fn method(&mut self, arg: ArgType) -> ReturnType {
            unsafe {
                let data = self.value.ptr();
                // Call extern "C" function that forwards to C++ method
                _zngur_trait_method_call(data, &arg as *const _ as *mut u8, ...);
                ...
            }
        }
    }

    unsafe {
        let wrapper = Wrapper {
            value: ZngurCppOpaqueOwnedObject::new(data, destructor),
        };
        let trait_object: Box<dyn Trait> = Box::new(wrapper);
        std::ptr::write(o as *mut _, trait_object);
    }
}
```

This enables C++ classes to implement Rust traits and be used as `Box<dyn Trait>` in Rust.

### Closure Builders

For `Box<dyn Fn(...)>` types, generates functions that wrap C++ lambdas:

```rust
#[no_mangle]
pub extern "C" fn _zngur_build_closure(
    data: *mut u8,
    destructor: extern "C" fn(*mut u8),
    call: extern "C" fn(data: *mut u8, args..., o: *mut u8),
    o: *mut u8,
) {
    let cpp_object = unsafe { ZngurCppOpaqueOwnedObject::new(data, destructor) };
    let closure: Box<dyn Fn(Args...) -> Output> = Box::new(move |args...| unsafe {
        let data = cpp_object.ptr();
        // Marshal arguments and call C++ function pointer
        call(data, ...);
        ...
    });
    unsafe { std::ptr::write(o as *mut _, closure) }
}
```

This allows C++ `std::function` and lambdas to be converted to Rust closures.

### Panic Handling (Optional)

If `#convert_panic_to_exception` is enabled in the `.zng` file, generates panic detection functions:

```rust
thread_local! {
    pub static PANIC_PAYLOAD: ::std::cell::Cell<Option<()>> =
        ::std::cell::Cell::new(None);
}

#[no_mangle]
pub fn _zngur_detect_panic() -> u8 {
    PANIC_PAYLOAD.with(|p| {
        if p.get().is_some() { 1 } else { 0 }
    })
}

#[no_mangle]
pub fn _zngur_take_panic() {
    PANIC_PAYLOAD.with(|p| p.take());
}
```

These functions allow C++ to detect when a Rust panic occurred and convert it to a C++ exception.

### Exported Function Declarations

For functions declared in `extern "C++"` blocks, generates `extern "C"` declarations:

```rust
extern "C" {
    fn _zngur_cpp_function(args..., o: *mut u8);
}
```

These are implemented on the C++ side and called from Rust.

## 2. `zngur.h` - C++ Foundation Header

**Purpose**: Provides the foundational infrastructure needed by all generated C++ code.
This is a reusable header that doesn't depend on the specific `.zng` file.

### Standard Includes

The header includes necessary C++ standard library headers:
`<cstddef>`, `<cstdint>`, `<cstring>`, `<csignal>`, `<array>`, `<iostream>`, `<functional>`, `<math.h>`

It also includes any user-specified additional includes via `#cpp_additional_includes` in the `.zng` file. These additional includes are placed in `zngur.h` (split-header mode) or at the top of the merged `generated.h` (single-header mode).

### Debug Macro

```cpp
#define zngur_dbg(x) (::rust::zngur_dbg_impl(__FILE__, __LINE__, #x, x))
```

This provides a C++ equivalent of Rust's `dbg!` macro for inspecting values during debugging.

### Core Templates and Utilities

All core functionality lives in the `rust` namespace:

#### Exception Type

```cpp
class Panic {};
```

Used for catching Rust panics as C++ exceptions (when `#convert_panic_to_exception` is enabled).

#### Internal Data Management Templates

These template functions manage the lifecycle and memory of Rust values in C++:

- `__zngur_internal_data_ptr<T>()` - Gets pointer to the underlying data buffer
- `__zngur_internal_assume_init<T>()` - Marks a value as initialized (sets drop flag)
- `__zngur_internal_assume_deinit<T>()` - Marks a value as moved/uninitialized (clears drop flag)
- `__zngur_internal_size_of<T>()` - Gets the size of a type
- `__zngur_internal_check_init<T>()` - Validates initialization state
- `__zngur_internal_move_to_rust()` - Moves a C++ value to Rust
- `__zngur_internal_move_from_rust()` - Moves a Rust value to C++

#### ZngurCppOpaqueOwnedObject

A type-erased owned pointer for C++ objects that need to be called from Rust.
This is essentially a type-erased `unique_ptr`:

```cpp
class ZngurCppOpaqueOwnedObject {
    uint8_t* data;
    void (*destructor)(uint8_t*);

public:
    template<typename T, typename... Args>
    inline static ZngurCppOpaqueOwnedObject build(Args&&... args);

    template<typename T>
    inline T& as_cpp();
};
```

#### Reference Types

Templates for representing Rust references in C++:

- `template<typename T> struct Ref<T>` - Immutable reference (`&T`)
- `template<typename T> struct RefMut<T>` - Mutable reference (`&mut T`)
- `template<typename T> struct Raw<T>` - Raw pointer (`*const T`)
- `template<typename T> struct RawMut<T>` - Mutable raw pointer (`*mut T`)

#### Field Access Types

Templates for accessing fields of Rust types:

- `template<typename T, size_t OFFSET> struct FieldOwned<T, OFFSET>` - Access to an owned field
- `template<typename T, size_t OFFSET> struct FieldRef<T, OFFSET>` - Shared reference to a field
- `template<typename T, size_t OFFSET> struct FieldRefMut<T, OFFSET>` - Mutable reference to a field

These enable zero-cost field access by encoding the field offset as a template parameter.

#### Trait Support

```cpp
class Inherent;

template<typename Type, typename Trait = Inherent>
class Impl;
```

The `Impl` template is specialized for each C++ trait implementation written in `extern "C++"` blocks.

#### Unit Type

```cpp
template<> struct Tuple<> { ... };
using Unit = Tuple<>;
```

Represents Rust's unit type `()`.

#### Builtin Type Specializations

Complete specializations of all internal templates for primitive types:
`int8_t`, `uint8_t`, `int16_t`, `uint16_t`, `int32_t`, `uint32_t`, `int64_t`, `uint64_t`,
`float`, `double`, `size_t`, and pointer types.

These specializations provide `Ref<T>` and `RefMut<T>` wrappers and integrate with the
internal template system.

## 3. `generated.h` - C++ Main Generated Header

**Purpose**: Contains C++ wrappers for all Rust types, functions, and traits declared in the `.zng` file.

### Header Structure

#### Includes zngur.h (Split-Header Mode Only)

In split-header mode, `generated.h` includes the foundation header:

```cpp
#include <zngur.h>
```

In single-header mode, this include is omitted and the content of `zngur.h` is merged directly into `generated.h`.

#### Panic Detection Functions

If `#convert_panic_to_exception` is enabled in the `.zng` file, generates functions for
detecting and handling Rust panics:

```cpp
extern "C" {
    uint8_t detect_panic();
    void take_panic();
}
```

#### Extern "C" Function Declarations

For every Rust function and method declared in `.zng`, generates an `extern "C"` declaration
with a mangled name:

```cpp
extern "C" {
    void __zngur_[mangled_name](uint8_t* i0, uint8_t* i1, ..., uint8_t* o) noexcept;
}
```

These declarations correspond to the functions generated in the Rust code and provide the
bridge between C++ and Rust.

### Per-Type Code Generation

For each `type` declared in the `.zng` file, Zngur generates comprehensive C++ wrapper code:

#### A. Forward Declarations

Opens appropriate C++ namespaces matching Rust's module structure:

```cpp
namespace rust {
namespace std {
namespace vec {
    template<typename T> class Vec;
}}}
```

#### B. Trait Definitions

For each Rust trait exposed in the `.zng` file, generates an abstract base class:

```cpp
namespace rust::std::iter {
    template<typename Item>
    class Iterator {
    public:
        virtual ~Iterator() {};
        virtual Option<Item> next() = 0;
    };
}
```

C++ classes can inherit from these to implement Rust traits.

#### C. Main Type Definition

For each type, generates a C++ class or struct with the following components:

##### 1. Private Data Storage

The storage strategy depends on the layout policy specified in the `.zng` file:

**Stack-allocated types** (`#layout(size=X, align=Y)`):

```cpp
alignas(Y) mutable ::std::array<uint8_t, X> data;
```

The `mutable` keyword allows modifying `const` objects (equivalent to Rust's `UnsafeCell`),
which is necessary for interior mutability.

**Heap-allocated types** (custom allocators):

```cpp
uint8_t* data;  // Allocated via Rust-provided allocator functions
```

**Only-by-ref types** (`#only_by_ref`):

```cpp
// No data storage - these types can only be used via references
```

##### 2. Drop Flag (for non-Copy types)

```cpp
bool drop_flag;  // Tracks whether destructor should run
```

This prevents double-free and tracks move semantics, similar to how the Rust compiler
generates drop flags.

##### 3. Special Member Functions

**For Copy types** (`wellknown_traits(Copy)`):

```cpp
Type();                          // Default constructor
~Type();                         // Destructor
Type(const Type&);              // Copy constructor
Type& operator=(const Type&);   // Copy assignment
Type(Type&&);                   // Move constructor
Type& operator=(Type&&);        // Move assignment
```

Copy types use `memcpy` for both copy and move operations.

**For non-Copy types**:

```cpp
Type() : drop_flag(false) { }   // Default constructor

~Type() {                        // Destructor with drop flag check
    if (drop_flag) {
        rust_drop_in_place(&data[0]);
    }
}

Type(const Type&) = delete;      // No copy allowed
Type& operator=(const Type&) = delete;

Type(Type&&);                    // Move transfers drop flag
Type& operator=(Type&&);
```

The move operations transfer the `drop_flag` from the source to the destination,
ensuring only one object owns the Rust value at a time.

##### 4. Constructors

For each `constructor` declaration in the `.zng` file:

```cpp
Type(arg1_type arg1, arg2_type arg2, ...);
```

These constructors call the corresponding Rust functions to initialize the object.

##### 5. Static and Member Methods

For each function and method declared, the header contains declarations:

```cpp
static ReturnType method_name(Args...) noexcept;           // Static method
ReturnType method_name(Args...) [const] noexcept;          // Member method
```

The implementations are placed in `generated.cpp` to reduce header bloat and compilation times.

Methods can have different receiver types:

- Static methods have no receiver
- `&self` methods are `const` member functions
- `&mut self` methods are non-const member functions
- `self` methods consume the object (move)

##### 6. Field Access

For each field exposed in the `.zng` file:

```cpp
[[no_unique_address]] ::rust::FieldOwned<FieldType, OFFSET> field_name;
```

The `[[no_unique_address]]` attribute ensures these zero-sized types don't increase
the size of the containing class. The `OFFSET` template parameter encodes the field's
byte offset for direct memory access.

##### 7. make_box for Trait Objects

For `Box<dyn Trait>` types, generates factory functions:

```cpp
// For normal traits
template<typename T, typename... Args>
static inline Type make_box(Args&&... args);

// For Fn traits
static inline Type make_box(::std::function<Output(Inputs...)> f);
```

These allow C++ classes and lambdas to be converted into Rust trait objects.

##### 8. C++ Opaque Type Support

If `#cpp_value` is specified for a type:

```cpp
inline CppType& cpp() {
    return (*accessor_fn(&data[0])).as_cpp<CppType>();
}
```

This provides access to the underlying C++ object for types that wrap C++ values.

#### D. Reference Type Specializations

For each type, generates `Ref<Type>` and `RefMut<Type>` template specializations:

```cpp
namespace rust {
    template<>
    struct Ref<Type> {
    private:
        size_t data;  // Or array<size_t, 2> for unsized types

    public:
        Ref();
        Ref(const Type& t);
        template<size_t OFFSET>
        Ref(const FieldOwned<Type, OFFSET>& f);
        // ... method forwarding
    };

    template<>
    struct RefMut<Type> {
        // Similar to Ref, but allows mutation
    };
}
```

**Thin vs Fat Pointers**: Sized types use a single `size_t` for the pointer, while
unsized types (`?Sized`) use `array<size_t, 2>` to store both the data pointer and
metadata (length for slices, vtable for trait objects).

#### E. Field Type Specializations

For each type, generates specializations of `FieldOwned`, `FieldRef`, and `FieldRefMut`
with nested field access:

```cpp
template<size_t OFFSET>
struct FieldOwned<Type, OFFSET> {
    // Nested field accessors
    FieldOwned<FieldType, OFFSET + field_offset> nested_field;

    // Method forwarding
    ReturnType method(Args...) const noexcept;
};
```

This enables chained field access like `obj.field1.field2.field3`.

#### F. Debug Support

For types with `wellknown_traits(Debug)`, generates pretty printer specializations:

```cpp
template<>
struct ZngurPrettyPrinter<Type> {
    static inline void print(Type const& t) {
        rust_debug_print(&t.data[0]);
    }
};
```

Plus corresponding specializations for `Ref<Type>`, `RefMut<Type>`, and all field types.

### Function Implementations

Function implementations are split between the header and source files:

#### Inline in Header

**Free functions** (non-member functions) remain inline in the header:

```cpp
inline ReturnType function_name(ArgType1 i0, ArgType2 i1) noexcept {
    ReturnType o{};
    ::rust::__zngur_internal_assume_deinit(i0);
    ::rust::__zngur_internal_assume_deinit(i1);
    __zngur_mangled_name(
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(i1),
        ::rust::__zngur_internal_data_ptr(o)
    );
    // Handle panics if enabled
    if (detect_panic()) {
        take_panic();
        throw rust::Panic();
    }
    ::rust::__zngur_internal_assume_init(o);
    return o;
}
```

**Template functions** (like `make_box<T>`) and **forwarding methods** (for `Ref`/`RefMut`/`Field*` types) also remain inline for correctness and performance.

#### Moved to generated.cpp

Static and non-static method implementations are moved to `generated.cpp` to reduce header bloat and improve compilation times. See the next section for details.

### Exported Functions and Impl Blocks

For `extern "C++"` declarations (C++ code called from Rust):

```cpp
namespace rust::exported_functions {
    ReturnType function_name(Args...);
}

namespace rust {
    template<>
    class Impl<Type, Trait> {
    public:
        static ReturnType method_name(Args...);
    };
}
```

These are forward declarations; the implementations must be provided by user C++ code.

## 4. `generated.cpp` - C++ Implementation File

**Purpose**: Contains implementations of method wrappers and C++ → Rust bridge functions.

### Header Include

```cpp
#include "generated.h"
```

### Constructor Implementations

For each constructor declared in the `.zng` file:

```cpp
Type::Type(ArgType1 i0, ArgType2 i1) noexcept {
    ::rust::__zngur_internal_assume_init(*this);
    __zngur_constructor_mangled_name(
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(i1),
        ::rust::__zngur_internal_data_ptr(*this)
    );
    ::rust::__zngur_internal_assume_deinit(i0);
    ::rust::__zngur_internal_assume_deinit(i1);
}
```

### Method Implementations

For each static and non-static method:

```cpp
ReturnType Type::method_name(ArgType1 i0, ArgType2 i1) noexcept {
    ReturnType o{};
    ::rust::__zngur_internal_assume_deinit(i0);
    ::rust::__zngur_internal_assume_deinit(i1);
    __zngur_method_mangled_name(
        ::rust::__zngur_internal_data_ptr(i0),
        ::rust::__zngur_internal_data_ptr(i1),
        ::rust::__zngur_internal_data_ptr(o)
    );
    // Handle panics if enabled
    if (detect_panic()) {
        take_panic();
        throw rust::Panic();
    }
    ::rust::__zngur_internal_assume_init(o);
    return o;
}
```

These implementations are moved out of the header to reduce compilation time and header bloat.

### make_box Implementations for Fn Traits

For `Box<dyn Fn(...)>` types, the non-template `make_box` implementations are also in the `.cpp` file:

```cpp
Type Type::make_box(::std::function<Output(Inputs...)> f) {
    auto data = new ::std::function<Output(Inputs...)>(f);
    Type o;
    ::rust::__zngur_internal_assume_init(o);
    __zngur_box_new_mangled_name(
        reinterpret_cast<uint8_t*>(data),
        [](uint8_t *d) { delete reinterpret_cast<::std::function<Output(Inputs...)>*>(d); },
        [](uint8_t *d, uint8_t* i0, ..., uint8_t* o) {
            auto dd = reinterpret_cast<::std::function<Output(Inputs...)>*>(d);
            Output oo = (*dd)(::rust::__zngur_internal_move_from_rust<Inputs>(i0), ...);
            ::rust::__zngur_internal_move_to_rust<Output>(o, oo);
        },
        ::rust::__zngur_internal_data_ptr(o)
    );
    return o;
}
```

Template `make_box` functions for normal traits remain in the header.

### Extern "C" Function Definitions

The `.cpp` file also contains `extern "C"` function definitions that Rust can call.

#### 1. Trait Method Implementations

For each C++ trait implementation, generates glue functions that translate between
the C ABI and C++ member functions:

```cpp
extern "C" {
    void __zngur_trait_method(uint8_t* data, uint8_t* i0, ..., uint8_t* o) {
        // Cast opaque pointer to trait type
        TraitType* data_typed = reinterpret_cast<TraitType*>(data);

        // Move arguments from Rust
        OutputType oo = data_typed->method_name(
            ::rust::__zngur_internal_move_from_rust<ArgType0>(i0),
            ...
        );

        // Move result back to Rust
        ::rust::__zngur_internal_move_to_rust(o, oo);
    }
}
```

#### 2. Exported Function Implementations

For each `extern "C++"` free function:

```cpp
extern "C" {
    void __zngur_mangled_name(uint8_t* i0, ..., uint8_t* o) {
        OutputType oo = ::rust::exported_functions::function_name(
            ::rust::__zngur_internal_move_from_rust<ArgType0>(i0),
            ...
        );
        ::rust::__zngur_internal_move_to_rust(o, oo);
    }
}
```

#### 3. Impl Block Implementations

For each `extern "C++"` impl block method:

```cpp
extern "C" {
    void __zngur_mangled_name(uint8_t* i0, ..., uint8_t* o) {
        OutputType oo = ::rust::Impl<Type, Trait>::method_name(
            ::rust::__zngur_internal_move_from_rust<ArgType0>(i0),
            ...
        );
        ::rust::__zngur_internal_move_to_rust(o, oo);
    }
}
```

## Key Design Principles

Understanding these design principles helps explain why the generated code looks the way it does:

### 1. ABI Safety

All Rust↔C++ communication goes through `extern "C"` functions that pass `uint8_t*` pointers.
This avoids Rust's undefined ABI by using only the stable C ABI. The actual types are
reconstructed on each side using `ptr::read` and `ptr::write` in Rust, and `memcpy`
in C++.

### 2. Move Semantics

Zngur simulates Rust's move semantics in C++ using `memcpy` plus a `drop_flag`.
When a C++ object is "moved", the data is copied and the source's drop flag is cleared,
preventing the destructor from running. This matches Rust's behavior where moves are
bitwise copies that leave the source uninitialized.

### 3. Memory Safety

The drop flag mechanism prevents double-free errors. The initialization check functions
catch use-after-move bugs in debug builds, providing similar safety to Rust's borrow checker
(though only at runtime in C++).

### 4. Zero-Cost Abstractions

References are implemented as wrapped pointers with no additional overhead.
Field access uses zero-sized types with offset template parameters, compiling down to
direct memory access with no runtime cost. Owned values use inline storage when possible
to avoid heap allocation.

### 5. Type Erasure

C++ objects become `ZngurCppOpaqueOwnedObject` when passed to Rust, using type erasure
to work around Rust's inability to store C++ objects by value. Traits are implemented
via vtable-like function pointers, enabling dynamic dispatch from Rust to C++ implementations.

### 6. Namespace Mapping

Rust's module hierarchy maps directly to C++ namespaces under the `::rust::` prefix.
For example, `::std::vec::Vec` in Rust becomes `::rust::std::vec::Vec` in C++.
This creates a clear and predictable naming scheme.

## Summary

Zngur's generated files work together to enable seamless bidirectional interop:

### Rust Side (Both Modes)

- **`generated.rs`** contains FFI bridge functions, trait wrappers, and compile-time assertions

### Split-Header Mode (Default)

**C++:**

- **`zngur.h`** provides reusable infrastructure (templates, utilities, base types)
- **`generated.h`** defines C++ wrappers, with type declarations and inline functions
- **`generated.cpp`** contains method implementations and C++ → Rust call bridges

The separation between foundation and application-specific code reduces compilation time, enables header reuse across multiple libraries, and keeps template functions inline for performance.

### Single-Header Mode (`--single-header`)

**C++:**

- **`generated.h`** contains both infrastructure and type wrappers (merged)
- **`generated.cpp`** contains method implementations and C++ → Rust call bridges

This simpler mode is useful for small projects or backward compatibility.

Together, these files enable:

- **Rust → C++**: Using Rust types naturally in C++ with proper move/copy semantics
- **Rust → C++**: Calling Rust functions from C++ as if they were native C++ functions
- **Rust → C++**: Field access, method calls, and pattern matching on Rust types from C++
- **C++ → Rust**: Implementing Rust traits in C++ classes
- **C++ → Rust**: Passing C++ objects to Rust as trait objects
- **C++ → Rust**: Converting C++ lambdas to Rust closures

The generated code maintains type safety, manages memory correctly across the language
boundary, and achieves zero-cost abstractions wherever possible. The Rust bridge code
(`generated.rs`) handles the unsafe FFI operations, exposing a safe Rust API, while the
C++ code provides ergonomic wrappers that feel natural in each language.
