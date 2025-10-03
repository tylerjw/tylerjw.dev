# Rust/C++ Interop Example

This project demonstrates various patterns for Rust/C++ interoperability, based on the blog post series by Tyler Weaver:

- [Part 1: Just the Basics](tylerjw.dev/posts/rust-cpp-interop)
- [Part 2: CMake Integration](tylerjw.dev/posts/rust-cmake-interop-cmake)
- [Part 3: Using Cxx](tylerjw.dev/posts/rust-cmake-interop-part-3-cxx)
- [Part 4: Binding to C++ Libraries](tylerjw.dev/posts/rust-cpp-part4-buildrs)
- [Part 5: Interop in 2025](tylerjw.dev/posts/20251003-rust-cpp-interop-2025-update)

## Project Structure

This workspace contains three crates demonstrating different interop approaches:

```
rust-cpp-interop-example/
├── Makefile                   # Primary build system
├── examples/                  # C++ usage examples
│   ├── manual_ffi_example.cpp # Manual FFI demonstration
│   └── cxx_example.cpp        # Cxx-based demonstration
└── crates/
    ├── robot_joint/           # Pure Rust library
    ├── robot_joint-cpp/       # Manual FFI C++ bindings (Parts 1 & 2)
    └── robot_joint-cxx/       # Cxx-based C++ bindings (Part 3)
```

## Features Demonstrated

### Core Concepts
- **Opaque Type Handles**: Safe handles to Rust objects from C++
- **Library Type Conversion**: nalgebra ↔ Eigen type conversions
- **Memory Management**: Proper allocation/deallocation across language boundaries
- **CMake Integration**: Making Rust libraries consumable by C++ projects

### robot_joint (Pure Rust)
- Basic robotics joint representation with nalgebra
- Transform calculations and limit checking
- Rust API with error handling using thiserror

### robot_joint-cpp (Manual FFI)
- Hand-written FFI with `#[no_mangle]` and `extern "C"`
- C++ wrapper classes with RAII
- Direct library type conversions between nalgebra and Eigen
- Static library output for C++ consumption

### robot_joint-cxx (Cxx-based)
- Generated interop using the cxx crate
- Type-safe bridging with minimal unsafe code
- Conversion utilities for library types
- Automatic code generation for safer bindings

## Prerequisites

### System Dependencies
```bash
# Ubuntu/Debian
sudo apt install build-essential cmake libeigen3-dev pkg-config

# Arch Linux
sudo pacman -S base-devel cmake eigen

# macOS
brew install cmake eigen pkg-config
```

### Rust
```bash
rustup toolchain install stable
```

## Building and Testing

This project uses a Makefile as the primary build system that coordinates Rust and C++ builds.

### Quick Start
```bash
# Build everything and run all tests
make

# Check dependencies first
make check-deps

# Clean all build artifacts
make clean
```

### Individual Components

#### Rust Libraries
```bash
# Build and test Rust crates only
make rust-build

# Or use cargo directly
cargo build --release
cargo test --release
```

#### Manual FFI C++ Library
```bash
# Build manual FFI version with tests
make manual-ffi-build

# Build with sanitizers
make sanitizer-tests
```

#### Cxx-based C++ Library
```bash
# Build cxx version with tests
make cxx-build
```

### Running Examples

The `examples/` directory contains C++ programs demonstrating both approaches:

```bash
# Build and run both examples
make examples

# Run individual examples
make manual_ffi_example
make cxx_example
```

#### Manual FFI Example Output
```
=== Manual FFI Example ===
Joint name: example_joint
Joint index: 0
Transform at 0 degrees:
1 0 0 0
0 1 0 0
0 0 1 0
0 0 0 1

Transform at 90 degrees:
6.12323e-17          -1           0           0
          1  6.12323e-17           0           0
          0           0           1           0
          0           0           0           1

Joint limits: [-3.14159, 3.14159]
Position 0.0 within limits: 1
Position 4.0 within limits: 0
```

#### Cxx Example Output
Similar output but using the cxx-generated bindings.

## Usage in Your C++ Projects

### Using FetchContent with Manual FFI

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

### Using FetchContent with Cxx

```cmake
include(FetchContent)
FetchContent_Declare(
  robot_joint_cxx
  GIT_REPOSITORY https://github.com/tylerjw/tylerjw.dev
  GIT_TAG main
  SOURCE_SUBDIR "rust-cpp-interop-example/crates/robot_joint-cxx"
)
FetchContent_MakeAvailable(robot_joint_cxx)

target_link_libraries(your_target PRIVATE robot_joint::robot_joint)
```

