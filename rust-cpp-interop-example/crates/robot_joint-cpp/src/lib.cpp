#include "robot_joint.hpp"
#include <stdexcept>
#include <cstring>

namespace robot_joint {

Joint::Joint(const std::string& name) noexcept
    : joint_{robot_joint_new(name.c_str())} {
    // Constructor handles the case where joint_ could be null
    // In a production system, you might want to throw an exception
    // or handle this error case more gracefully
}

std::string Joint::name() const {
    if (!joint_) {
        return "";
    }

    const char* name_ptr = robot_joint_get_name(joint_.get());
    if (!name_ptr) {
        return "";
    }

    // Make a copy of the string
    std::string result(name_ptr);

    // Free the string allocated by Rust
    robot_joint_free_string(const_cast<char*>(name_ptr));

    return result;
}

unsigned int Joint::index() const noexcept {
    if (!joint_) {
        return 0;
    }
    return robot_joint_get_index(joint_.get());
}

unsigned int Joint::parent_link_index() const noexcept {
    if (!joint_) {
        return 0;
    }
    return robot_joint_get_parent_link_index(joint_.get());
}

unsigned int Joint::child_link_index() const noexcept {
    if (!joint_) {
        return 0;
    }
    return robot_joint_get_child_link_index(joint_.get());
}

unsigned int Joint::dof_index() const noexcept {
    if (!joint_) {
        return 0;
    }
    return robot_joint_get_dof_index(joint_.get());
}

Eigen::Isometry3d Joint::calculate_transform(const Eigen::VectorXd& variables) const {
    if (!joint_) {
        return Eigen::Isometry3d::Identity();
    }

    const auto rust_transform = robot_joint_calculate_transform(
        joint_.get(),
        variables.data(),
        static_cast<unsigned int>(variables.size())
    );

    return mat4d_to_isometry3d(rust_transform);
}

Eigen::Isometry3d Joint::parent_link_to_joint_origin() const {
    if (!joint_) {
        return Eigen::Isometry3d::Identity();
    }

    const auto rust_transform = robot_joint_get_parent_link_to_joint_origin(joint_.get());
    return mat4d_to_isometry3d(rust_transform);
}

bool Joint::is_within_limits(double position) const noexcept {
    if (!joint_) {
        return false;
    }
    return robot_joint_is_within_limits(joint_.get(), position);
}

std::pair<double, double> Joint::limits() const {
    if (!joint_) {
        return {0.0, 0.0};
    }

    double min_limit, max_limit;
    robot_joint_get_limits(joint_.get(), &min_limit, &max_limit);
    return {min_limit, max_limit};
}

Eigen::Isometry3d Joint::mat4d_to_isometry3d(const rust::Mat4d& mat) {
    // The Rust Mat4d is in column-major order, same as Eigen
    Eigen::Isometry3d transform;
    transform.matrix() = Eigen::Map<const Eigen::Matrix4d>(mat.data);
    return transform;
}

} // namespace robot_joint
