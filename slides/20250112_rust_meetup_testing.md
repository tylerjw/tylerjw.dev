---
title: Testing in Rust
colorSchema: light
---

# Basics to Advanced: Testing

Tyler Weaver

<!--
As a new programmer, the loop I went through when writing programmers was to write some code, compile, and then run the program I'd written.
As I progressed, I started to build more and more complex programs, and the iteration loop of writing some new code, compiling, and then interacting with my program to test that code got longer.
Later, when I started writing software on teams where the scale of software we were writing was bigger and bigger, it became impossible to test what I and others were making in a naive way.

This meant that the software we wrote was regularly not what we thought we built.
Testing is often the practice of automating that whole step where we run the program to discover what it does.
Instead of manually interacting with our whole program, we write many small programs that interact with different amounts of our project to learn what they do.
-->

---
layout: section
---

# The Basics

---

## Assert Macro

```rust
assert!(add(2, 2) == 3);
```

<!--
Let's talk about the humble assert macro.
This is the work-horse of statements within the test code.

Let's look at the output from this assert macro call.
-->

---

## Assert Macro

```rust
assert!(add(2, 2) == 3);
```
```
---- tests::no_worky stdout ----
thread 'tests::no_worky' panicked at src/lib.rs:17:9:
assertion failed: add(2, 2) == 3
```

<!--
Can we do better?
-->

---

## Assert Macro

```rust
assert!(add(2, 2) == 3);
```
```
---- tests::no_worky stdout ----
thread 'tests::no_worky' panicked at src/lib.rs:17:9:
assertion failed: add(2, 2) == 3
```

```rust
assert_eq!(add(2, 2), 3);
```
```
---- tests::no_worky stdout ----
thread 'tests::no_worky' panicked at src/lib.rs:17:9:
assertion `left == right` failed
  left: 4
 right: 3
```

<!--
What do we like, what do we not like about this?
-->

---

## Assert Macro

```rust
assert!(add(2, 2) == 3);
```
```
---- tests::no_worky stdout ----
thread 'tests::no_worky' panicked at src/lib.rs:17:9:
assertion failed: add(2, 2) == 3
```

```rust
assert_eq!(add(2, 2), 3);
```
```
---- tests::no_worky stdout ----
thread 'tests::no_worky' panicked at src/lib.rs:17:9:
assertion `left == right` failed
  left: 4
 right: 3
```

```rust
assert_eq!(add(2,2), 3, "2+2 should equal 3");
```
```
---- tests::no_worky stdout ----
thread 'tests::no_worky' panicked at src/lib.rs:17:9:
assertion `left == right` failed: 2+2 should equal 3
  left: 4
 right: 3
```

<!--
What do we like, what do we not like about this?

Note that there is also an assert_ne macro we can use.
-->

---

## Assert Macro takes a Format expression

```rust
assert!(a + b == 30, "a = {a}, b = {b}");
```
```
---- tests::no_worky stdout ----
thread 'tests::no_worky' panicked at src/lib.rs:20:9:
assertion `left == right` failed: a = 2, b = 33
  left: 35
 right: 30
```


---

## Unit Tests

````md magic-move
```rust
fn add(left: usize, right: usize) -> usize {
    left + right
}

#[test]
fn it_works() {
    assert_eq!(add(2,2), 4);
}
```
```rust
fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
```
````

<!--
You write these tests right along side your existing library code.
To build and run these tests we can simply type `cargo test`.

How this works is conditional compilation.
What that means is that there are different flags that can be set when compiling that affects what source code gets built.
The test build config is special in that you don't write the main function.
Instead you provide a set of free functions annotated with the test macro that adds these free functions to a list of test functions.

Rust then builds your code with a main function provided by the test harness (the code that gets added in automatically) which calls these test functions.
It does a whole bunch of other things too we can interact with in different ways through command line arguments.

(transition to show test module)

