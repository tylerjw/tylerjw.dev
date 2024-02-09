+++
title = "Rust/C++ Interop Part 2 - First-class CMake User Experience"
date = 2024-02-09
type = "post"
description = "Love Thy Neighbor"
in_search_index = true
[taxonomies]
tags = ["Rust", "C++", "CMake"]
+++

{{ admonition(kind="note", body="This is a follow-on to [Rust/C++ Interop](/posts/rust-cpp-interop) where we built a bridge between Rust and C++.") }}

As a bridge builder and Rust radical living on a C++ team, CMake is the next layer of this onion you need to peel back.
For this blog post, I'll presume these things:

- You have written some Rust code and done the work to create a C++ interface for it.
- You have coworkers who use CMake to build their C++ projects.
- You would like to write tests in C++ and compile them with sanitizers to identify bugs in your interop layer.

Unlike in Rust, C++ has many different build systems.
I work at [PickNik Robotics](https://picknik.ai/), and we use CMake to build our C++ projects, so I'll cover that here.

{{ admonition(kind="note", body="If you are copy-pasting examples. I've used the standin `<lib-name>` for the name of the library you are building an interface to.") }}

At the end of this, you should have a C++ library that CMake users can depend on using `FetchContent`:
```cmake
include(FetchContent)
FetchContent_Declare(
  <lib-name>
  GIT_REPOSITORY https://github.com/org/<lib-name>
  GIT_TAG main
  SOURCE_SUBDIR "crates/<lib-name>-cpp")
FetchContent_MakeAvailable(<lib-name>)

target_link_libraries(mylib PRIVATE <lib-name>::<lib-name>)
```

## Project Layout
I use a [cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) for Rust projects with interop to other languages. These are the folders and files in the structure:

```
├── Cargo.toml
├── README.md
└── crates
   ├── <lib-name>
   │   ├── Cargo.toml
   │   └── src
   │       └── lib.rs
   └── <lib-name>
       ├── Cargo.toml
       ├── CMakeLists.txt
       ├── cmake
       │   └── <lib-name>Config.cmake.in
       ├── include
       │   └── <lib-name>.hpp
       ├── src
       │   ├── lib.cpp
       │   └── lib.rs
       └── tests
           ├── CMakeLists.txt
           └── tests.cpp
```

## `Cargo.toml`
```toml
[workspace]
resolver = "2"
members = ["crates/<lib-name>", "crates/<lib-name>-cpp"]

[workspace.package]
description = "What is your project about?"
authors = ["Name <email@example.com>"]
version = "0.1.0"
edition = "2021"
license = "BSD-3-Clause"
readme = "README.md"
keywords = [""]
categories = [""]
repository = "https://github.com/org/project/"
```
Here is my root `Cargo.toml`. When I do a cargo build, I want to build both my safe and FFI Rust libraries.

## `crates/<lib-name>-cpp/Cargo.toml`
```toml
[package]
name = "<lib-name>-cpp"
authors.workspace = true
version.workspace = true
edition.workspace = true
license.workspace = true

[lib]
name = "<lib-name>cpp"
crate-type = ["staticlib"]

[dependencies]
<lib-name> = { path = "../<lib-name>" }
```
Here, we tell Cargo how to build the Rust FFI library and use the relative path syntax for how it depends on the pure Rust library. Under the lib section, you should note that we name the library with the cpp suffix, which is essential to separate it from the pure Rust library for Cargo. We drop the dash used in the package name because Cargo does not allow a dash in library names. Finally, we build this as a static lib because we will statically link this into the C++ interop library. This FFI library is a detail we expect users to only depend on or link with. This library should not be published to crates.io.

## `crates/<lib-name>-cpp/CMakeLists.txt`
This is the entry point for building your C++ interop layer and the most complex part of this project. We'll take it in sections.

### Project and Dependencies
```cmake
cmake_minimum_required(VERSION 3.16)
project(<lib-name> VERSION 0.1.0)

find_package(Eigen3 REQUIRED)
```
First, we need to tell CMake the minimum version we depend on. This is a trade-off between choosing an old enough version all your users will have on their systems and a new enough version with all the features you want to use. For my project, 3.16 is the sweet spot because it is the oldest version users will likely have packaged with their Linux install.

Next comes the `project` command. This can tell CMake many things about your project. I've chosen to only include the version. Here are the CMake docs on the project.

After that, you use `find_package` to list the C++ dependencies you need to link into your C++ library. In my case, I'm using `Eigen3`.

### Corrosion - Build the Rust
```cmake
include(FetchContent)
FetchContent_Declare(
  Corrosion
  GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
  GIT_TAG v0.4)
FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH Cargo.toml CRATES <lib-name>-cpp)
```
[Corrosion](https://corrosion-rs.github.io/corrosion/) is a CMake module that knows how to build Rust projects. Here, we use `FetchContent` to retrieve it over the Internet. `corrosion_import_crate` tells CMake how to convert your Rust library into a CMake target. Later, when we make a CMake target that depends on this new target, it will set up the dependency relationship, so building the CMake target will first build the Rust library.
Lastly, an important detail is that I told it to only create a CMake target for the FFI crate. I did this because I want to use the same library name for my CMake target and project as I did for my pure Rust library. If you don't specify this option, Corrosion will create CMake targets for every library building the `Cargo.toml` builds.

### CMake library
```cmake
add_library(<lib-name> STATIC src/lib.cpp)
target_include_directories(
  <lib-name> PUBLIC $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
                $<INSTALL_INTERFACE:include>)
target_link_libraries(<lib-name> PUBLIC Eigen3::Eigen)
target_link_libraries(<lib-name> PRIVATE <lib-name>cpp)
set_property(TARGET julien PROPERTY CXX_STANDARD 20)
set_property(TARGET julien PROPERTY POSITION_INDEPENDENT_CODE ON)
```
Here, we tell CMake how to build our C++ library, where to find the header files, to link it with Eigen3::Eigen, and our rust library `<lib-name>cpp`.
I'm using C++20, so I set that here.
To make links with the Rust static library work, I enable `POSITION_INDEPENDENT_CODE` for this project.

### Install
```cmake
# Install
include(CMakePackageConfigHelpers)
include(GNUInstallDirs)

install(
  TARGETS <lib-name> <lib-name>cpp
  EXPORT ${PROJECT_NAME}Targets
  RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
  LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
  ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR})

install(
  EXPORT ${PROJECT_NAME}Targets
  NAMESPACE <lib-name>::
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")

configure_package_config_file(
  cmake/<lib-name>Config.cmake.in
  "${PROJECT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
  INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")
install(FILES "${PROJECT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
        DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")

install(FILES include/<lib-name>.hpp DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})
```
Next comes the grossest part of this whole thing. This is needed so the CMake target we are creating is consumable by other CMake projects. To use this, copy-paste it and replace all the occurrences of `<lib-name>` with your project's name. You'll also need the file `crates/<lib-name>-cpp/cmake/<lib-name>Config.cmake.in` shown below:

```cmake
@PACKAGE_INIT@

include(CMakeFindDependencyMacro)
find_dependency(Eigen3)

include("${CMAKE_CURRENT_LIST_DIR}/@PROJECT_NAME@Targets.cmake")
```
One important point to make here is that you'll need to add a `find_dependency` for every C++ dependency on which you previously did a `find_package`.

### Testing
```cmake
if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME)
  include(CTest)
  if(BUILD_TESTING)
    add_subdirectory(tests)
  endif()
endif()
```
We only want to build tests when building this as the root CMake project and not when this CMake project is built through `FetchContent` in another project. This is one way to make your project user-friendly by giving the fastest possible build times depending on your library.

## `crates/<lib-name>-cpp/tests/CMakeLists.txt`
```cmake
include(FetchContent)
FetchContent_Declare(
  Catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2.git
  GIT_TAG v3.5.2)
FetchContent_MakeAvailable(Catch2)
include(Catch)

add_executable(tests tests.cpp)
target_link_libraries(tests PRIVATE Catch2::Catch2WithMain
                                                    <lib-name>::<lib-name>)
catch_discover_tests(tests)
```
[Catch2](https://github.com/catchorg/Catch2?tab=readme-ov-file#what-is-catch2) is a beautiful, modern C++ testing framework. The docs explain how to write tests using it. The killer feature of having tests of your C++ interop library is that we can now build with linters and test for mistakes in our unsafe code.

# Building and Testing
After all that boilerplate, we can now build our project using CMake.

```bash
cmake -B build -S crates/<lib-name>-cpp -DCMAKE_BUILD_TYPE=Debug
cmake --build build
ctest --test-dir build --output-on-failure
```
To build and test with sanitizers, add options like these to the first command:
- `-DCMAKE_CXX_FLAGS="-fsanitize=undefined"`
- `-DCMAKE_CXX_FLAGS="-fsanitize=address"`

## GitHub Actions CI
To make this all build in CI here is my `.github/workflows/ci.yaml` file you can copy into your project.

```yaml
name: CI

on:
  workflow_dispatch:
  pull_request:
  push:
    branches:
      - main

jobs:
  cpp:
    name: Cpp
    runs-on: ubuntu-latest
    strategy:
      matrix:
        cxx-flags:
          - "-fsanitize=undefined"
          - "-fsanitize=address"
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: sudo apt install libeigen3-dev
      - name: Configure, Build, and Test Project
        uses: threeal/cmake-action@v1.3.0
        with:
          source-dir: crates/<lib-name>-cpp
          generator: Ninja
          cxx-flags: ${{ matrix.cxx-flags }}
          run-build: true
          run-test: true
```

# Conclusion
Drop this in your README.md and your coworkers who use CMake should find your library easy to use.
```cmake
include(FetchContent)
FetchContent_Declare(
  <lib-name>
  GIT_REPOSITORY https://github.com/org/<lib-name>
  GIT_TAG main
  SOURCE_SUBDIR "crates/<lib-name>-cpp")
FetchContent_MakeAvailable(<lib-name>)

target_link_libraries(mylib PRIVATE <lib-name>::<lib-name>)
```

## References:
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [CMake project command](https://cmake.org/cmake/help/latest/command/project.html)
- [Corrosion](https://corrosion-rs.github.io/corrosion/)
- [Catch2](https://github.com/catchorg/Catch2?tab=readme-ov-file#what-is-catch2)
