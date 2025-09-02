# Panic and exceptions

By default, Zngur aborts the process if unwinding (Rust panic or C++ exception) reaches the cross-language boundary.
Handling unwinding adds a non-zero performance cost to every function call
and complicates things as catching unwinding can result in a corrupt state for some objects.
(See [unwind safety](https://doc.rust-lang.org/std/panic/trait.UnwindSafe.html))

But Zngur has support for converting Rust panics into a C++ exception. To enable that, add

```
#convert_panic_to_exception
```

to your `main.zng` file. Now you can catch exceptions of kind `rust::Panic` for handling Rust panics:

```C++
try {
    std::cout << "s[2] = " << *s.get(2).unwrap() << std::endl;
    std::cout << "s[4] = " << *s.get(4).unwrap() << std::endl;
} catch (rust::Panic e) {
    std::cout << "Rust panic happened" << std::endl;
}
```

assuming `s` contains `[2, 5, 7, 3]`, the above code prints:

```
s[2] = 7
thread '<unnamed>' panicked at 'called `Option::unwrap()` on a `None` value', examples/simple/src/generated.rs:184:39
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
s[4] = Rust panic happened
```

To disable the log, you need to register a panic hook on the Rust side
(See [this Stack Overflow question](https://stackoverflow.com/questions/35559267/suppress-panic-output-in-rust-when-using-paniccatch-unwind)). Note that the `rust::Panic` object contains nothing, and you will lose the panic message.

For proper error handling, consider returning `Result` from your Rust functions
and throw native C++ exceptions with proper details in case of an `Err` variant.
Use this panic-to-exception mechanism only in places where you need `catch_unwind` in Rust (e.g. for increasing fault tolerance).
