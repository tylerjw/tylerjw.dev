+++
title = "Building Trustworthy Software: The Power of Testing in Rust"
date = 2024-11-06
type = "post"
in_search_index = true
[taxonomies]
tags = ["Rust", "Testing", "Software"]
+++

You and I only have a small amount of time to write software.
And with that same block of time, some people build incredible software, and others are stuck doing small changes around the edges of something they never come to understand.
You can chalk it up to randomness or try to figure out how some are incredibly productive while others struggle with the same things repeatedly.

Before you come at me with your pitchforks, I'm not talking about software that is a financial success or gains many users.
We all know Microsoft Teams sucks in comparison to everything else, but it has orders of magnitudes of more users.
Business strategies like bundling can make the quality of the product irrelevant if all you care about is user counts.
I'm talking about building good software; I want to develop software that other people like me will love.
Don't you?

{{ img_caption(path="/images/20241106_the_feeling.jpg", caption="This Could be You") }}

A big part of building software one can be proud of is robustness.
The simple property that the library or application has the behaviors specified and does not unpleasantly surprise the user.
This is critical because software is not a one-person operation; your quality is not only between you and some end user; it is the amount other programmers can trust you.
It directly affects the quality of abstractions you can build.

The way we get there is testing.
Testing is vital for skilled artisans, separating them from middling programmers.
The strong testing story is a killer feature of the Rust language and ecosystem.
I'm here to walk you through what you need to know to test what you are building.

 The basics of writing tests are well described in [the book](https://doc.rust-lang.org/book/ch11-01-writing-tests.html).
 But to set a foundation, let's walk through them here, too.

## The Unit Test

The first sort of test we'll write is the unit test.
It lives in your module (rust file in the src directory) right next to the code it tests.
Unit tests are just functions with the `#[test]` annotation.


In 2022, I used the Advent of Code puzzles to improve my rust skills.
Here is a small excerpt from one of the programs I wrote.
Those of you who have represented 2d data in a 1d data structure like a vector will probably find this function familiar.

```Rust
fn idx(width: usize, x: usize, y: usize) -> usize {
    width * y + x
}
```

Starting with that first one, here is a test that permeates through various inputs that result in a zero index.

```Rust
fn idx(width: usize, x: usize, y: usize) -> usize {
    width * y + x
}

#[test]
fn zero_index() {
    assert_eq!(idx(0, 0, 0), 0);
    assert_eq!(idx(100, 0, 0), 0);
    assert_eq!(idx(usize::MAX, 0, 0), 0);
    assert_eq!(idx(0, 0, 15), 0);
}
```

How do we feel about this test? Does it make sense that the width can be 0? Is the order of the arguments in my function confusing? What does the order of my function arguments say about the memory layout of my 2d data?

Writing this arguably dumb test leads to asking valuable questions about how we could change the interface of this function and its contract with the user.

What is going on behind the scenes here is magic, which I can tell you about, but the critical thing here is that it is built in.
Another wonderful thing is tests don't have to be written in some DSL.
That is possible only because the language is both remarkably expressive and open source contributors to the language recognized that testing should be first class and built these tools.

## Testing as a User

I came from the world of C++, and every little thing you "should do" when it came to testing often went into the "nice to have" bucket.
The nice-to-have bucket rarely gets funding unless you are fortunate.
You just don't have time.
Don't get me wrong, some teams have time, but most don't.

I believe all teams should have nice things, and one thing I'm a huge proponent of is testing your public interface.
Unless you came from a rather mature C++ team, you might not even know what I mean by that.

Testing your public interface means writing code that tests using your code in the exact way your users can use your code from outside your package.

This comes for free in Rust.
Write rust files in a tests directory, and cargo picks them up and builds each as if they are separate crates that depend on your crate.
From these, you use your library precisely as your users do.

This means that you catch ergonomic issues instead of your users.
These tests are much less about picking apart the tiny interior details and more about seeing your library as an outside might.

But that's not all.
Rust has doc tests.
No [not like that](https://github.com/doctest/doctest).
This is something magical I've never seen before in another language.
In Rust, you write code examples in the doxygen-like comments that get built into the docs your users see, and as part of the cargo test run, it builds and runs all these tiny programs.
This means that unlike other kinds of documentation that always seems to be out of date, if your tests pass, the code in your documentation is up to date.

```Rust
/// Calculate the index into the 1d array from 2d coordinates and a width
///
/// ```
/// # use sample_data_loader::idx;
/// let width = 4;
/// let index = idx(width, 2, 1);
/// # assert_eq!(index, 6);
/// ```
pub fn idx(width: usize, x: usize, y: usize) -> usize {
    width * y + x
}
```

{{ img_caption(path="/images/20241106_doctest_docs.png", caption="cargo doc --open") }}

One really cool thing I want to point out is you can use the `#` character at the start of lines, which is essential to make the code compile, but you don't want to show up in your example.
Another cool thing is a test body, and the main is generated for you, so you can write just the critical parts to show your users how it works.
You'll notice I included an assertion at the end to make this test fail if the value changed, but I didn't bother showing that to the user.

## Who Watches the Watchers

The built-in Rust testing tooling is incredible and a solid foundation for writing fantastic software.
But that's not the whole story; there is more.
This last month, I started doing the sort of testing in Rust which I would have only been able to talk about in theory in C++.

How do you answer the question, "Do these tests measure the behaviors I intended to build?" Before Rust, the answer was only "by inspection".
That is slow, what if the computer could help you answer that question.

In Rust, we even test the tests.
My team has implemented using [cargo-mutants](https://mutants.rs/), which mutate our code, run the tests, and record if the tests caught the mutation.
We are not at the point of enforcing no mutations, but this has helped us find subtle bugs and holes in our testing.

As programmers, we rarely see substantial technological advancements that improve our productivity.
Rust will be the language for system software for the next 40 years because it makes writing robust software first-class.

If that's exciting to you, please [apply to join my team](https://apply.workable.com/scitec/j/DC27C4C558/).
If that link is not working as we've filled it out, here is the [open positions page](https://scitec.com/join/#positions) where I'm working.
