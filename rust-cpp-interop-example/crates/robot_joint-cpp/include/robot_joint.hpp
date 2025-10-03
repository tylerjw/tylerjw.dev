#pragma once

#include <memory>
#include <string>
#include <Eigen/Geometry>

namespace robot_joint::rust {

/// Forward-declaration of opaque type to use as pointer to the Rust object.
struct RobotJointHandle;

/// C-compatible matrix representation
struct Mat4d {
    double data[16];
};

} // namespace robot_joint::rust

extern "C" {
    // Joint lifecycle
    extern robot_joint::rust::RobotJointHandle* robot_joint_new(const char* name);
    extern void robot_joint_free(robot_joint::rust::RobotJointHandle* joint);

    // String management
    extern const char* robot_joint_get_name(const robot_joint::rust::RobotJointHandle* joint);
    extern void robot_joint_free_string(char* s);

    // Property getters
    extern unsigned int robot_joint_get_index(const robot_joint::rust::RobotJointHandle* joint);
    extern unsigned int robot_joint_get_parent_link_index(const robot_joint::rust::RobotJointHandle* joint);
    extern unsigned int robot_joint_get_child_link_index(const robot_joint::rust::RobotJointHandle* joint);
    extern unsigned int robot_joint_get_dof_index(const robot_joint::rust::RobotJointHandle* joint);

    // Transform calculations
    extern robot_joint::rust::Mat4d robot_joint_calculate_transform(
        const robot_joint::rust::RobotJointHandle* joint,
        const double* variables,
        unsigned int size);

    extern robot_joint::rust::Mat4d robot_joint_get_parent_link_to_joint_origin(
        const robot_joint::rust::RobotJointHandle* joint);

    // Joint limits
    extern bool robot_joint_is_within_limits(
        const robot_joint::rust::RobotJointHandle* joint,
        double position);

    extern void robot_joint_get_limits(
        const robot_joint::rust::RobotJointHandle* joint,
        double* min_limit,
        double* max_limit);
}

/// Create a custom deleter from a function template argument.
template<auto fn>
struct deleter_from_fn {
    template<typename T>
    constexpr void operator()(T* arg) const {
        fn(arg);
    }
};

namespace robot_joint {

/// Move-only handle to robot_joint object (living on the Rust side).
class Joint {
public:
    /// Create a new joint with the given name
    explicit Joint(const std::string& name) noexcept;

    /// Default destructor
    ~Joint() noexcept = default;

    /// Move constructor and assignment
    Joint(Joint&& other) noexcept = default;
    Joint& operator=(Joint&& other) noexcept = default;

    /// Delete copy constructor and assignment (move-only)
    Joint(const Joint&) = delete;
    Joint& operator=(const Joint&) = delete;

    /// Get the joint name
    std::string name() const;

    /// Get joint properties
    unsigned int index() const noexcept;
    unsigned int parent_link_index() const noexcept;
    unsigned int child_link_index() const noexcept;
    unsigned int dof_index() const noexcept;

    /// Calculate transform for given joint variables
    Eigen::Isometry3d calculate_transform(const Eigen::VectorXd& variables) const;

    /// Get parent link to joint origin transform
    Eigen::Isometry3d parent_link_to_joint_origin() const;

    /// Check joint limits
    bool is_within_limits(double position) const noexcept;
    std::pair<double, double> limits() const;

private:
    /// Convert Rust Mat4d to Eigen::Isometry3d
    static Eigen::Isometry3d mat4d_to_isometry3d(const rust::Mat4d& mat);

    /// Managed pointer to the Rust joint object
    std::unique_ptr<rust::RobotJointHandle, deleter_from_fn<robot_joint_free>> joint_;
};

} // namespace robot_joint
