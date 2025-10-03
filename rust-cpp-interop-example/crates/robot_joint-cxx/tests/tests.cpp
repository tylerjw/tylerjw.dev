#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>
#include <robot_joint/robot_joint.hpp>

#include <Eigen/Geometry>
#include <cmath>

using namespace robot_joint;
using namespace Catch::Matchers;

TEST_CASE("Cxx Joint creation and basic properties", "[joint][cxx][creation]") {
    SECTION("Create joint with name") {
        auto joint = new_joint("test_joint_cxx");

        REQUIRE(joint->name() == "test_joint_cxx");
        REQUIRE(joint->index() == 0);
        REQUIRE(joint->parent_link_index() == 0);
        REQUIRE(joint->child_link_index() == 1);
        REQUIRE(joint->dof_index() == 0);
    }

    SECTION("Create joint with empty name") {
        auto joint = new_joint("");
        REQUIRE(joint->name() == "");
    }

    SECTION("Create joint with special characters in name") {
        auto joint = new_joint("joint_with-special.chars");
        REQUIRE(joint->name() == "joint_with-special.chars");
    }

    SECTION("Multiple joints are independent") {
        auto joint1 = new_joint("joint_1");
        auto joint2 = new_joint("joint_2");

        REQUIRE(joint1->name() == "joint_1");
        REQUIRE(joint2->name() == "joint_2");
        REQUIRE(joint1->index() == joint2->index()); // Same default properties
    }
}

TEST_CASE("Cxx Transform calculations", "[joint][cxx][transform]") {
    auto joint = new_joint("transform_joint_cxx");

    SECTION("Identity transform at zero position") {
        rust::Vec<double> variables;
        variables.push_back(0.0);

        auto transform_vec = joint->calculate_transform(to_rust_slice(variables));
        auto transform = to_eigen_isometry3d(std::move(transform_vec));

        // Should be identity transform
        REQUIRE_THAT(transform.translation().x(), WithinAbs(0.0, 1e-10));
        REQUIRE_THAT(transform.translation().y(), WithinAbs(0.0, 1e-10));
        REQUIRE_THAT(transform.translation().z(), WithinAbs(0.0, 1e-10));

        // Check rotation matrix is identity
        Eigen::Matrix3d rotation = transform.rotation();
        Eigen::Matrix3d identity = Eigen::Matrix3d::Identity();

        for (int i = 0; i < 3; ++i) {
            for (int j = 0; j < 3; ++j) {
                REQUIRE_THAT(rotation(i, j), WithinAbs(identity(i, j), 1e-10));
            }
        }
    }

    SECTION("90 degree rotation") {
        rust::Vec<double> variables;
        variables.push_back(M_PI / 2.0);  // 90 degrees

        auto transform_vec = joint->calculate_transform(to_rust_slice(variables));
        auto transform = to_eigen_isometry3d(std::move(transform_vec));

        // Should have rotation but no translation
        REQUIRE_THAT(transform.translation().x(), WithinAbs(0.0, 1e-10));
        REQUIRE_THAT(transform.translation().y(), WithinAbs(0.0, 1e-10));
        REQUIRE_THAT(transform.translation().z(), WithinAbs(0.0, 1e-10));

        // Check that we have a 90-degree rotation around Z-axis
        Eigen::AngleAxisd angle_axis(transform.rotation());
        REQUIRE_THAT(angle_axis.angle(), WithinAbs(M_PI / 2.0, 1e-10));
    }

    SECTION("Full rotation") {
        rust::Vec<double> variables;
        variables.push_back(2.0 * M_PI);  // 360 degrees

        auto transform_vec = joint->calculate_transform(to_rust_slice(variables));
        auto transform = to_eigen_isometry3d(std::move(transform_vec));

        // Should be back to identity (within numerical precision)
        Eigen::Matrix3d rotation = transform.rotation();
        Eigen::Matrix3d identity = Eigen::Matrix3d::Identity();

        for (int i = 0; i < 3; ++i) {
            for (int j = 0; j < 3; ++j) {
                REQUIRE_THAT(rotation(i, j), WithinAbs(identity(i, j), 1e-10));
            }
        }
    }

    SECTION("Multiple variables (only first used)") {
        rust::Vec<double> variables;
        variables.push_back(M_PI / 4.0);
        variables.push_back(1.0);
        variables.push_back(2.0);

        auto transform_vec = joint->calculate_transform(to_rust_slice(variables));
        auto transform = to_eigen_isometry3d(std::move(transform_vec));

        // Should use only the first variable
        Eigen::AngleAxisd angle_axis(transform.rotation());
        REQUIRE_THAT(angle_axis.angle(), WithinAbs(M_PI / 4.0, 1e-10));
    }

    SECTION("Empty variables vector") {
        rust::Vec<double> variables; // Empty

        auto transform_vec = joint->calculate_transform(to_rust_slice(variables));
        auto transform = to_eigen_isometry3d(std::move(transform_vec));

        // Should return identity for empty variables
        REQUIRE_THAT(transform.translation().norm(), WithinAbs(0.0, 1e-10));

        Eigen::Matrix3d rotation = transform.rotation();
        Eigen::Matrix3d identity = Eigen::Matrix3d::Identity();

        for (int i = 0; i < 3; ++i) {
            for (int j = 0; j < 3; ++j) {
                REQUIRE_THAT(rotation(i, j), WithinAbs(identity(i, j), 1e-10));
            }
        }
    }
}

