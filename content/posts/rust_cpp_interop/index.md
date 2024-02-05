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

Before we can talk about how we must discuss why.
I will presume these things:

- You want to work in Rust
- You have coworkers who use C++

This is the situation I find myself in.
The primary objections I heard using Rust in projects at work are social and not technical in nature.
Here are some that I have heard:

> It is easier for everyone on the project if we use one language. Using multiple languages will make it harder for me to contribute. I fear that if we write it in Rust we will not be able to hire people to work on this codebase.

Research has demonstrated that [diversity leads to better business outcomes](https://www.mckinsey.com/capabilities/people-and-organizational-performance/our-insights/why-diversity-matters).
It is also better for our projects.
Homogeneous projects may feel more comfortable.
The fear of *the other* and the desire for comfort is the underlying motivations in these statements.

Programming is a craft and the quality of the outcome of our work is improved by working together from different perspectives.
C++, the projects built in it, and the programmers who use it have value.
Rust brings new ideas and new perspectives and building a bridge within our projects can lead to code-bases that are better than they were as homogeneous projects.

Writing software is a team sport where we want to welcome a diversity of ideas and approaches to find the best solutions to any given problem.
Even if you wanted to rewrite a large C++ project into Rust that is unlikely to be possible given project timelines and the makeup of your team.
If you have a C++ codebase you likely have C++ programmers as coworkers and you will be more likely to win their support if you build a bridge.

## Code Generation

There are a handful of amazing projects aimed at automating the process of creating bridges towards and from C++.

- [cxx](https://cxx.rs/) -- Safe interop between Rust and C++
- [bindgen](https://rust-lang.github.io/rust-bindgen/) -- generate Rust FFI bindings to C/C++ libraries
- [cbindgen](https://github.com/mozilla/cbindgen) -- generate C/C++11 headers for Rust librararies which expose a public C API

Due to my desire to create interfaces involving library types in Rust and C++ that felt first class in both languages none of these tools met my requirements.
At PickNik we write robotics code and much of C++ code uses [Eigen](https://eigen.tuxfamily.org/index.php?title=Main_Page) types.
In Rust I wanted to use [nalgebra](https://docs.rs/nalgebra/latest/nalgebra/) types to represent the same concepts.

## On the Shoulders of Giants

[OptIk](https://github.com/kylc/optik) is the project I learned much of this from.
Look to it for a complete example of these techniques.

## System Design

Interop to C++ is done via the classic hourglass approach.
We bridge the Rust library to C and then create C++ types that safely use the C interface.
This is the same way interop to other languages such as Python works.

For C++ to call Rust functions that take Rust objects as arguments we need a way for C++ to have those Rust objects.
To do this we must create Rust objects and leak pointers to them to the C++ code.
We also include functions in Rust that can destruct these objects given a pointer to one.
From these two building blocks we can use a C++ class which holds the opaque pointers and manages freeing them using the C++ destructor.
One important reason this is necessary is that allocators and deallocators come in pairs.
It is not valid to destruct a Rust object with the C++ deallocator or vice-versa.

In cases where we need to create C++ library types from Rust library types such as creating a `Eigen::Isometry3d` from a `nalgebra::geometry::Isometry3` we must copy the underlying data instead of sharing the memory.
This is because in C++ there is no way for us to extend a library type to handle destruction of the underlying memory using a different deallocator.
Similar to the custom type situation above with the opaque types we must have an unsafe rust function to free the memory for the Rust types and call it before returning from our C++ function.

There is the concnern of how do we integrate with a C++ build system.
As the C++ code at my work uses CMake I will show how to bridge to CMake.

For code-layout I'm going to separate my project into two different Rust crates (packages).

- `robot_joint` -- Rust library I want to use from C++
- `robot_joint-cpp` -- C++ interop layer


## Custom Opaque Types

Given this Rust struct and factory function we need to create a C interface.
```rust
pub struct Joint {
    name: String,
    parent_link_to_joint_origin: Isometry3<f64>,
}

impl Joint {
    pub fn new() -> Self;
}
```

Over in `robot_joint-cpp` I create a `lib.rs` with these details:
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

Each of these functions need the `#[no_mangle]`  attribute to turn off Rust name mangling and `extern "C"` to give the function the C calling convention.
`Box::into_raw(Box::new(` is a technique for creating a Rust object on the heap and leaking a pointer to it.
Lastly, `drop(Box::from_raw` is a way to take a pointer and convert it back into a Box type and destroy it.

Next we create a C++ header `robot_joint.hpp`:
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
other
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

Lastly, and perhaps the hardest part we need to make this compatible with CMake projects.
Here is a complete example with all the various moving parts from Kyle's OptIk library:

- [CMakeLists.txt](https://github.com/kylc/optik/blob/ea584bfea4c702e52039d2cb09536a9513414121/crates/optik-cpp/CMakeLists.txt#L1)
- [cmake/optikConfig.cmake.in](https://github.com/kylc/optik/blob/ea584bfea4c702e52039d2cb09536a9513414121/crates/optik-cpp/cmake/optikConfig.cmake.in#L1) - rename this file appropriately for your project
- [examples/CMakeLists.txt](https://github.com/kylc/optik/blob/ea584bfea4c702e52039d2cb09536a9513414121/examples/CMakeLists.txt#L1) - how to consume from downstream CMake project

## First-class Library Types

Remember I said I took the manual approach because I wanted an interface with Eigen types on the C++ side.
Here is a simple example of how to accomplish that.
Presume we have this Rust function on our `Joint` type:
```rust
impl Joint {
    pub fn calculate_transform(&self, variables: &[f64]) -> Isometry3<f64>;
}
```

We want to create a C++ interface like this:

```C++
class Joint {
  public:
    Eigen::Isometry3d calculate_transform(const Eigen::VectorXd& variables);
};
```

First we must create the Rust FFI interface to this function:
```rust
use std::ffi::{c_double, c_uint};

#[repr(C)]
struct RawVecDouble {
    ptr: *mut c_double,
    length: usize,
    capacity: usize,
}

#[no_mangle]
extern "C" fn robot_joint_calculate_transform(
    joint: *const Joint,
    variables: *const c_double,
    size: c_uint,
) -> RawVecDouble {
    unsafe {
        let joint = joint.as_ref().expect("Invalid pointer to Joint");
        let variables = std::slice::from_raw_parts(variables, size as usize);
        let transform = joint.calculate_transform(variables);
        let transform = transform.to_matrix().data.as_slice().to_vec();
        let length = transform.len();
        let capacity = transform.capacity();
        RawVecDouble {
            ptr: transform.leak().as_mut_ptr(),
            length,
            capacity,
        }
    }
}

#[no_mangle]
extern "C" fn vector_free(vector: RawVecDouble) {
    unsafe {
        drop(Vec::<f64>::from_raw_parts(
            vector.ptr,
            vector.length,
            vector.capacity,
        ));
    }
}
```

C types we need for parameters come from the [ffi module](https://doc.rust-lang.org/std/ffi/index.html) in the Rust standard library.
Before calling the rust `calculate_transform` we first need to construct the Rust types from the parameters.
At the point of return we leak the memory as a mutable raw pointer.

Then we can write a C++ function that calls the C functions:
```C++
extern "C" {
struct RawVecDouble {
    double* ptr;
    size_t length;
    size_t capacity;
};

extern const double* robot_joint_calculate_transform(const robot_joint::rust::Joint*, const double*, unsigned int);
extern void vector_free(const RawVecDouble*);
}

namespace robot_joint {
Eigen::Isometry3d Joint::calculate_transform(const Eigen::VectorXd& variables)
{
    const auto data = robot_joint_calculate_transform(joint_, variables.data(), variables.size());
    Eigen::Isometry3d t;
    t.matrix() = Eigen::Map<Eigen::Matrix4d>(data.ptr);
    vector_free(data);
    return t;
}
}  // namespace robot_joint
```

This approach involves several type conversions.
I first convert the Rust `Isometry3` type into a rust `Vec` then I store the details of the vector in a struct `RawVecDouble` and return that through my C interface.
The C++ code receives this pointer and constructs an Eigen type by copying data pointed to.
This is possible because both the Rust `Isometry3` and C++ `Isometry3d` types are backed by a column major 4x4 matrix of doubles.
Lastly, I call a rust function to free the vector.

## Conclusion

You will likely have more trouble convincing your C++ loving coworkers to let you write code in Rust than doing the interop.
Building a bridge that creates both nice C++ and Rust interfaces is not as hard as many think.

## Future Work

Code without tests should be considered broken.
To trust all this unsafe C++ and Rust code we should write tests that exercise all the code paths and run them with sanitizers.

## References

- [The Rustnomicon](https://doc.rust-lang.org/nomicon/) -- The dark arts of unsafe Rust
- [kylec/optick](https://github.com/kylc/optik) -- Rust IK solver with C++ and Rust bindings
