+++
title = "Rust/C++ Interop Part 3 - Cxx"
date = 2024-02-16
type = "post"
description = "Robots are Friendly"
in_search_index = true
[taxonomies]
tags = ["Rust", "C++", "CMake", "Cxx"]
+++

I'm not a stranger to using code generation for repetitive and error-prone tasks. In 2023, I [presented at ROSCON](/posts/roscon23-parameters) about a code-generation tool I helped author that takes a yaml file and emits a C++ library for configuration declaration in ROS.

[Cxx](https://cxx.rs/) is the most well-known code generation tool for C++ <-> Rust interop. From the `cxx` docs:

> This library provides a safe mechanism for calling C++ code from Rust and Rust code from C++. It carves out a regime of commonality where Rust and C++ are semantically very similar and guides the programmer to express their language boundary effectively within this regime.

I did not use `cxx` when I started this journey in [Part 1](/posts/rust-cpp-interop). I wanted first-class library types in my interfaces. I did achieve keeping those library types in the function parameters of my API, but in exchange, I had to write a lot of boilerplate and unsafe code. As this unsafe code is time-consuming to write and test, I wrote the minimum I needed for my application.

{{ img_caption(path="/images/640px-Db_tuda_jes2899_a.jpg", caption="Cobot") }}

The minimal C++ API left my C++ coworkers feeling like they received a second-class API for my library. The unsafe code and the amount of boilerplate in the interface also made them think that they didn't want to build more of that themselves to expand the API.

When building software systems with others, we must consider the human costs of our technical decisions. In this case, while the minimal hand-written unsafe interop worked for the one project, it missed the mark of making a library where developing in C++ and Rust both feel first class.

## Enter the Robots
The basic idea of the `cxx` crate is that it takes code you write in Rust and C++ and uses macros to expand into the interop layer. It only works with a limited number of types and might improve over time. This limited number of types makes the challenge of first-class library types in parameters harder. Here, I'll show you how I've approached this problem.

From [Part 1](/posts/rust-cpp-interop#custom-opaque-types), we have our Joint type. I've expanded it to show off some more features of `cxx`.

### crates/robot_joint/src/joint.rs
```rust
#[derive(Clone, Debug)]
pub struct Joint {
    pub name: String,
    pub parent_link_to_joint_origin: Isometry3<f64>,
    pub parent_link_index: usize,
    pub child_link_index: usize,
    pub index: usize,
    pub dof_index: usize,
}

impl Joint {
    pub fn new() -> Self;
    pub fn calculate_transform(&self, variables: &[f64]) -> Isometry3<f64>;
}
```

Before showing the cxx interop code, I need to point out some more details and choices I made. The Rust library we are binding to is a pure Rust. I'm going to create a separate crate for the cxx interop layer. This choice means that I need to do a sort re-export of the types from my Rust crate in my cxx crate.

### crates/robot_joint-cxx/src/lib.rs Part 1
```rust
struct Joint(robot_joint::joint::Joint);

#[cxx::bridge(namespace = "robot_joint")]
mod ffi {
    extern "Rust" {
        type Joint;
        fn new_joint() -> Box<Joint>;
        fn name(self: &Joint) -> String;
        fn parent_link_to_joint_origin(self: &Joint) -> Vec<f64>;
        fn parent_link_index(self: &Joint) -> usize;
        fn child_link_index(self: &Joint) -> usize;
        fn index(self: &Joint) -> usize;
        fn dof_index(self: &Joint) -> usize;
        fn calculate_transform(self: &Joint, variables: &[f64]) -> Vec<f64>;
        fn to_string(self: &Joint) -> String;
    }
}
```

As my Joint type is from a pure Rust crate, it does not have a C memory layout, so we can't share the type itself. Instead, we must create an [opaque type](https://cxx.rs/extern-rust.html#opaque-rust-types). Cxx does not allow you to generate opaque interop for Rust types defined in another crate. The way I found around this is a [newtype pattern](https://doc.rust-lang.org/rust-by-example/generics/new_types.html).

The second thing to see is that I'm going to write getters for each of the members of the Joint type. The implementation for those is in the following section. Lastly, You'll see that we do not have Nalgebra types in our interfaces. We have instead exposed `Vec<f64>` in place of the Nalgebra types.

### crates/robot_joint-cxx/src/lib.rs Part 2
```rust
fn new_joint() -> Box<Joint> {
    Box::new(Joint(robot_joint::joint::Joint::new()))
}

impl Joint {
    fn name(&self) -> String {
        self.0.name.clone()
    }

    fn parent_link_to_joint_origin(&self) -> Vec<f64> {
        convert::vec_from_isometry3(self.0.parent_link_to_joint_origin)
    }

    fn parent_link_index(&self) -> usize {
        self.0.parent_link_index
    }

    fn child_link_index(&self) -> usize {
        self.0.child_link_index
    }

    fn index(&self) -> usize {
        self.0.index
    }

    fn dof_index(&self) -> usize {
        self.0.dof_index
    }

    fn calculate_transform(&self, variables: &[f64]) -> Vec<f64> {
        convert::vec_from_isometry3(self.0.calculate_transform(variables))
    }
}

impl std::fmt::Display for Joint {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:#?}", self.0)
    }
}
```

Holly boilerplate batman. There is probably some clever macro I could write or use to avoid writing implementations that abstract through the newtype to internal members.

The `Display` implementation here makes a `to_string` function exist that we can bind through.

Keen eyes will see a function I'm using here I haven't implemented yet. `convert::vec_from_isometry3` is a function that makes a `Vec<f64>` from an `Isometry3<f64>`. You'll see that next.

### crates/robot_joint-cxx/src/lib.rs Part 2
```rust
mod convert {
    use nalgebra::{Isometry3, Matrix6xX};
    pub fn isometry3_from_slice(data: &[f64]) -> Isometry3<f64> {
        nalgebra::try_convert(nalgebra::Matrix4::from_column_slice(data))
            .expect("Invalid isometry!")
    }

    pub fn vec_isometry3_from_vec(data: Vec<f64>) -> Vec<Isometry3<f64>> {
        data.chunks(16).map(isometry3_from_slice).collect()
    }

    pub fn vec_from_isometry3(transform: Isometry3<f64>) -> Vec<f64> {
        transform.to_matrix().data.as_slice().to_vec()
    }

    pub fn vec_from_vec_isometry3(transforms: Vec<Isometry3<f64>>) -> Vec<f64> {
        transforms
            .into_iter()
            .flat_map(|t| t.to_matrix().data.as_slice().to_vec())
            .collect::<Vec<f64>>()
    }

    pub fn vec_from_matrix6x(matrix: Matrix6xX<f64>) -> Vec<f64> {
        matrix.data.as_vec().to_owned()
    }
}
```

I've shown more here than we used above to demonstrate how this works and used it for my more extensive library. One thing to point out about this approach is that going through interop involves two copies of these types. One from a Rust library type to primitive types that `cxx` supports. Secondly, on the C++ side, you have to do it again, making a copy from the primitive types to C++ library types. I have benchmarked this, and compared to the time it takes to do the math, this time could be more meaningful for my application.

## C++ header

To make this interop work, we need functions to convert the primitive types into C++ library types. I've put these in a header-file that includes the generated header I'm going to export as part of my CMake target.

{{ img_caption(path="/images/ICub_sciencefestival_1.jpg", caption="ICub") }}

### crates/robot_joint-cxx/include/robot_joint/robot_joint.hpp
```c++
#pragma once

#include <robot_joint/lib.h> // generated by cxx
#include <rust/cxx.h>
#include <Eigen/Geometry>

namespace robot_joint {
constexpr auto kMatrix4dLen = sizeof(Eigen::Matrix4d) / sizeof(double);

template <typename T, typename V>
rust::Slice<T> to_rust_slice(V const& vec) {
  return rust::Slice<T>(vec.data(), vec.size());
}

rust::Slice<const double> to_rust_slice(const Eigen::Isometry3d& transform) {
  return rust::Slice<const double>(transform.matrix().data(), 16);
}

rust::Vec<double> to_rust_vec(
    const std::vector<Eigen::Isometry3d>& transforms) {
  rust::Vec<double> vec;
  vec.reserve(16 * transforms.size());
  for (const auto& t : transforms) {
    auto* matrix = t.matrix().data();
    for (auto i = 0; i < 16; ++i) {
      vec.push_back(matrix[i]);
    }
  }
  return vec;
}

std::vector<Eigen::Isometry3d> to_c_isometry_vector(
    rust::Vec<double>&& raw_vec) {
  auto const n_transforms = raw_vec.size() / kMatrix4dLen;
  std::vector<Eigen::Isometry3d> transforms;
  transforms.reserve(n_transforms);
  for (size_t i = 0; i < n_transforms; ++i) {
    double* ptr = raw_vec.data() + (i * kMatrix4dLen);
    Eigen::Isometry3d t;
    t.matrix() = Eigen::Map<Eigen::Matrix4d>(ptr);
    transforms.push_back(t);
  }
  return transforms;
}

Eigen::Isometry3d to_c_isometry(rust::Vec<double>&& raw_vec) {
  Eigen::Isometry3d transform;
  transform.matrix() = Eigen::Map<Eigen::Matrix4d>(raw_vec.data());
  return transform;
}

template <typename T>
std::vector<T> to_c_vector(const rust::Vec<T>& rust_vec) {
  std::vector<T> cpp_vec;
  std::copy(rust_vec.begin(), rust_vec.end(), std::back_inserter(cpp_vec));
  return cpp_vec;
}
}  // namespace robot_joint
```

Again, I've included more conversion functions than we need for the `Joint` type.

## Example
### crates/robot_joint-cxx/example/example.hpp
```c++
#include <julien/julien.hpp>
#include <Eigen/Geometry>
#include <iostream>
#include <string>

use robot_joint::to_c_isometry;
use robot_joint::to_rust_slice;

int main() {
  auto joint = robot_joint::new_joint();
  std::cout << "Joint:\n" << std::string(joint->to_string()) << "\n";

  Eigen::VectorXd variables = Eigen::VectorXd::Zero(1);
  auto const joint_transform = to_c_isometry(
      joint->calculate_transform(to_rust_slice<const double>(variables)));
  std::cout << "joint_transform (0):\n" << joint_transform.matrix() << "\n";
}
```

Here, we see the cost of not using first-class types in the interface. The user of this interface has to use methods to convert parameters to rust types and to convert the return types.

One cool thing is that `cxx` has support for slice types so we can make a rust slice (think C++ iterator) that can be used on existing C++ memory. This is really efficient and doesn't copy the underlying data.

## crates/robot_joint-cxx/example/CMakeLists.txt
```cmake
cmake_minimum_required(VERSION 3.16)
project(robot_joint_cxx_example)

find_package(Eigen3 REQUIRED)

include(FetchContent)
FetchContent_Declare(
  robot_joint
  SOURCE_DIR
  "${CMAKE_CURRENT_LIST_DIR}/../../.."
  SOURCE_SUBDIR
  "crates/robot_joint-cxx")
FetchContent_MakeAvailable(robot_joint)

add_executable(example example.cpp)
target_link_libraries(example PRIVATE robot_joint::robot_joint)
target_link_libraries(example PUBLIC Eigen3::Eigen)
```

As you can see, we are arriving in the same place as before with our CMake. The example here can use FetchContent to specify the rust interop library as a dependency and link with the resulting CMake target.

## CMake and Cargo

Lastly, we look at the cmake to glue this together. Again, we are using the excellent cmake module [Corrosion](https://corrosion-rs.github.io/corrosion/) to build our rust library.

{{ img_caption(path="/images/Laproscopic_Surgery_Robot.jpg", caption="Laproscopic Surgery Robot") }}

### crates/robot_joint-cxx/CMakeLists.txt
```cmake
cmake_minimum_required(VERSION 3.16)
project(robot_joint CXX)

find_package(Eigen3 REQUIRED)

include(FetchContent)
FetchContent_Declare(
  Corrosion
  GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
  GIT_TAG v0.4)
FetchContent_MakeAvailable(corrosion)

corrosion_import_crate(MANIFEST_PATH Cargo.toml CRATES robot_joint-cxx)
corrosion_add_cxxbridge(robot_joint CRATE robot_joint-cxx FILES lib.rs)
set_property(TARGET robot_joint PROPERTY CXX_STANDARD 20)
target_link_libraries(robot_joint PUBLIC Eigen3::Eigen)
target_include_directories(
  robot_joint PUBLIC $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
                $<INSTALL_INTERFACE:include>)

add_library(robot_joint::robot_joint ALIAS robot_joint)

include(CMakePackageConfigHelpers)
include(GNUInstallDirs)
install(
  TARGETS robot_joint robot_joint-cxx
  EXPORT ${PROJECT_NAME}Targets
  RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
  LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
  ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR})
install(
  EXPORT ${PROJECT_NAME}Targets
  NAMESPACE robot_joint::
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")

configure_package_config_file(
  cmake/robot_jointConfig.cmake.in
  "${PROJECT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
  INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")
install(FILES "${PROJECT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
        DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")

install(DIRECTORY include/ DESTINATION include)
```

Before you ask why I didn't include the content of `cmake/robot_jointConfig.cmake.in`, it is the same as I explained in [Part 2](/posts/rust-cmake-interop-cmake/#install). Go copy it from there if you need it.

### crates/robot_joint-cxx/build.rs
```rust
#[allow(unused_must_use)]
fn main() {
    cxx_build::bridge("src/lib.rs")
        .file("include/robot_joint/robot_joint.hpp")
        .std("C++20")
        .flag_if_supported("-std=c++20");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/robot_joint/robot_joint.hpp");
}
```

One unique aspect of this approach is the need to write a `build.rs` script. This is the escape hatch in cargo builds often used for code generation. The script is largely copied from the docs of `cxx_bridge`.

### crates/robot_joint-cxx/Cargo.toml
```toml
[package]
name = "robot_joint-cxx"
authors.workspace = true
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
crate-type = ["staticlib"]

[dependencies]
robot_joint = { path = "../robot_joint" }
cxx = "1.0"
nalgebra = "0.32.3"

[build-dependencies]
cxx-build = "1.0"
```
{{ img_caption(path="/images/640px-IRobot_Roomba_870_(15860914940).jpg", caption="iRobot") }}

# Conclusion
This approach has some clear benefits. No unsafe code. We used robots to do that part, and in exchange, we were able to build a more full-featured C++ API.

## References
- [C++ Interop Part 1 - Just the Basics](/posts/rust-cpp-interop)
- [C++ Interop Part 2 - CMake](/posts/rust-cmake-interop-cmake/)
- [Cxx](https://cxx.rs/)
- [Corrosion](https://corrosion-rs.github.io/corrosion/)
- [Cobot Image](https://en.wikipedia.org/wiki/Cobot) - Wikipedia
- [ICub Image](https://commons.wikimedia.org/wiki/File:ICub_sciencefestival_1.jpg) - Wikipedia
- [Surgery Robot Image](https://commons.wikimedia.org/wiki/File:Laproscopic_Surgery_Robot.jpg) - Wikipedia
- [iRobot Image](https://commons.wikimedia.org/wiki/File:IRobot_Roomba_870_(15860914940).jpg) - Wikipedia