Another thing you can do is put your test functions in a test module.
One advantage of this is you can have a different set of use statements just for the test code separate from your regular library code.
Later we'll see examples where we need to have use statements for tests that we don't want in our regular library code.
There is a common short hand way of using all the code in the outside module you see here.
You'll notice that the add function is not public, yet we can still access it here in this separate module.
-->

---
layout: section
---

# Integration Tests

<!--
To understand what our software does at every layer we need to test it at that layer.
And, what layer is more important than the way our software behaves from the perspective of our users?

To test that we need to write tests that use our library in exactly the way our users can.

When you place a rust file in the directory `tests` at the root of your project next to `src` cargo builds that rust file into a stand-alone rust crate that depends on your library and hooks it up to the test harness.
-->

---

## Integration Tests

### src/lib.rs
```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

<!--
Going back to our toy function from above let's make it pub so our users can use it.
We assume it is in a crate called `addr`.
-->

---

## Integration Tests

### src/lib.rs
```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### tests/integration_tests.rs
```rust
#[test]
fn test_add() {
    assert_eq!(adder::add(3, 2), 5);
}
```

<!--
Now when we call `cargo test` it'll build this separate crate called `integration_tests` which depends on our `addr` library.
-->

---

## Documentation Tests

````md magic-move
```rust
pub fn add(left: usize, right: usize) -> usize {
    left + right
}
```
```rust
/// ```
/// let result = addr::add(2, 3);
/// assert_eq!(result, 5);
/// ```
pub fn add(left: usize, right: usize) -> usize {
    left + right
}
```
```rust
/// This is the summary line for the function.
///
/// # Examples
///
/// ```
/// let result = addr::add(2, 3);
/// # assert_eq!(result, 5);
/// ```
pub fn add(left: usize, right: usize) -> usize {
    left + right
}
```
````

<!--
The final kind of testing you need to understand in this beginners' guide is documentation testing.

If you've had the experience of writing documentation for a library before you'll know what I mean when I say that documentation is always out of date.
Without an incredible amount of manual testing or re-creating the workflows in your docs in tests, in other language ecosystems I came to dread code examples in docs.

At one point when working as a maintainer of a large robotic arm library where we had a tutorial website I started manually re-producing the code in the examples in tests to try to help with this problem.

You can tell the authors of the tooling for documentation and testing in Rust came from similar experiences.

(transition) show how to write a basic example

(transition) show how to hide a line in the output example

This is often helpful if you have some ugly setup you want to include in the doc-test or ugly asserts you want for testing purposes.
Be careful doing this though as it can be easy to hide from users lines of code that are important for them to get your library code to work.
-->

---

## Doctests from the README

```rust
#![doc = include_str!("../README.md")]
```

---
layout: image

image: ./20250122_tests_name_meme.jpg
---

<!--
I know, I've done it too.
We need to talk about naming.

So, if you didn't know, one of the purposes of the test name is you can filter the tests you want to run with cargo.
So, for example, say I put the word database in the name of all the tests that use the database I could run cargo test database and it'll test all those tests.

Also, your module name you put the tests in is in that namespace too.
So if you want to run all the tests in your args module you can run cargo test args.
-->

---
layout: section
---

# Itermediate: Async

---

## Tokio Async Test

```rust
#[tokio::test]
async fn my_test() {
    assert!(true);
}
```

---

## Tokio Async Test

```rust
#[tokio::test]
async fn my_test() {
    assert!(true);
}
```

```rust
#[tokio::test(flavor = "multi_thread")]
async fn my_test() {
    assert!(true);
}
```

---

## Waiting

```rust
// wait for is_finished to be true, or timeout
let start = tokio::time::Instant::now();
loop {
    if is_finished() {
        break;
    }
    if tokio::time::Instant::now().duration_since(start) >
        tokio::time::Duration::from_secs(5)
    {
        panic!("Timed out waiting for is_finished to be true");
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(1));
}
```

<!--
Here is some code out of a test I wrote at some point earlier this year.
Talk to me about how you'd improve this, or what you'd do differently.
-->

---

## Lazy Sync Runner

```rust
fn lazy_sync_runner() -> anyhow::Result<Arc<tokio::runtime::Runtime>> {
    static ASYNC_RUNTIME: OnceLock<Mutex<Weak<tokio::runtime::Runtime>>> = OnceLock::new();
    let mut guard = ASYNC_RUNTIME
        .get_or_init(|| Mutex::new(Weak::new()))
        .lock()
        .map_err(|e| {
            anyhow!("failed to build a runtime for sync-runner: {e}")
        })?;

    match guard.upgrade() {
        Some(runtime) => Ok(runtime),
        None => {
            let runtime = Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    .thread_name("lazy-sync-runner")
                    .worker_threads(2)
                    .enable_all()
                    .build()?,
            );
            *guard = Arc::downgrade(&runtime);
            Ok(runtime)
        }
    }
}
```

---
layout: section
---

# Advanced: Global State

---

## Logging


````md magic-move
```rust
let _default_guard = tracing::subscriber::set_global_default(
    tracing_subscriber::fmt::Subscriber::builder()
    .compact()
    .with_max_level(tracing::Level::TRACE)
    .finish(),
);
```
```rust
use std::sync::Once;

