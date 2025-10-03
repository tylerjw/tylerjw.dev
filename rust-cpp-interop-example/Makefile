# Makefile for Rust/C++ Interop Example

# Default shell and options
SHELL := /bin/bash
.SHELLFLAGS := -euo pipefail -c

# Project directories
PROJECT_ROOT := $(shell pwd)
ROBOT_JOINT_CPP_DIR := crates/robot_joint-cpp
ROBOT_JOINT_CXX_DIR := crates/robot_joint-cxx
EXAMPLES_DIR := examples

# Build directories
BUILD_CPP_DIR := $(ROBOT_JOINT_CPP_DIR)/build
BUILD_CXX_DIR := $(ROBOT_JOINT_CXX_DIR)/build
BUILD_CPP_ASAN_DIR := $(ROBOT_JOINT_CPP_DIR)/build-asan
BUILD_CPP_UBSAN_DIR := $(ROBOT_JOINT_CPP_DIR)/build-ubsan
BUILD_CXX_ASAN_DIR := $(ROBOT_JOINT_CXX_DIR)/build-asan

# Parallel build jobs
NPROC := $(shell nproc 2>/dev/null || echo 4)

# Default target: build and run all tests with sanitizers
.PHONY: default all
default all: check-deps test examples

# Check dependencies
.PHONY: check-deps
check-deps:
	@command -v cargo >/dev/null 2>&1 || (@echo "Cargo not found. Please install Rust." && exit 1)
	@command -v cmake >/dev/null 2>&1 || (@echo "CMake not found. Please install CMake." && exit 1)
	@pkg-config --exists eigen3 2>/dev/null || (@echo "Eigen3 not found via pkg-config. Make sure Eigen3 is installed." && exit 1)

# Clean all builds
.PHONY: clean
clean:
	cargo clean
	rm -rf $(BUILD_CPP_DIR) $(BUILD_CXX_DIR)
	rm -rf $(BUILD_CPP_ASAN_DIR) $(BUILD_CPP_UBSAN_DIR) $(BUILD_CXX_ASAN_DIR)
	rm -rf $(EXAMPLES_DIR)/*.o $(EXAMPLES_DIR)/manual_ffi_example $(EXAMPLES_DIR)/cxx_example
	rm -f .rust-build-stamp .manual-ffi-build-stamp .cxx-build-stamp

# Build Rust libraries and run tests
rust-build: .rust-build-stamp

.rust-build-stamp: $(wildcard crates/*/src/*.rs) $(wildcard crates/*/Cargo.toml)
	cargo build --release
	cargo test --release
	@touch .rust-build-stamp

# Build manual FFI version
manual-ffi-build: .manual-ffi-build-stamp

