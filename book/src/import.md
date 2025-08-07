# Import

The `import` directive allows you to include type definitions and other declarations from other `.zng` files into your main specification. Types, traits, and modules can appear multiple times across the transitive set of imported files, and their content is merged together.

## Syntax

```zng
import "./path/to/file.zng";
```

## Path Resolution

Import paths are resolved relative to the directory containing the current `.zng` file:

- `import "./types.zng";`

At this time, absolute paths are not supported.
Importing paths without a leading specified (e.g. `import "foo/bar.zng";`) is reserved for a possible future extension.

Above, "the current zng file" refers to the `.zng` file being parsed, which is not necessarily the top-level `.zng` file passed to `zngur` on the command line.

## Behavior

When an import statement is processed:

1. The parser reads and parses the imported file
2. All declarations from the imported file are _merged_ into the current specification
3. Imported content becomes available as if it were defined in the importing file
4. Import processing happens recursively - imported files can themselves contain import statements

## Merging

Zngur's merge algorithm attempts to compute the union of each set of declarations which share an identity (e.g. every `type crate::Inventory { ... }` across all imported files). Duplicates are ignored, but contradictions will raise a compiler error. For example, if two different `type crate::Inventory { ... }` declarations both specify `wellknown_traits(Debug);`, parsing will succeed. However, if they specify different layouts, an error will be reported.

## `#convert_panic_to_exception` constraints

`#convert_panic_to_exception` may only appear in a top-level `.zng` file. This is an application-level decision that should not be determined by dependent libraries.

## Example

**main.zng:**
```zng
import "./core_types.zng";
import "./iterators.zng";

// May only appear in the top-level file.
#convert_panic_to_exception

type MyApp {
    #layout(size = 8, align = 8);

    fn run(&self) -> i32;
}
```

**core_types.zng:**
```zng
mod ::std {
    type option::Option<i32> {
        #layout(size = 8, align = 4);
        wellknown_traits(Copy);

        constructor None;
        constructor Some(i32);

        fn unwrap(self) -> i32;
    }

    mod vec {
        type Vec<i32> {
            #layout(size = 24, align = 8);
            fn new() -> Vec<i32>;
            fn push(&mut self, i32);
        }
    }
}
```

**iterators.zng:**
```zng
mod ::std {
    mod vec {
        type Vec<i32> {
            fn into_iter(self) -> ::std::vec::IntoIter<i32>;
        }
    }
}
```

In this example, `main.zng` imports type definitions from two separate files, allowing for better organization of the zngur specification.

Notice that `iterators.zng` is able to "reopen" the `::std::vec::Vec<i32>` specification and extend it with a single function, `into_iter`. It does not need to respecify the `#layout` because that is already declared in `core_types.zng`.
