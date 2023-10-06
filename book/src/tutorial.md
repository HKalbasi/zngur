# Tutorial

A Zngur project consists of 3 things:

- An IDL (interface definition language) file named `main.zng`
- A Rust crate (that can be everything, binary, rlib, static-lib, cdy-lib, ...)
- A C++ project.

To start, install Zngur:

```
cargo install zngur-cli
```

Then generate a new `staticlib` crate using `cargo init` and appending this to the `Cargo.toml`:

```Toml
[lib]
crate-type = ["staticlib"]
```

And create an empty `main.cpp` and `main.zng` file. Your directory tree should look like this:

```
├── Cargo.toml
├── main.cpp
├── main.zng
└── src
    └── lib.rs
```

## Basic structure of `main.zng`

Imagine we want to use this inventory in C++:

```Rust
struct Item {
    name: String,
    size: u32,
}

struct Inventory {
    items: Vec<Item>,
    remaining_space: u32,
}

impl Inventory {
    fn new_empty(space: u32) -> Self {
        Self {
            items: vec![],
            remaining_space: space,
        }
    }
}
```

Copy it into `src/lib.rs`. Now we need to declare things that we need to access in C++ in the `main.zng` file:

```
type crate::Inventory {
    #layout(size = 32, align = 8);

    fn new_empty(u32) -> crate::Inventory;
}
```

Zngur needs to know the size and align of the types inside the bridge. You can figure it out using the rust-analyzer (by hovering
over the struct or type alias) or fill it with some random number and then fix it from the compiler error.

> **Note:** Ideally `main.zng` file should be auto-generated, but we are not there yet. Also, Zngur can work without explicit size and
> align (with some caveats), see [layout policies](./call_rust_from_cpp/layout_policy.md) for more details.

Now, run `zngur g ./main.zng` to generate the C++ and Rust glue files. It will generate a `./generated.h` C++ header file, and a
`./src/generated.rs` file. Add a `mod generated;` to your `lib.rs` file to include the generated Rust file. Then fill `main.cpp` file
with the following content:

```C++
#include "./generated.h"

int main() {
  auto inventory = rust::crate::Inventory::new_empty(1000);
}
```

Zngur will add every Rust item with its full path in the `rust` namespace, so for example `String` will become `rust::std::string::String` in the
C++ side.

To build it, you need to first build the Rust code using `cargo build`, which will generate a `libyourcrate.a` in the `./target/debug` folder, and
you can build your C++ code by linking to it:

```bash
clang++ main.cpp -g -L ./target/debug/ -l your_crate
```

To ensure that everything works, let's add a `#[derive(Debug)]` to `Inventory` and use `zngur_dbg` to see it:

```
type crate::Inventory {
    #layout(size = 32, align = 8);
    wellknown_traits(Debug);

    fn new_empty(u32) -> crate::Inventory;
}
```

```C++
int main() {
  auto inventory = rust::crate::Inventory::new_empty(1000);
  zngur_dbg(inventory);
}
```

There are some traits that Zngur has special support for them, and `Debug` is among them. [This page](./call_rust_from_cpp/wellknown_traits.md) has the
complete list of them.

Assuming that everything works correctly, you should see something like this after executing the program:

```
[main.cpp:5] inventory = Inventory {
    items: [],
    remaining_space: 1000,
}
```

Now let's add some more methods to it:

```Rust
impl Inventory {
    fn add_item(&mut self, item: Item) {
        self.remaining_space -= item.size;
        self.items.push(item);
    }

    fn add_banana(&mut self, count: u32) {
        for _ in 0..count {
            self.add_item(Item {
                name: "banana".to_owned(),
                size: 7,
            });
        }
    }
}
```

```
type () {
    #layout(size = 0, align = 1);
}

type crate::Inventory {
    #layout(size = 32, align = 8);
    wellknown_traits(Debug);

    fn new_empty(u32) -> crate::Inventory;
    fn add_banana(&mut self, u32);
}
```

Note that the return type of `add_banana` is the `()` type, so we need to add it as well. Now we can use it in the C++ file:

