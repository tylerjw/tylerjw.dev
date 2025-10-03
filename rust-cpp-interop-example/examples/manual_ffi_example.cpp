#include <robot_joint.hpp>
#include <iostream>
#include <Eigen/Geometry>

int main() {
    std::cout << "=== Manual FFI Example ===" << std::endl;

    // Create a joint
    robot_joint::Joint joint("example_joint");

    std::cout << "Joint name: " << joint.name() << std::endl;
    std::cout << "Joint index: " << joint.index() << std::endl;

    // Calculate transform at zero position
    Eigen::VectorXd variables(1);
    variables << 0.0;

    auto transform = joint.calculate_transform(variables);
    std::cout << "Transform at 0 degrees:" << std::endl;
    std::cout << transform.matrix() << std::endl;

    // Calculate transform at 90 degrees
    variables << M_PI / 2.0;
    transform = joint.calculate_transform(variables);
    std::cout << "\\nTransform at 90 degrees:" << std::endl;
    std::cout << transform.matrix() << std::endl;

    // Check limits
    auto [min_limit, max_limit] = joint.limits();
    std::cout << "\\nJoint limits: [" << min_limit << ", " << max_limit << "]" << std::endl;
    std::cout << "Position 0.0 within limits: " << joint.is_within_limits(0.0) << std::endl;
    std::cout << "Position 4.0 within limits: " << joint.is_within_limits(4.0) << std::endl;

    return 0;
}