### C++ Code Example (Manual FFI)
```cpp
#include <robot_joint.hpp>
#include <iostream>
#include <Eigen/Geometry>

int main() {
    // Create a robot joint
    robot_joint::Joint joint("shoulder_joint");

    // Calculate transform
    Eigen::VectorXd variables(1);
    variables << 1.57; // 90 degrees in radians

    auto transform = joint.calculate_transform(variables);
    std::cout << "Transform:\n" << transform.matrix() << std::endl;

    return 0;
}
```

### C++ Code Example (Cxx-based)
```cpp
#include <robot_joint/robot_joint.hpp>
#include <iostream>
#include <Eigen/Geometry>

int main() {
    // Create a joint using cxx interface
    auto joint = robot_joint::new_joint("shoulder_joint");

    // Calculate transform
    rust::Vec<double> variables;
    variables.push_back(1.57); // 90 degrees in radians

    auto transform_vec = joint->calculate_transform(robot_joint::to_rust_slice(variables));
    auto transform = robot_joint::to_eigen_isometry3d(std::move(transform_vec));
    std::cout << "Transform:\n" << transform.matrix() << std::endl;

    return 0;
}
```

## Testing with Sanitizers

The Makefile includes sanitizer testing to catch memory safety issues:

```bash
# Run AddressSanitizer and UndefinedBehaviorSanitizer tests
make sanitizer-tests
```

This runs:
- Manual FFI tests with AddressSanitizer
- Manual FFI tests with UndefinedBehaviorSanitizer
- Cxx tests with AddressSanitizer

## Key Patterns Demonstrated

### 1. Opaque Type Handles
Rust objects are represented as opaque pointers in C++, with C++ wrapper classes managing their lifetime using RAII principles.

### 2. Library Type Conversions
Efficient conversions between Rust nalgebra types (`Isometry3<f64>`) and C++ Eigen types (`Eigen::Isometry3d`) while maintaining performance.

### 3. Error Handling
Proper error propagation across the FFI boundary, with Rust errors being converted to appropriate C++ exceptions or error codes.

### 4. Memory Safety
Ensuring allocators and deallocators are paired correctly across language boundaries to prevent memory leaks and use-after-free bugs.

## Performance Considerations

- **Zero-copy when possible**: Using slices and references to avoid unnecessary memory copies
- **Efficient conversions**: Direct memory layout compatibility between nalgebra and Eigen matrices
- **Static linking**: Avoiding runtime dependencies by producing static libraries

## Makefile Targets

Available make targets:

| Target | Description |
|--------|-------------|
| `make` or `make all` | Build everything and run all tests |
| `make check-deps` | Verify system dependencies |
| `make clean` | Clean all build artifacts |
| `make rust-build` | Build and test Rust libraries |
| `make manual-ffi-build` | Build manual FFI C++ library |
| `make cxx-build` | Build cxx-based C++ library |
| `make sanitizer-tests` | Run tests with sanitizers |
| `make examples` | Build both C++ examples |
| `make manual_ffi_example` | Build and run manual FFI example |
| `make cxx_example` | Build and run cxx example |
| `make docs` | Generate Rust documentation |
| `make help` | Show all available targets |

## Project Comparison

### Manual FFI vs Cxx-based Approach

| Aspect | Manual FFI | Cxx-based |
|--------|------------|-----------|
| **Setup Complexity** | High (hand-written bindings) | Medium (generated bindings) |
| **Performance** | Optimal (direct calls) | Good (small overhead) |
| **Memory Safety** | Manual (requires careful coding) | Automatic (cxx provides safety) |
| **Maintenance** | High (manual updates needed) | Low (automatic generation) |
| **Learning Curve** | Steep (requires FFI expertise) | Moderate (cxx abstracts complexity) |
| **Build Dependencies** | Minimal (just Rust + C++) | Additional (cxx crate) |

### When to Use Which

**Choose Manual FFI when:**
- Performance is absolutely critical
- You need maximum control over the FFI boundary
- You have experienced unsafe Rust developers
- You want minimal external dependencies

**Choose Cxx-based when:**
- Development speed and safety are priorities
- You have large APIs to expose
- You want automatic memory management
- Your team prefers maintainable, generated code

## Documentation

Generate Rust documentation:
```bash
make docs
# or
cargo doc --no-deps --release
```

## License

BSD-3-Clause

## References

- [nalgebra](https://nalgebra.org/) - Linear algebra library for Rust
- [Eigen](https://eigen.tuxfamily.org/) - C++ template library for linear algebra
- [cxx](https://cxx.rs/) - Safe interop between Rust and C++
- [Corrosion](https://corrosion-rs.github.io/corrosion/) - CMake integration for Rust
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) - The Dark Arts of Unsafe Rust
- [Tyler Weaver's Blog](https://tylerjw.dev) - Original blog post series