```C++
#include "./generated.h"

int main() {
  auto inventory = rust::crate::Inventory::new_empty(1000);
  inventory.add_banana(3);
  zngur_dbg(inventory);
}
```

```
[main.cpp:6] inventory = Inventory {
    items: [
        Item {
            name: "banana",
            size: 7,
        },
        Item {
            name: "banana",
            size: 7,
        },
        Item {
            name: "banana",
            size: 7,
        },
    ],
    remaining_space: 979,
}
```

Bridging the `add_item` method requires a little more effort. We need to declare `crate::Item` type as well since it is an argument
of that function:

```
// ...

type crate::Item {
    #layout(size = 32, align = 8);
}

type crate::Inventory {
    // ...
    fn add_item(&mut self, crate::Item);
}
```

But using that alone we can't use the `add_item` since there is no way to obtain a `rust::crate::Item` in the C++ side. To fix that, we
need to add the constructor for the `Item`:

```
type ::std::string::String {
    #layout(size = 24, align = 8);
}

type crate::Item {
    #layout(size = 32, align = 8);

    constructor { name: ::std::string::String, size: u32 };
}
```

But it doesn't solve the problem, since we can't create a `String` so we can't call the constructor. To creating a `String`, we declare the
primitive type `str` and its `to_owned` method:

```
type str {
    wellknown_traits(?Sized);

    fn to_owned(&self) -> ::std::string::String;
}
```

There are some new things here. First, since `str` is a primitive it doesn't need full path. Then there is `wellknown_traits(?Sized)` instead
of `#layout(size = X, align = Y)` which tells Zngur that this type is unsized and it should consider its references as fat and prevent storing it
by value.

Now you may wonder how we can obtain a `&str` to make a `String` from it? Fortunately, Zngur has some special support for primitive types and it
has a `rust::Str::from_char_star` function that creates a `&str` from a zero terminated, valid UTF8 `char*` with the same lifetime. If Zngur didn't
have this, we could create a `&[u8]` by exporting its `from_raw_parts` and then converting it to a `&str`, `from_char_star` exists just for convenience.

So now we can finally use the `add_item` method:

```C++
int main() {
  auto inventory = rust::crate::Inventory::new_empty(1000);
  inventory.add_banana(3);
  rust::Ref<rust::Str> name = rust::Str::from_char_star("apple");
  inventory.add_item(rust::crate::Item(name.to_owned(), 5));
  zngur_dbg(inventory);
}
```

```
[main.cpp:8] inventory = Inventory {
    items: [
        Item {
            name: "banana",
            size: 7,
        },
        Item {
            name: "banana",
            size: 7,
        },
        Item {
            name: "banana",
            size: 7,
        },
        Item {
            name: "apple",
            size: 5,
        },
    ],
    remaining_space: 974,
}
```

## Generic types

Let's try to add and bridge the `into_items` method:

```Rust
impl Inventory {
    fn into_items(self) -> Vec<Item> {
        self.items
    }
}
```

`Vec` is a generic type, but the syntax to use it is not different:

```
type ::std::vec::Vec<crate::Item> {
    #layout(size = 24, align = 8);
    wellknown_traits(Debug);
}

type crate::Inventory {
    // ...
    fn into_items(self) -> ::std::vec::Vec<crate::Item>;
}
```

Note that this only brings `Vec<Item>`, for using `Vec<i32>` or `Vec<String>` or `Vec<SomethingElse>` you need to add each of
them separately.

Now you can use `into_items` method in C++:

```C++
rust::std::vec::Vec<rust::crate::Item> v = inventory.into_items();
zngur_dbg(v);
```

```
[main.cpp:11] v = [
    Item {
        name: "banana",
        size: 7,
    },
    Item {
        name: "banana",
        size: 7,
    },
    Item {
        name: "banana",
        size: 7,
    },
    Item {
        name: "apple",
        size: 5,
    },
]
```

You can see the full code at [`examples/tutorial`](https://github.com/HKalbasi/zngur/blob/main/examples/tutorial)
