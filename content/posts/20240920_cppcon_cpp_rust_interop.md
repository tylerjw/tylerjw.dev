+++
title = "CppCon - C++/Rust Interop: Using Bridges in Practice"
date = 2024-09-20
type = "post"
in_search_index = true
[taxonomies]
tags = ["CppCon", "Talks", "Rust"]
+++

<iframe width="720" height="400" src="https://www.youtube.com/embed/RccCeMsXW0Q?si=Ir6PeZSwudYT942I" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>


I spent a year trying to create a Rust project inside a company that primarily wrote C++.
This conflict drove me to explore Interop to allow me to develop new Rust code without boiling the ocean.
After many failed attempts to get it working, I found a way that worked well for my applications.
The blog posts I wrote and this talk are the product of that struggle.

- [PDF of Slides](/pdf/20240920_C++_Rust_Interop__Using_Bridges_in_Practice.pdf)
- [Part 1 - Just the Basics](/posts/rust-cpp-interop)
- [Part 2 - CMake](/posts/rust-cmake-interop-cmake)
- [Part 3 - Cxx](/posts/rust-cmake-interop-part-3-cxx)
- [Part 4 - Binding to a C++/CMake/Conan Project](/posts/rust-cpp-part4-buildrs)
