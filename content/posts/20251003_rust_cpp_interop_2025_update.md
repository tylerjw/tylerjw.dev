+++
title = "Rust/C++ Interop in 2025: A Complete Example"
date = 2025-10-03
type = "post"
in_search_index = true
[taxonomies]
tags = ["Rust", "C++", "Interop", "CMake", "Cxx", "FFI"]
+++

Nearly a year has passed since I wrote my [original blog post series](/posts/rust-cpp-interop) on Rust/C++ interoperability.
That series covered the fundamentals and was based on code I couldn't share at the time.
Today, I'm excited to share an example that demonstrates not just the concepts, but the complete tooling and project structure needed to make Rust/C++ interop work in practice.

The [rust-cpp-interop-example](https://github.com/tylerjw/tylerjw.dev/tree/main/rust-cpp-interop-example) project showcases both approaches from my original series, manual FFI and cxx-based bindings in a single workspace with build tooling, testing, and real-world examples.

{{ admonition(kind="note", body="You can find the complete example at [rust-cpp-interop-example](https://github.com/tylerjw/tylerjw.dev/tree/main/rust-cpp-interop-example) in my blog's repository.") }}

## What's Changed Since 2024?

### 1. Rust Standards

The example now leverages the latest Rust features:

```rust
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
```

The new Cargo resolver and Rust 2024 edition provide better dependency resolution and feature handling across the workspace.

Rust edition 2024 brought some minor language changes I've included in the examples.
The `no_mangle` attribute needs to now be marked unsafe.
```rust
#[unsafe(no_mangle)]
```

### 2. Project Structure

```
rust-cpp-interop-example/
├── Makefile                   # Unified build system
├── examples/                  # Complete C++ usage examples
│   ├── manual_ffi_example.cpp
│   └── cxx_example.cpp
└── crates/
    ├── robot_joint/           # Pure Rust library
    ├── robot_joint-cpp/       # Manual FFI bindings
    └── robot_joint-cxx/       # Cxx-based bindings
```

This structure makes it easy to compare approaches side-by-side and understand the trade-offs.

### 3. Error Handling

The pure Rust library now uses `thiserror` for proper error handling:

```rust
/// Error types for robot joint operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid joint configuration: {message}")]
    InvalidConfiguration { message: String },

    #[error("Expected {expected} variables, got {actual}")]
    InvalidVariableCount { expected: usize, actual: usize },

    #[error("Mathematical error: {message}")]
    MathError { message: String },
}
```

This provides much better error messages and debugging information compared to the basic `Result` types in the original examples.

### 4. Makefile

To try out the example you can clone the repo and use the make file in the rust-cpp-interop-example directory.
You will need some basic dependencies in your environment like the Rust and C++ toolchains, CMake, and Eigen.

```makefile
make
```

### 5. Testing with Sanitizers

The example includes testing with AddressSanitizer and UndefinedBehaviorSanitizer.
This catches memory safety issues that can easily slip through manual FFI code.

## Comparing the Two Approaches

The biggest improvements in this example is the side-by-side comparison of manual FFI versus cxx-based approaches.

### Manual FFI Example

```cpp
#include <robot_joint.hpp>

int main() {
    // Direct, first-class C++ API
    robot_joint::Joint joint("example_joint");

    Eigen::VectorXd variables(1);
    variables << M_PI / 2.0;

    // Natural Eigen types in the interface
    auto transform = joint.calculate_transform(variables);
    std::cout << transform.matrix() << std::endl;

    // Simple method calls
    auto [min_limit, max_limit] = joint.limits();
    return 0;
}
```

### Cxx-Based Example

```cpp
#include <robot_joint/robot_joint.hpp>

int main() {
    // Factory function required for opaque types
    auto joint = robot_joint::new_joint("cxx_example_joint");

    rust::Vec<double> variables;
    variables.push_back(M_PI / 2.0);

    // Explicit conversions required
    auto transform_vec = joint->calculate_transform(
        robot_joint::to_rust_slice(variables));
    auto transform = robot_joint::to_eigen_isometry3d(
        std::move(transform_vec));

    std::cout << transform.matrix() << std::endl;
    return 0;
}
```

## Enhanced Rust Library

The pure Rust library has more realistic details instead of the stub of a type in the blog series.

### Rich Joint Representation

```rust
#[derive(Clone, Debug)]
pub struct Joint {
    name: String,
    parent_link_to_joint_origin: Isometry3<f64>,
    parent_link_index: usize,
    child_link_index: usize,
    index: usize,
    dof_index: usize,
    axis: Vector3<f64>,
}
```

```rust
impl Joint {
    /// Create with full configuration
    pub fn new_with_config(
        name: String,
        parent_link_to_joint_origin: Isometry3<f64>,
        parent_link_index: usize,
        child_link_index: usize,
        index: usize,
        dof_index: usize,
        axis: Vector3<f64>,
    ) -> Self;

    /// Calculate transform matrix as flat array for FFI
    pub fn calculate_transform_matrix(&self, variables: &[f64]) -> [f64; 16];

    /// Joint limit checking
    pub fn is_within_limits(&self, position: f64) -> bool;
}
```

The `calculate_transform_matrix` method specifically optimizes for FFI use cases by returning the matrix data in the format that C++ expects.

## Modern CMake Integration

The CMake integration has been refined with better dependency handling:

```cmake
# More robust dependency checking
find_package(Eigen3 REQUIRED)

# Better Corrosion integration
include(FetchContent)
FetchContent_Declare(
  Corrosion
  GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
  GIT_TAG v0.5)
FetchContent_MakeAvailable(Corrosion)

# Cleaner target setup
corrosion_import_crate(MANIFEST_PATH Cargo.toml CRATES robot_joint-cpp)
```

And the installation targets are now more robust for downstream consumption.

## Usage Patterns

### For CMake Projects

Both approaches are designed to work seamlessly with CMake's `FetchContent`:

```cmake
include(FetchContent)
FetchContent_Declare(
  robot_joint
  GIT_REPOSITORY https://github.com/tylerjw/tylerjw.dev
  GIT_TAG main
  SOURCE_SUBDIR "rust-cpp-interop-example/crates/robot_joint-cpp"
)
FetchContent_MakeAvailable(robot_joint)

target_link_libraries(your_target PRIVATE robot_joint::robot_joint)
```

## References

- [rust-cpp-interop-example](https://github.com/tylerjw/tylerjw.dev/tree/main/rust-cpp-interop-example) - Complete working example
- [Part 1: Just the Basics](/posts/rust-cpp-interop) - Original blog post series
- [Part 2: CMake Integration](/posts/rust-cmake-interop-cmake)
- [Part 3: Using Cxx](/posts/rust-cmake-interop-part-3-cxx)
- [Part 4: Binding to C++ Libraries](/posts/rust-cpp-part4-buildrs)
- [CppCon 2024 Talk](/posts/20240920-cppcon-cpp-rust-interop) - Video presentation
- [nalgebra](https://nalgebra.rs/) - Linear algebra library for Rust
- [Eigen](https://eigen.tuxfamily.org/) - C++ template library for linear algebra
- [cxx](https://cxx.rs/) - Safe interop between Rust and C++
- [Corrosion](https://corrosion-rs.github.io/corrosion/) - CMake integration for Rust
- [thiserror](https://docs.rs/thiserror/) - Rust error handling made easy
