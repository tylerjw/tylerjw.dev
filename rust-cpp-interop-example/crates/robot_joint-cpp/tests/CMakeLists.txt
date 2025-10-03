include(FetchContent)
FetchContent_Declare(
  Catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2.git
  GIT_TAG v3.5.2)
FetchContent_MakeAvailable(Catch2)
include(Catch)

add_executable(robot_joint_tests tests.cpp)
target_link_libraries(robot_joint_tests PRIVATE
  Catch2::Catch2WithMain
  robot_joint::robot_joint)

# Set C++ standard to match the library
set_property(TARGET robot_joint_tests PROPERTY CXX_STANDARD 20)
set_property(TARGET robot_joint_tests PROPERTY CXX_STANDARD_REQUIRED ON)

# Register tests with CTest
catch_discover_tests(robot_joint_tests)