TEST_CASE("Cxx Parent link to joint origin transform", "[joint][cxx][parent_transform]") {
    auto joint = new_joint("origin_joint_cxx");

    SECTION("Default parent transform") {
        auto transform_vec = joint->parent_link_to_joint_origin();
        auto transform = to_eigen_isometry3d(std::move(transform_vec));

        // Should be identity by default
        REQUIRE_THAT(transform.translation().x(), WithinAbs(0.0, 1e-10));
        REQUIRE_THAT(transform.translation().y(), WithinAbs(0.0, 1e-10));
        REQUIRE_THAT(transform.translation().z(), WithinAbs(0.0, 1e-10));

        Eigen::Matrix3d rotation = transform.rotation();
        Eigen::Matrix3d identity = Eigen::Matrix3d::Identity();

        for (int i = 0; i < 3; ++i) {
            for (int j = 0; j < 3; ++j) {
                REQUIRE_THAT(rotation(i, j), WithinAbs(identity(i, j), 1e-10));
            }
        }
    }
}

TEST_CASE("Cxx Joint limits", "[joint][cxx][limits]") {
    auto joint = new_joint("limited_joint_cxx");

    SECTION("Default limits") {
        auto limits_vec = joint->get_limits();
        auto limits = to_limits_pair(limits_vec);

        REQUIRE_THAT(limits.first, WithinAbs(-M_PI, 1e-10));
        REQUIRE_THAT(limits.second, WithinAbs(M_PI, 1e-10));
    }

    SECTION("Within limits check") {
        REQUIRE(joint->is_within_limits(0.0));
        REQUIRE(joint->is_within_limits(M_PI - 0.1));
        REQUIRE(joint->is_within_limits(-M_PI + 0.1));
        REQUIRE(joint->is_within_limits(M_PI));
        REQUIRE(joint->is_within_limits(-M_PI));
    }

    SECTION("Outside limits check") {
        REQUIRE_FALSE(joint->is_within_limits(M_PI + 0.1));
        REQUIRE_FALSE(joint->is_within_limits(-M_PI - 0.1));
        REQUIRE_FALSE(joint->is_within_limits(2.0 * M_PI));
        REQUIRE_FALSE(joint->is_within_limits(-2.0 * M_PI));
    }
}

TEST_CASE("Cxx conversion utilities", "[joint][cxx][conversion]") {
    SECTION("Isometry3d to rust Vec and back") {
        Eigen::Isometry3d original = Eigen::Isometry3d::Identity();
        original.translation() = Eigen::Vector3d(1.0, 2.0, 3.0);
        original.rotate(Eigen::AngleAxisd(M_PI / 4.0, Eigen::Vector3d::UnitZ()));

        auto rust_vec = to_rust_vec(original);
        auto recovered = to_eigen_isometry3d(std::move(rust_vec));

        // Should be nearly identical
        auto diff = (original.matrix() - recovered.matrix()).norm();
        REQUIRE_THAT(diff, WithinAbs(0.0, 1e-15));
    }

    SECTION("Multiple transforms conversion") {
        std::vector<Eigen::Isometry3d> transforms = {
            Eigen::Isometry3d::Identity(),
            Eigen::Isometry3d(Eigen::Translation3d(1.0, 0.0, 0.0)),
            Eigen::Isometry3d(Eigen::AngleAxisd(M_PI / 2.0, Eigen::Vector3d::UnitZ()))
        };

        auto rust_vec = to_rust_vec(transforms);
        auto recovered = to_eigen_isometry_vector(std::move(rust_vec));

        REQUIRE(recovered.size() == transforms.size());

        for (size_t i = 0; i < transforms.size(); ++i) {
            auto diff = (transforms[i].matrix() - recovered[i].matrix()).norm();
            REQUIRE_THAT(diff, WithinAbs(0.0, 1e-15));
        }
    }

    SECTION("Limits conversion") {
        rust::Vec<double> limits_vec;
        limits_vec.push_back(-M_PI);
        limits_vec.push_back(M_PI);

        auto limits = to_limits_pair(limits_vec);
        REQUIRE_THAT(limits.first, WithinAbs(-M_PI, 1e-15));
        REQUIRE_THAT(limits.second, WithinAbs(M_PI, 1e-15));
    }

    SECTION("Invalid limits conversion") {
        rust::Vec<double> invalid_limits_vec;
        invalid_limits_vec.push_back(1.0); // Only one element

        auto limits = to_limits_pair(invalid_limits_vec);
        REQUIRE_THAT(limits.first, WithinAbs(0.0, 1e-15));
        REQUIRE_THAT(limits.second, WithinAbs(0.0, 1e-15));
    }
}

