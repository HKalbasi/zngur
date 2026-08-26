# Coroutines and `Future`

Zngur supports integration between `async` and `Future` of Rust, and `co_await` and `co_return` features of C++20.
Similar to other things, C++ coroutine supports many different patterns,
including the Rust style `async`/`await` and future which uses `poll` and `Waker`,
but Rust can't support all C++ coroutine patterns. This reflects in what Zngur supports.

## Calling Rust async function from C++

You can have your Rust async function in the zng file using these equivalent syntaxes:

```
async fn foo() -> Bar;
fn foo() -> impl ::std::future::Future<Output = Bar>;
```

Similar to all other RPIT functions, the return value is available to you as a `Box<dyn Future<Output = Bar>>` type.
You need to have this type in your zng file, and you can have methods like `.into_pin` for it like any other type.

For using futures with tokio and similar runtimes, you may need the `Send + Sync` version of the function:

```
fn foo() -> impl ::std::future::Future<Output = Bar> + Send + Sync;
```

Which would return a `Box<dyn Future<Output = Bar>> + Send + Sync` in the C++ side.

## The `rust::Box<rust::Dyn<Future<T>>>` is a C++ coroutine and awaitable object

While using methods on the future type is possible, it is not enough for ergonomically working with async functions.
To make cross language async code seamless, Zngur makes `Box<Dyn<Future<T>>>` (and it's friend `Box<Dyn<Future<T>, Send, Sync>>`)
a C++ coroutine return type and awaitable object at the same time. This allows you to write code like this:

```C++
template <typename T>
using BoxFuture = rust::Box<rust::Dyn<rust::std::future::Future<T>, rust::Send, rust::Sync>>;

BoxFuture<uint64_t> cpp_sleep(uint64_t ms) {
  auto dur = rust::std::time::Duration::from_millis(ms);
  co_await rust::tokio::time::sleep(std::move(dur));
  std::cout << "tokio_sleep " << ms << " done" << std::endl;
  co_return ms;
}
```

Which should work almost identical to this Rust code:

```Rust
async fn cpp_sleep(ms: u64) -> u64 {
    let dur = std::time::Duration::from_millis(ms);
    tokio::time::sleep(dur).await;
    println!("tokio_sleep {ms} done");
    ms
}
```

The returned future from the `cpp_sleep` coroutine is a normal Rust future, you can pass it to Rust functions (like tokio's `block_on`)
or `co_await` on it in other coroutines.

## What Runtime to use?

Both C++ coroutines and Rust async need a runtime to schedule and manage async tasks.
Zngur is runtime agnostic and allow you to use whatever runtime you want.

### Using a Rust runtime

A Rust async runtime is just a library which have some functions like `spawn` that take a future.
You can use it like any other libraries in C++, and wrap its functions, including `spawn`.
Futures returned from C++ coroutines are a normal Zngur object, and you can pass it to the Rust runtime.
See [the tokio example](https://github.com/HKalbasi/zngur/blob/main/examples/tokio) which does this for the tokio runtime.

### Using a C++ runtime

C++ async runtimes can't accept a Rust future directly. So you need to write a code that drive the future,
call its poll and provide the waker. There are multiple C++ async runtimes so that code couldn't land in the Zngur core,
but it is possible to do it as a reusable library. If you did it for a C++ async runtime as an open source library,
please add a link to it here so that it become discoverable.

## Awaiting C++ awaitables in Rust

So Zngur allows you to use `co_await` with Rust futures, but what about the other direction?
Can you `.await` C++ awaitable objects in Rust? C++ coroutines have many degrees of freedom,
but Rust `.await` syntax is tied to the `Future` trait: if your awaitable object can be used using the `Future` protocol,
you can use it with Rust `.await`, and otherwise there is no way.
Your awaitable object needs to be lazy, have a `poll` function which makes progress and tell if it is done or not,
and accepts a callback (called `waker`) to notify the executor that some progress might be available again.
Many of C++ awaitable objects are fundamentally incompatible with this model, so you can't use them with Rust's `.await` syntax.

If the awaitable object can be used using the `Future` protocol,
you can construct a Zngur `Box<Dyn<Future<T>>>` from them. Zngur allows you to
[implement the `Future` trait](../call_cpp_from_rust/trait_object.md)
for your C++ classes, and construct a `Box<Dyn<Future<T>>>` from them,
which is `.await`able from Rust.

## Performance

Calling a Rust async function in C++ will add a heap allocation for the `Box` used.
Zngur could support writing size and alignments for futures returned by async functions in the zng file and holding them in stack,
but it would be extremely fragile (every change to the async functions would change the size of future)
so I don't think it would worth the effort.
If performance is important, I would suggest you to keep all of your async code in Rust,
and call the C++ logic using a sync and IO free boundary. Just because Zngur has a feature,
it doesn't mean that you should use it :).
