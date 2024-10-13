+++
title = "Upgrade the Logging in your Rust Tests"
date = 2024-10-12
type = "post"
in_search_index = true
[taxonomies]
tags = ["Rust", "Testing", "Logging"]
+++

In the spring of 2024, a rather unpleasant thing happened.
The startup I was working at was running out of money, and to keep from going out of business, they fired a bunch of us.

This turned into finally getting something I've wanted for the last few years.
I joined a team that was building a large project in Rust.
Seven months in, I can confidently say it has been glorious.

But as I've transitioned from someone who built small things for myself using Rust to someone working on a team delivering a larger project, I've learned some things the hard way.
Today, I'm going to walk you through one of those problems so you won't have to.

If you, like me, came from a language like C++, where most nice things you had you or a coworker built, this should feel familiar.
In Rust, it is much easier to reach for a dependency than in C++, but that doesn't solve the problem of knowing when to do so.
If I just look at tools others have built for problems I haven't really understood, it is hard for me to tell the difference between something that solves my problem and something I should avoid.

## Stage 0: A Bad Test

After several months of working on a team writing Rust, we have written many tests.
As we've written those tests, we sometimes use the logging system to help us write or debug them.
We've built an assortment of inconsistent and unhelpful test initializations without talking to each other about this.

This went largely unnoticed until I wrote the worst kind of test, a flaky test.
To debug this, I started picking through the logging output in CI and then locally and started looking more closely at logging in our tests.

## Stage 1: Copy-pasta

You have to initialize the logging state to get any output from your logs.
I knew each test should be able to run independently, so I put the logging boilerplate at the top of each testing function.
Here are a few code lines I then copy-pasted into all my tests.

```rust
let _default_guard = tracing::subscriber::set_global_default(
    tracing_subscriber::fmt::Subscriber::builder()
    .compact()
    .with_max_level(tracing::Level::TRACE)
    .finish(),
);
```

As with everything copied-pasted, this had the disadvantage that I had to try to keep all my tests in sync by copy-pasting this consistently into all of them.
That was annoying.
Also, anytime I wanted to debug a specific test, I would increase the log level.
This logging configuration broke one of the excellent features of `cargo test`, which was annoying.
I'll talk about that later.

## Stage 2: A Function

I wanted to improve this copy-paste situation, so I turned to my favorite programming feature: the humble function.
Before I wrote this function, I didn't realize there was a bug in the above code.

Do you see it?
What's the type of `_default_guard`?

It is a `Result`.
That is because you should only do this once in your program, and calling the function `set_global_default` a second-time results in an Error.
Okay, we also know tests can run in parallel or in the same process, so this is usually an error that we were just silently ignoring.

Having understood, I then reached for `lazy_static` and made it so this would only be initialized once.

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
Recognizing that I wanted to use this everywhere, I put it in a library, as you do when you want to reuse some code.
It lowered the amount of copy-pasted code to just a function call.

But then I had to return to my original problem, the flaky test.
This cleanup of copy-pasted code was fine and all but didn't make debugging that test easier.

## Stage 3: Debugging a Failed Test

Usually, when you run `cargo test`, it captures all the output to stdout and stderr from running your tests, and only when a test fails do you see what was sent to stdout and stderr.
This is an absolute requirement for your sanity, as tests are run in parallel.
If you don't have it capturing test output and hiding the passing ones, you'll have difficulty telling what happened via the logs when a test fails.
Your trouble case and perfectly good tests will have logs being interleaved.

For my specific case, this wasn't a huge pain on our CI system because we have lame runners that basically only run a single test at a time, but locally on our workstations, this means that if you get a test failure, the logs are no help.

So, to combat this when debugging locally, I was setting the log level to `ERROR` on all but the test I was debugging.
However, this means that I have to **make a code change, re-compile, and rerun the test** to have logs I could use.
This became rather unhelpful when I was trying to debug that flaky test that only flaked when run in parallel with certain other tests.

## Stage 4: Searching the Docs

At this point, I did a thing I often find myself doing after I came to a better understanding of my problem.
Looking into more details of how to configure various `tracing` options, you find ways to address some of these pain points.
I still don't know why the log capturing isn't working with this configuration, but I can at least solve the re-compile issue.
I found filters in `tracing_subscriber` that enable the user to [change how logging works via environment variables](https://docs.rs/tracing-subscriber/0.3.18/tracing_subscriber/filter/struct.EnvFilter.html#directives).

Going back to the beginning, though, you might wonder why I have to solve this problem myself.
I can't be the first person to write tests in Rust and want to use logging when debugging those tests.

## Stage 5: Not Alone

You are right.
I'm not the first person to point out that there exists a good solution for all this in the ecosystem.
One problem is that before you go through this, or before someone else tells you about all this, you don't know you need a solution.
So that's what I'm doing here.

The crate you were looking for is called [`test-log`](https://crates.io/crates/test-log).
It does everything I wanted and more (it even fixes the broken log output capturing behavior too).

How you use it is stupidly simple.
How it works is magic.
I don't understand but haven't had any reason to understand yet.

I did not see it in that readme, but if you use `tracing` for logging, you want to use the `trace` feature.
Also, because it makes it easier to read the logs, turn on `color` too.
Here is how I add it:

```toml
[dev-dependencies]
test-log = { version = "0.2.16", features = ["trace", "color"] }
```

After adding it to my dependencies, I could use it as shown in the example code below.

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

Let me show you what happens when you run this.
Here is the output from running `cargo test`.

```
❯ cargo test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running unit tests src/lib.rs (target/debug/deps/testing_examples-d432e0856feef924)

running 2 tests
test tests::no_more_logging_boilerplate ... ok
test tests::a_failing_test ... FAILED

failures:

---- tests::a_failing_test stdout ----
thread 'tests::a_failing_test' panicked at src/lib.rs:15:9:
assertion `left == right` failed
  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::a_failing_test

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

Okay, but where is our logging output?
Shouldn't we see a line with the output "wonder why this failed?" log?
The reason is that the default logging level with test-log is `ERROR`, so we can change that and get the output like this.

```
❯ RUST_LOG=info cargo test
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running unit tests src/lib.rs (target/debug/deps/testing_examples-d432e0856feef924)

running 2 tests
test tests::a_failing_test ... FAILED
test tests::no_more_logging_boilerplate ... ok

failures:

---- tests::a_failing_test stdout ----
2024-10-12T23:18:46.509344Z  INFO testing_examples::tests: wonder why this failed?
thread 'tests::a_failing_test' panicked at src/lib.rs:15:9:
assertion `left == right` failed
  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::a_failing_test

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

There it is, right in the stdout above the failure.
Okay, now that you know, you can upgrade how debuggable your tests are and get rid of any feature-less copy-paste you may have added to them.
