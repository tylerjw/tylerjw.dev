+++
title = "Boulder Rust - C++ Interop"
date = 2024-02-04
type = "post"
description = "Together We Are Better"
in_search_index = true
[taxonomies]
tags = ["Talks"]
+++

## A Collective Craft

Before we talk about how we must discuss why.
I will presume these things:

- You want to work in Rust
- You have coworkers who use C++

This is the situation I find myself in.
The primary objections I heard about using Rust in projects at work are social and not technical.
Here are some that I have heard:

> Using one language makes the project more accessible for everyone. Using multiple languages will make it harder for me to contribute. If we write it in Rust, it will make hiring people to work on this project challenging.

Research has demonstrated that [diversity leads to better business outcomes](https://www.mckinsey.com/capabilities/people-and-organizational-performance/our-insights/why-diversity-matters).
It is also better for our projects.
The underlying sentiment in these statements is the fear of *the other*.
Instead of criticizing coworkers for their fear, I have found it helpful to focus on how building a heterogeneous codebase can lead to a better project.

Programming is a craft, and the quality of the outcome of our work is improved by working together from different perspectives.
C++, the projects built in it, and the programmers who use it have value.
Rust brings new ideas and perspectives, and building a bridge within our projects can lead to better codebases than they were as homogeneous projects.

Writing software is a team sport where we want to welcome a diversity of ideas and approaches to find the best solutions to any given problem.
Even if you wanted to rewrite a large C++ project into Rust, that is unlikely to be possible given project timelines and the makeup of your team.
If you have a C++ codebase, you likely have C++ programmers as coworkers, and building a bridge will make you more likely to win their support.

## Code Generation

A handful of well-known projects aim to automate creating bridges to and from C++.

- [cxx](https://cxx.rs/) -- Safe interop between Rust and C++
- [bindgen](https://rust-lang.github.io/rust-bindgen/) -- generate Rust FFI bindings to C/C++ libraries
- [cbindgen](https://github.com/mozilla/cbindgen) -- generate C/C++11 headers for Rust libraries which expose a public C API

Due to my desire to create interfaces involving library types in Rust and C++ that felt first class in both languages none of these tools met my requirements.
At PickNik we write robotics code and much of C++ code uses [Eigen](https://eigen.tuxfamily.org/index.php?title=Main_Page) types.
In Rust I wanted to use [nalgebra](https://docs.rs/nalgebra/latest/nalgebra/) types to represent the same concepts.

## On the Shoulders of Giants

[OptIk](https://github.com/kylc/optik) is the project I learned much of this from.
Look at it for a complete example of these techniques.

## System Design

Interop to C++ is done via the classic hourglass approach.
We bridge the Rust library to C and create C++ types that safely use the C interface.
This is the same way interop to other languages such as Python works.

{{ img_caption(path="/images/hourglass_rust_cpp.png", caption="Hourglass Pattern") }}

We need a way for C++ to have those Rust objects to call Rust functions that take Rust objects as arguments.
We must create Rust objects and leak pointers to the C++ code to do this.
We also include functions in Rust that can destroy these objects, given a pointer to one.
We can use a C++ class with opaque pointers to Rust objects as members, which takes care of freeing them using its destructor method.
One important reason this is necessary is that allocators and deallocators come in pairs.
It is not valid to destroy a Rust object with the C++ deallocator or vice-versa.

In cases where we must create C++ library types from Rust library types, such as making an `Eigen::Isometry3d` from a `nalgebra::geometry::Isometry3`, we must copy the underlying data instead of sharing the memory.
This is because, in C++, we cannot extend a library type to handle the destruction of the underlying memory using a different deallocator.

In the particular case of the Rust homogeneous transform type `nalgebra::geometry::Isometry3`, the underlying data is a 4x4 matrix of doubles represented by a single array of 16 doubles.
A fixed-size array is something that we can pass across the FFI boundary.
We'll take advantage of this to avoid making extra copies or allocations.

There is a concern about how we integrate with a C++ build system.
As the C++ code at my work uses CMake, I will link to an example showing how to make this C++ project consumable by other CMake projects.

I will separate my project into two Rust crates (packages) for code layout.

- `robot_joint` -- Rust library I want to use from C++
- `robot_joint-cpp` -- C++ interop layer

## Custom Opaque Types

Given this Rust struct and factory function, we must create a C interface.
```rust
pub struct Joint {
    name: String,
    parent_link_to_joint_origin: Isometry3<f64>,
}

impl Joint {
    pub fn new() -> Self;
}
```

Over in `robot_joint-cpp`, I create a `lib.rs` with these details.
```rust
use robot_joint::Joint;

#[no_mangle]
extern "C" fn robot_joint_new() -> *mut Joint {
    Box::into_raw(Box::new(Joint::new()))
}

#[no_mangle]
extern "C" fn robot_joint_free(joint: *mut Joint) {
    unsafe {
        drop(Box::from_raw(joint));
    }
}
```

Each function needs the `#[no_mangle]`  attribute to turn off Rust name mangling and `extern "C"` to give the function the C calling convention.
`Box::into_raw(Box::new(` is a technique for creating a Rust object on the heap and leaking a pointer to it.
Lastly, `drop(Box::from_raw)` is a way to take a pointer, convert it back into a Box type, and destroy it.

Next, we create a C++ header `robot_joint.hpp`.
```C++
namespace robot_joint {
namespace rust {
// Opaque type for holding pointer to rust object
struct Joint;
}

class Joint {
  public:
    Joint();
    ~Joint();

    // Disable copy as we cannot safely copy opaque pointers to rust objects.
    Joint(Joint& other) = delete;
    Joint& operator=(Joint& other) = delete;

    // Explicit move.
    Joint(Joint&& other);
    Joint& operator=(Joint&& other);

  private:
    rust::Joint joint_ = nullptr;
};

}  // namespace robot_joint
```

Here, we create the source file for our C++ interface.
Note how we use `extern "C"` to enable our C++ code to call the C functions from our Rust code.
This is something we are manually keeping in sync.
Had we used one of the previously linked-to code-generators, we would not have had to do this.

The constructor calls the Rust function that creates the `Joint` type and stores the pointer in the member `joint_`.
The move constructor and assignment functions make this C++ type-safe to move by never creating two copies of the internal pointer.
Lastly, the destructor frees the rust `joint_` object by calling the Rust function, which drops the memory.
```C++
#include "robot_joint.hpp"

extern "C" {
extern robot_joint::rust::Joint* robot_joint_new();
extern void robot_joint_free(robot_joint::rust::Joint*);
}

namespace robot_joint {

Joint::Joint() : joint_(robot_joint_new()) {}

Joint::Joint(Joint&& other) : joint_(other.joint_) {
  other.joint_ = nullptr;
}

Joint& Joint::operator=(Joint&& other) {
  joint_ = other.joint_;
  other.joint_ = nullptr;
  return *this;
}

Joint::~Joint() {
  if (joint_ != nullptr) {
    robot_joint_free(joint_);
  }
}

}  // namespace robot_joint
```

Lastly, the most challenging part is to make this compatible with CMake projects.
Here is a complete example with all the various moving parts from Kyle's OptIk library.

- [CMakeLists.txt](https://github.com/kylc/optik/blob/ea584bfea4c702e52039d2cb09536a9513414121/crates/optik-cpp/CMakeLists.txt#L1)
- [cmake/optikConfig.cmake.in](https://github.com/kylc/optik/blob/ea584bfea4c702e52039d2cb09536a9513414121/crates/optik-cpp/cmake/optikConfig.cmake.in#L1) - rename this file appropriately for your project
- [examples/CMakeLists.txt](https://github.com/kylc/optik/blob/ea584bfea4c702e52039d2cb09536a9513414121/examples/CMakeLists.txt#L1) - how to consume from downstream CMake project

## First-class Library Types

Remember, I said I took the manual approach because I wanted an interface with `Eigen` types on the C++ side.
Here is a simple example of how to accomplish that.
Presume we have this Rust function on our `Joint` type.
```rust
impl Joint {
    pub fn calculate_transform(&self, variables: &[f64]) -> Isometry3<f64>;
}
```

We want to create a C++ interface like this.
```C++
class Joint {
  public:
    Eigen::Isometry3d calculate_transform(const Eigen::VectorXd& variables);
};
```

First, we must create the Rust FFI interface for this function.
```rust
use std::ffi::{c_double, c_uint};

#[repr(C)]
struct Mat4d {
    data: [c_double; 16],
}

#[no_mangle]
extern "C" fn robot_joint_calculate_transform(
    joint: *const Joint,
    variables: *const c_double,
    size: c_uint,
) -> Mat4d {
    unsafe {
        let joint = joint.as_ref().expect("Invalid pointer to Joint");
        let variables = std::slice::from_raw_parts(variables, size as usize);
        let transform = joint.calculate_transform(variables);
        Mat4d {
            data: transform.to_matrix().as_slice().try_into().unwrap(),
        }
    }
}
```

C types we need for parameters come from the [ffi module](https://doc.rust-lang.org/std/ffi/index.html) in the Rust standard library.
Before calling the rust `calculate_transform`, we first need to construct the Rust types from the parameters.

Interestingly, we use an [undocumented fact that thin pointers can be utilized in ffi](https://github.com/rust-lang/nomicon/issues/23).
A sized slice is a thin pointer that does not store the size at runtime.
We can return a sized slice by value by placing it in a struct and setting the memory representation as `C`.

Then, we can write a C++ function that calls the C functions.
```C++
struct Mat4d {
  double data[16];
};

extern "C" {
extern struct Mat4d robot_joint_calculate_transform(
    const robot_joint::rust::Joint*, const double*, unsigned int);
}

namespace robot_joint {
Eigen::Isometry3d Joint::calculate_transform(const Eigen::VectorXd& variables)
{
    const auto rust_isometry = robot_joint_calculate_transform(
        joint_, variables.data(), variables.size());
    Eigen::Isometry3d transform;
    transform.matrix() = Eigen::Map<Eigen::Matrix4d>(std::move(rust_isometry.data));
    return transform;
}
}  // namespace robot_joint
```

The Rust `Mat4d` type returned from `robot_joint_calculate_transform` contains a fixed-size array of sixteen doubles.
We can type-cast a 4x4 Eigen matrix using this array and assign it to an `Isometry3d`, which we then return.

## Conclusion

Building a bridge that creates excellent C++ and Rust interfaces is more straightforward than many think.
You will likely have more trouble convincing your C++-loving coworkers to let you write code in Rust than doing the interop.

## Future Work

Code without tests should be considered broken.
To trust all this unsafe C++ and Rust code, we should write tests that exercise all the code paths and run them with sanitizers.
In a future post, I'll show you how to use the excellent C++ Catch2 library to test your C++ bindings with addresses and undefined behavior sanitizers.

I also want to explore the idea of relying primarily on the cxx crate for interop and building a C++ interface or extending the macros to handle types like `Isometry3`.
The significant upside is that I can reduce the amount of manually written unsafe code.

## References

- [The Rustnomicon](https://doc.rust-lang.org/nomicon/) -- The dark arts of unsafe Rust
- [kylec/optick](https://github.com/kylc/optik) -- Rust IK solver with C++ and Rust bindings
- [Google investing in Interoperability Between Rust and C++](https://security.googleblog.com/2024/02/improving-interoperability-between-rust-and-c.html?m=1)
- [Passing nothing is surprisingly difficult](https://davidben.net/2024/01/15/empty-slices.html) _by David Benjamin_ -- A gotcha for passing slices between C, C++, and Rust
- [cxx](https://cxx.rs/) -- Safe interop between Rust and C++
- [bindgen](https://rust-lang.github.io/rust-bindgen/) -- generate Rust FFI bindings to C/C++ libraries
- [cbindgen](https://github.com/mozilla/cbindgen) -- generate C/C++11 headers for Rust libraries which expose a public C API