TEST_CASE("Cxx rust::Slice integration", "[joint][cxx][slice]") {
    auto joint = new_joint("slice_test_cxx");

    SECTION("Using Eigen vector with rust slice conversion") {
        Eigen::VectorXd eigen_vars(3);
        eigen_vars << 0.5, 1.0, 1.5;

        // Convert to rust slice
        auto rust_slice = to_rust_slice(eigen_vars);

        // Use the slice directly
        auto transform_vec = joint->calculate_transform(rust_slice);
        auto transform = to_eigen_isometry3d(std::move(transform_vec));

        // Should use the first element (0.5)
        Eigen::AngleAxisd angle_axis(transform.rotation());
        REQUIRE_THAT(angle_axis.angle(), WithinAbs(0.5, 1e-10));
    }
}

TEST_CASE("Cxx Memory safety and performance", "[joint][cxx][safety]") {
    SECTION("Multiple joints operations") {
        auto joint1 = new_joint("perf_joint_1");
        auto joint2 = new_joint("perf_joint_2");
        auto joint3 = new_joint("perf_joint_3");

        // Perform operations on all joints
        for (int i = 0; i < 10; ++i) {
            rust::Vec<double> vars;
            vars.push_back(static_cast<double>(i) * 0.1);

            auto t1 = joint1->calculate_transform(to_rust_slice(vars));
            auto t2 = joint2->calculate_transform(to_rust_slice(vars));
            auto t3 = joint3->calculate_transform(to_rust_slice(vars));

            // All should be valid 16-element vectors
            REQUIRE(t1.size() == 16);
            REQUIRE(t2.size() == 16);
            REQUIRE(t3.size() == 16);
        }
    }

    SECTION("Large variable vectors") {
        auto joint = new_joint("large_vars_joint");

        rust::Vec<double> large_vars;
        for (int i = 0; i < 1000; ++i) {
            large_vars.push_back(0.001 * i);
        }

        // Should handle gracefully
        auto transform_vec = joint->calculate_transform(to_rust_slice(large_vars));
        REQUIRE(transform_vec.size() == 16);

        auto transform = to_eigen_isometry3d(std::move(transform_vec));
        REQUIRE_THAT(transform.translation().norm(), WithinAbs(0.0, 1e-10));
    }

    SECTION("Repeated calculations consistency") {
        auto joint = new_joint("consistency_joint");

        rust::Vec<double> vars;
        vars.push_back(M_PI / 3.0);

        // Multiple calls should be identical
        auto transform1_vec = joint->calculate_transform(to_rust_slice(vars));
        auto transform2_vec = joint->calculate_transform(to_rust_slice(vars));

        REQUIRE(transform1_vec.size() == transform2_vec.size());

        for (size_t i = 0; i < transform1_vec.size(); ++i) {
            REQUIRE_THAT(transform1_vec[i], WithinAbs(transform2_vec[i], 1e-15));
        }
    }
}

TEST_CASE("Cxx Joint display and string operations", "[joint][cxx][display]") {
    auto joint = new_joint("display_joint_cxx");

    SECTION("String representation") {
        std::string display_str = to_std_string(joint->to_string());

        REQUIRE_FALSE(display_str.empty());
        REQUIRE(display_str.find("display_joint_cxx") != std::string::npos);
        REQUIRE(display_str.find("index:") != std::string::npos);
    }

    SECTION("Name retrieval") {
        std::string name = to_std_string(joint->name());
        REQUIRE(name == "display_joint_cxx");
    }
}
