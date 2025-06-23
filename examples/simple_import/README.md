# Example: Import and Merge

A demonstration of Zngur's `import` and `merge` functionality using four focused modules.

## Structure

- **`primitives.zng`** - Defines basic primitive types (`bool`)
- **`foo.{zng,cpp}`** - Imports primitives, defines `Vec<i32>` APIs and returns a populated Vec
- **`bar.{zng,cpp}`** - Imports primitives, defines `Option<String>` APIs and returns an Option
- **`main.{zng,cpp}`** - Imports foo and bar (transitively gets primitives), extends both types with additional APIs, and demonstrates everything

## API Extensions in main.zng

The main module doesn't just import - it extends the imported types:

- **Vec<i32>**: Adds `is_empty()` and `clear()` methods beyond `foo.zng`'s `new()` and `push()`
- **Option<String>**: Adds `unwrap()` method beyond `bar.zng`'s constructors and `is_some()`/`is_none()`

## Running

```bash
make
./a.out
```