static LOGGING_STATE: Once = Once::new();
pub fn setup_test_logging() {
    LOGGING_STATE.call_once(|| {
        tracing::subscriber::set_global_default(
            tracing_subscriber::fmt::Subscriber::builder()
                .compact()
                .with_max_level(tracing::Level::ERROR)
                .finish(),
        )
        .expect("logging init for tests failed")
    });
}
```
```rust
#[cfg(test)]
mod tests {
    use test_log::test;
    use tracing::info;

    #[test]
    fn no_more_logging_boilerplate() {
        info!("logging works as expected now...");
        assert_eq!(4 % 2, 0);
    }

    #[test]
    fn a_failing_test() {
        info!("wonder why this failed?");
        assert_eq!(4 % 2, 1);
    }
}
```
````

<!--
Anyone know the type of _default_guard?
-->

---

## test_log

```toml
[dev-dependencies]
test-log = { version = "0.2.16", features = ["trace", "color"] }
```

```rust
use test_log::test;
use tracing::info;

#[test]
fn sync_test() {
    info!("we can log in tests");
}

#[test(tokio::test)]
async fn async_test() {
    info!("even async ones");
}
```

---

## Testing with a Database

```rust
#[test]
fn database_update_works() {
    let default_database_url = "postgresql://yugabyte@localhost:5432/yugabyte?sslmode=disable";
    let connection = diesel::PgConnection::establish(&default_database_url);
    // ...
}
```

<!--
Let's talk about what is wrong with this.

Tell me how you might and have dealt with this?
-->

---

## test_context

```rust
use test_context::{test_context, AsyncTestContext};

struct MyAsyncContext {
    value: String
}

impl AsyncTestContext for MyAsyncContext {
    async fn setup() -> MyAsyncContext {
        MyAsyncContext { value: "Hello, world!".to_string() }
    }
    async fn teardown(self) {
        // Perform any teardown you wish.
    }
}

#[test_context(MyAsyncContext)]
#[tokio::test]
async fn test_async_works(ctx: &mut MyAsyncContext) {
    assert_eq!(ctx.value, "Hello, World!");
}
```

<!--
What could we do with this to fix our database testing problem?
-->

---

## testcontainers

```rust
use testcontainers_modules::{postgres, testcontainers::runners::SyncRunner};

#[test]
fn test_with_postgres() {
    let container = postgres::Postgres::default().start().unwrap();
    let host_port = container.get_host_port_ipv4(5432).unwrap();
    let connection_string = &format!(
        "postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",
    );
}
```

<!--
If we combine the test_context and this we can have a nice easy solution, right?
Why not?

Talk about what I did
-->


---
layout: section
---

# Who tests the tests?

<!--
cargo-mutants
-->

---
layout: section
---

# I wanna to go fast!

<!--
nextest
-->
---
