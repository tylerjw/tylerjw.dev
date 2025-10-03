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

# Import the Rust FFI library
corrosion_import_crate(MANIFEST_PATH Cargo.toml CRATES robot_joint-cpp)

# Create the C++ library
add_library(robot_joint STATIC src/lib.cpp)

# Set up include directories
target_include_directories(
  robot_joint PUBLIC $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
                     $<INSTALL_INTERFACE:include>)

# Link with dependencies
target_link_libraries(robot_joint PUBLIC Eigen3::Eigen)
target_link_libraries(robot_joint PRIVATE robot_jointcpp)

# Set C++ standard and compiler properties
set_property(TARGET robot_joint PROPERTY CXX_STANDARD 20)
set_property(TARGET robot_joint PROPERTY CXX_STANDARD_REQUIRED ON)
set_property(TARGET robot_joint PROPERTY POSITION_INDEPENDENT_CODE ON)

# Create an alias for namespaced usage
add_library(robot_joint::robot_joint ALIAS robot_joint)

# Installation configuration
include(CMakePackageConfigHelpers)
include(GNUInstallDirs)

# Install the targets
install(
  TARGETS robot_joint robot_jointcpp
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
install(FILES include/robot_joint.hpp
        DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

# Testing configuration - only build tests when this is the root project
if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME)
  include(CTest)
  if(BUILD_TESTING)
    add_subdirectory(tests)
  endif()
endif()
