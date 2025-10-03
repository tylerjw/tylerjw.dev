#include <robot_joint/robot_joint.hpp>
#include <iostream>
#include <Eigen/Geometry>

int main() {
    std::cout << "=== Cxx-based Example ===" << std::endl;

    // Create a joint using cxx interface
    auto joint = robot_joint::new_joint("cxx_example_joint");

    std::cout << "Joint name: " << joint->name() << std::endl;
    std::cout << "Joint index: " << joint->index() << std::endl;

    // Calculate transform at zero position
    rust::Vec<double> variables;
    variables.push_back(0.0);

    auto transform_vec = joint->calculate_transform(robot_joint::to_rust_slice(variables));
    auto transform = robot_joint::to_eigen_isometry3d(std::move(transform_vec));
    std::cout << "Transform at 0 degrees:" << std::endl;
    std::cout << transform.matrix() << std::endl;

    // Calculate transform at 90 degrees
    variables.clear();
    variables.push_back(M_PI / 2.0);
    transform_vec = joint->calculate_transform(robot_joint::to_rust_slice(variables));
    transform = robot_joint::to_eigen_isometry3d(std::move(transform_vec));
    std::cout << "\\nTransform at 90 degrees:" << std::endl;
    std::cout << transform.matrix() << std::endl;

    // Check limits
    auto limits_vec = joint->get_limits();
    auto limits = robot_joint::to_limits_pair(limits_vec);
    std::cout << "\\nJoint limits: [" << limits.first << ", " << limits.second << "]" << std::endl;
    std::cout << "Position 0.0 within limits: " << joint->is_within_limits(0.0) << std::endl;
    std::cout << "Position 4.0 within limits: " << joint->is_within_limits(4.0) << std::endl;

    return 0;
}