.manual-ffi-build-stamp: .rust-build-stamp $(wildcard $(ROBOT_JOINT_CPP_DIR)/src/*) $(wildcard $(ROBOT_JOINT_CPP_DIR)/include/*) $(ROBOT_JOINT_CPP_DIR)/CMakeLists.txt
	cd $(ROBOT_JOINT_CPP_DIR) && \
	cmake -B build -S . -DCMAKE_BUILD_TYPE=Release -DCMAKE_EXPORT_COMPILE_COMMANDS=ON && \
	cmake --build build --parallel $(NPROC)
	cd $(ROBOT_JOINT_CPP_DIR) && ctest --test-dir build --output-on-failure --parallel $(NPROC)
	@touch .manual-ffi-build-stamp

# Build cxx version
cxx-build: .cxx-build-stamp

.cxx-build-stamp: .rust-build-stamp $(wildcard $(ROBOT_JOINT_CXX_DIR)/src/*) $(wildcard $(ROBOT_JOINT_CXX_DIR)/include/*) $(ROBOT_JOINT_CXX_DIR)/CMakeLists.txt
	cd $(ROBOT_JOINT_CXX_DIR) && \
	cmake -B build -S . -DCMAKE_BUILD_TYPE=Release -DCMAKE_EXPORT_COMPILE_COMMANDS=ON && \
	cmake --build build --parallel $(NPROC)
	cd $(ROBOT_JOINT_CXX_DIR) && ctest --test-dir build --output-on-failure --parallel $(NPROC)
	@touch .cxx-build-stamp

# Run sanitizer tests
.PHONY: sanitizer-tests
sanitizer-tests: manual-ffi-build cxx-build
	# Manual FFI with AddressSanitizer
	cd $(ROBOT_JOINT_CPP_DIR) && \
	cmake -B build-asan -S . -DCMAKE_BUILD_TYPE=Debug -DCMAKE_CXX_FLAGS="-fsanitize=address -fno-omit-frame-pointer" && \
	cmake --build build-asan --parallel $(NPROC) && \
	ctest --test-dir build-asan --output-on-failure

	# Manual FFI with UndefinedBehaviorSanitizer
	cd $(ROBOT_JOINT_CPP_DIR) && \
	cmake -B build-ubsan -S . -DCMAKE_BUILD_TYPE=Debug -DCMAKE_CXX_FLAGS="-fsanitize=undefined -fno-omit-frame-pointer" && \
	cmake --build build-ubsan --parallel $(NPROC) && \
	ctest --test-dir build-ubsan --output-on-failure

	# Cxx version with AddressSanitizer
	cd $(ROBOT_JOINT_CXX_DIR) && \
	cmake -B build-asan -S . -DCMAKE_BUILD_TYPE=Debug -DCMAKE_CXX_FLAGS="-fsanitize=address -fno-omit-frame-pointer" && \
	cmake --build build-asan --parallel $(NPROC) && \
	ctest --test-dir build-asan --output-on-failure

# Build and run examples
.PHONY: examples
examples: cxx_example manual_ffi_example

$(EXAMPLES_DIR)/manual_ffi_example: $(EXAMPLES_DIR)/manual_ffi_example.cpp .manual-ffi-build-stamp
	cd $(ROBOT_JOINT_CPP_DIR) && \
	g++ -std=c++20 -I./include $$(pkg-config --cflags eigen3) \
		../../$(EXAMPLES_DIR)/manual_ffi_example.cpp -o ../../$(EXAMPLES_DIR)/manual_ffi_example \
		-L./build -lrobot_joint -lrobot_jointcpp -lpthread -ldl -lm

# Individual example targets
$(EXAMPLES_DIR)/cxx_example: $(EXAMPLES_DIR)/cxx_example.cpp .cxx-build-stamp
	cd $(ROBOT_JOINT_CXX_DIR) && \
	g++ -std=c++20 -I./include -I./build/cargo/build/x86_64-unknown-linux-gnu/cxxbridge \
		$$(pkg-config --cflags eigen3) \
		../../$(EXAMPLES_DIR)/cxx_example.cpp -o ../../$(EXAMPLES_DIR)/cxx_example \
		./build/librobot_joint_cxx.a \
		-pthread -ldl

.PHONY: cxx_example
cxx_example: $(EXAMPLES_DIR)/cxx_example
	./$(EXAMPLES_DIR)/cxx_example

.PHONY: manual_ffi_example
manual_ffi_example: $(EXAMPLES_DIR)/manual_ffi_example
	./$(EXAMPLES_DIR)/manual_ffi_example

# Documentation generation
.PHONY: docs
docs:
	cargo doc --no-deps --release

.PHONY: test
test: check-deps rust-build manual-ffi-build cxx-build docs

# Help target
.PHONY: help
help:
	@echo "Rust/C++ Interop Example Makefile"
	@echo "================================="
	@echo
	@echo "Targets:"
	@echo "  default, all             Build and test everything with sanitizers"
	@echo "  check-deps               Check system dependencies"
	@echo "  clean                    Clean all build artifacts"
	@echo "  rust-build               Build Rust libraries and run tests"
	@echo "  manual-ffi-build         Build manual FFI C++ library and run tests"
	@echo "  cxx-build                Build cxx-based C++ library and run tests"
	@echo "  sanitizer-tests          Run tests with AddressSanitizer and UBSan"
	@echo "  examples                 Build both C++ examples"
	@echo "  build-examples           Build the manual FFI example binary"
	@echo "  manual_ffi_example       Build and run the manual FFI example"
	@echo "  cxx_example              Build and run the cxx example"
	@echo "  docs                     Generate Rust documentation"
	@echo "  test                     Run comprehensive test suite"
	@echo "  help                     Show this help message"
	@echo
	@echo "Environment variables:"
	@echo "  NPROC=N                  Override parallel job count"
