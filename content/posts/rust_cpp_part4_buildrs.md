+++
title = "Rust/C++ Interop Part 4 - Binding to a C++/CMake/Conan Project"
date = 2024-03-11
type = "post"
in_search_index = true
[taxonomies]
tags = ["Rust", "C++", "CMake", "Cxx", "Conan", "build.rs"]
+++

Today's interop story stands alone.
Previously, I showed you how to create Rust libraries that can be used from C++.
In this post, I'll show you how to use a C++ library in Rust.

[Data Tamer](https://github.com/PickNikRobotics/data_tamer) is Davide's C++ library for logging numerical values into periodic snapshots and then sending those snapshots out to a sink.
This library uses CMake as the build tool and the [Conan 2](https://docs.conan.io/2/) package manager to fetch and build dependencies.

## Approach

Before we can create bindings, we need to figure out how to fetch and build Data Tamer and its dependencies.
I would like my Rust project built using Cargo, so I'll use the `build.rs` script to set up the dependencies.
Lastly, I will use CXX to make a Rust interface to the C++ code.

Before attempting this, I've installed Conan 2 and Bindgen's dependencies:
```bash
python3 -m pip install conan --upgrade
conan profile detect
sudo apt install llvm-dev libclang-dev clang
```

## Build

In my `Cargo.toml`, I have these dependencies:
```toml
[dependencies]
cxx = "1.0"

[build-dependencies]
cxx-build = "1.0"
anyhow = "1.0.79"
git2 = "0.18.2"
conan2 = "0.1"
cmake = "0.1"
```

At the root of my project, I create a `build.rs` file, which Cargo will run as part of my build.
In this file, we are going to do these steps:

1. Get the build directory
2. Clone Data Tamer into the build directory
3. Invoke Conan to fetch and build dependencies
4. Build Data Tamer using CMake
5. Code-Gen bindings using Cxx
6. Setup linking from Rust to C++ dependencies

```rust
use conan2::ConanInstall;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");

    let out_dir: PathBuf = std::env::var_os("OUT_DIR")
        .expect("OUT_DIR environment variable must be set")
        .into();

    // Clone data-tamer
    let data_tamer_url = "https://github.com/PickNikRobotics/data_tamer";
    let data_tamer_source = out_dir.join(Path::new("data_tamer"));
    if !data_tamer_source.exists() {
        git2::Repository::clone(data_tamer_url, data_tamer_source.as_path())?;
    }

    let data_tamer_cpp = out_dir.join(Path::new("data_tamer/data_tamer_cpp"));

    // Build conan dependencies of data_tamer
    let conan_instructions = ConanInstall::with_recipe(&data_tamer_cpp)
        .build("missing")
        .run()
        .parse();
    let conan_includes = conan_instructions.include_paths();
    let toolchain_file = out_dir.join(Path::new("build/Debug/generators/conan_toolchain.cmake"));

    // Build the data_tamer library
    let data_tamer_install = cmake::Config::new(&data_tamer_cpp)
        .define("CMAKE_TOOLCHAIN_FILE", toolchain_file)
        .build();
    let data_tamer_lib_path = data_tamer_install.join(Path::new("lib"));
    let data_tamer_include_path = data_tamer_install.join(Path::new("include"));

    // Code-gen bindings and build the rust project
    cxx_build::bridge("src/main.rs")
        .includes(conan_includes)
        .include(data_tamer_include_path)
        .include("src")
        .std("c++17")
        .compile("demo");

    // Statically link with data_tamer
    println!(
        "cargo:rustc-link-search=native={}",
        data_tamer_lib_path.display()
    );
    println!("cargo:rustc-link-lib=static=data_tamer");

    // Emit cargo instructions for linking to transitive dependencies
    conan_instructions.emit();
    Ok(())
}
```

If you run into linking errors for undefined symbols, one cause can be the order of the linker arguments.
To fix this, emit linker instructions to Cargo after the call to compile the cxx bridge.

## Binding to the ChannelRegistry Constructor

The first thing I want to bind to in Data Tamer is the `ChannelsRegistry` type.
This type can either be used as a singleton or through its constructor.
Neither of those methods is natively supported for bindings in Cxx.
We will use the constructor to create a `unique_ptr` to the registry.

To do this, I created a `shim.hpp` to make functions we can use in our bindings in the `src` directory.
You may have noticed that I included the `src` directory as an include in the `build.rs`.
```C++
#pragma once

#include <memory>

namespace DataTamer
{
    template <typename T, typename... Args>
    std::unique_ptr<T> construct_unique(Args... args)
    {
        return std::make_unique<T>(args...);
    }
}
```

Then, on the Rust side, we can generate a binding using this template function:
```rust
#[cxx::bridge(namespace = "DataTamer")]
mod data_tamer {
    unsafe extern "C++" {
        include!("shim.hpp");
        include!("data_tamer/data_tamer.hpp");

        type ChannelsRegistry;

        #[rust_name = "channels_registry_new"]
        fn construct_unique() -> UniquePtr<ChannelsRegistry>;
    }
}

fn main() {
    let mut registry = data_tamer::channels_registry_new();
}
```

# Conclusion
The Rust ecosystem provides the tools we need to Build C++ projects as part of our Cargo build.

## References
- [C++ Interop Part 1 - Just the Basics](/posts/rust-cpp-interop)
- [C++ Interop Part 2 - CMake](/posts/rust-cmake-interop-cmake/)
- [C++ Interop Part 3 - Cxx](/posts/rust-cmake-interop-part-3-cxx/)
- [Cxx](https://cxx.rs/)
- [Data Tamer](https://github.com/PickNikRobotics/data_tamer)
