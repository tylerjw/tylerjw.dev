cmake_minimum_required(VERSION 3.16)
project(robot_joint VERSION 0.1.0)

find_package(Eigen3 REQUIRED)

# Fetch and configure Corrosion for Rust integration
include(FetchContent)
FetchContent_Declare(
  Corrosion
  GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
  GIT_TAG v0.5)
FetchContent_MakeAvailable(Corrosion)

# Import the Rust crate - this builds the Rust code and generates cxx headers
corrosion_import_crate(MANIFEST_PATH Cargo.toml CRATES robot_joint-cxx)

# Get the Rust target triple dynamically
execute_process(
    COMMAND rustc --version --verbose
    OUTPUT_VARIABLE RUSTC_VERSION_OUTPUT
    OUTPUT_STRIP_TRAILING_WHITESPACE
)

string(REGEX MATCH "host: ([^\n]+)" _ ${RUSTC_VERSION_OUTPUT})
set(_RUST_TARGET ${CMAKE_MATCH_1})

# Fallback to common targets if detection fails
if(NOT _RUST_TARGET)
    if(WIN32)
        if(CMAKE_SIZEOF_VOID_P EQUAL 8)
            set(_RUST_TARGET "x86_64-pc-windows-msvc")
        else()
            set(_RUST_TARGET "i686-pc-windows-msvc")
        endif()
    elseif(APPLE)
        if(CMAKE_SYSTEM_PROCESSOR MATCHES "arm64|aarch64")
            set(_RUST_TARGET "aarch64-apple-darwin")
        else()
            set(_RUST_TARGET "x86_64-apple-darwin")
        endif()
    else() # Linux and other Unix-like systems
        if(CMAKE_SYSTEM_PROCESSOR MATCHES "aarch64|arm64")
            set(_RUST_TARGET "aarch64-unknown-linux-gnu")
        elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "arm")
            set(_RUST_TARGET "armv7-unknown-linux-gnueabihf")
        else()
            set(_RUST_TARGET "x86_64-unknown-linux-gnu")
        endif()
    endif()
endif()

message(STATUS "Using Rust target: ${_RUST_TARGET}")

# Create a C++ wrapper library that includes our utility headers
add_library(robot_joint_wrapper INTERFACE)

# Add include directories - both our headers and the cxx-generated ones
target_include_directories(robot_joint_wrapper INTERFACE
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<BUILD_INTERFACE:${CMAKE_CURRENT_BINARY_DIR}>
  $<BUILD_INTERFACE:${CMAKE_CURRENT_BINARY_DIR}/cargo/build/${_RUST_TARGET}/cxxbridge>
  $<INSTALL_INTERFACE:include>)

# Link the Rust library and Eigen
target_link_libraries(robot_joint_wrapper INTERFACE
  robot_joint_cxx
  Eigen3::Eigen)

# Set C++ standard
target_compile_features(robot_joint_wrapper INTERFACE cxx_std_20)

# Create the main target as an alias
add_library(robot_joint ALIAS robot_joint_wrapper)
add_library(robot_joint::robot_joint ALIAS robot_joint_wrapper)

# Installation configuration
include(CMakePackageConfigHelpers)
include(GNUInstallDirs)

# Install the targets
install(
  TARGETS robot_joint_wrapper robot_joint_cxx
  EXPORT ${PROJECT_NAME}Targets
  RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
  LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
  ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR})

# Install the export set
install(
  EXPORT ${PROJECT_NAME}Targets
  NAMESPACE robot_joint::
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")

# Generate and install the config file
configure_package_config_file(
  cmake/robot_jointConfig.cmake.in
  "${PROJECT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
  INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")

install(FILES "${PROJECT_BINARY_DIR}/${PROJECT_NAME}Config.cmake"
        DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")

# Install headers
install(DIRECTORY include/ DESTINATION include)

# Testing configuration - only build tests when this is the root project
if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME)
  include(CTest)
  if(BUILD_TESTING)
    add_subdirectory(tests)
  endif()
endif()
