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
    properties(size = 32, align = 8);

    fn new_empty(u32) -> crate::Inventory;
}
```

Zngur needs to know the size and align of the types inside the bridge. You can figure it out using the rust-analyzer (by hovering
over the struct or type alias) or fill it with some random number and then fix it from the compiler error.

> **Note:** Ideally `main.zng` file should be auto-generated, but we are not there yet.

Now, run `zngur g ./main.zng` to generate the C++ and Rust glue files. It will generate a `./generated.h` C++ header file, and a
`./src/generated.rs` file. Add a `mod generated;` to your `lib.rs` file to include the generated Rust file. Then fill `main.cpp` file
with the following content:

```C++
#include "./generated.h"

int main() {
  auto inventory = rust::crate::Inventory::new_empty(1000);
}
```

To build it, you need to first build the Rust code using `cargo build`, which will generate a `libyourcrate.a` in the `./target/debug` folder, and
you can build your C++ code by linking to it:

```bash
clang++ main.cpp -g -L ./target/debug/ -l your_crate
```

To ensure that everything works, let's add a `#[derive(Debug)]` to `Inventory` and use `zngur_dbg` to see it:

```
type crate::Inventory {
    properties(size = 32, align = 8);
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

There are some traits that Zngur has special support for them, and `Debug` is among them. [This page](./call_rust_from_cpp/wellknown_traits.html) has the
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
    properties(size = 0, align = 1);
}

type crate::Inventory {
    properties(size = 32, align = 8);
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

Bridging the `add_item` method requires a little more effort:

## Generic types

Now let's try to add and bridge the `into_items` method:

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
    properties(size = 24, align = 8);
    wellknown_traits(Debug);
}

type crate::Inventory {
    // Old things...
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
