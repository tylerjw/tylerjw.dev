#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_floating_point.hpp>
#include <robot_joint.hpp>
#include <Eigen/Geometry>
#include <type_traits>
#include <cmath>

using namespace robot_joint;
using namespace Catch::Matchers;

// Static assertions to verify the Joint class has expected properties
static_assert(std::is_nothrow_destructible_v<Joint>);
static_assert(!std::is_copy_constructible_v<Joint>);
static_assert(!std::is_copy_assignable_v<Joint>);
static_assert(std::is_nothrow_move_constructible_v<Joint>);
static_assert(std::is_nothrow_move_assignable_v<Joint>);

TEST_CASE("Joint creation and basic properties", "[joint][creation]") {
    SECTION("Create joint with name") {
        Joint joint("test_joint");

        REQUIRE(joint.name() == "test_joint");
        REQUIRE(joint.index() == 0);
        REQUIRE(joint.parent_link_index() == 0);
        REQUIRE(joint.child_link_index() == 1);
        REQUIRE(joint.dof_index() == 0);
    }

    SECTION("Create joint with empty name") {
        Joint joint("");
        REQUIRE(joint.name() == "");
    }

    SECTION("Create joint with special characters in name") {
        Joint joint("joint_with-special.chars");
        REQUIRE(joint.name() == "joint_with-special.chars");
    }
}

TEST_CASE("Joint move semantics", "[joint][move]") {
    SECTION("Move constructor") {
        Joint joint1("moveable_joint");
        std::string original_name = joint1.name();

        Joint joint2 = std::move(joint1);
        REQUIRE(joint2.name() == original_name);
    }

    SECTION("Move assignment") {
        Joint joint1("source_joint");
        Joint joint2("target_joint");

        std::string source_name = joint1.name();
        joint2 = std::move(joint1);

        REQUIRE(joint2.name() == source_name);
    }
}

TEST_CASE("Transform calculations", "[joint][transform]") {
    Joint joint("transform_joint");

    SECTION("Identity transform at zero position") {
        Eigen::VectorXd variables(1);
        variables << 0.0;

        Eigen::Isometry3d transform = joint.calculate_transform(variables);

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
        Eigen::VectorXd variables(1);
        variables << M_PI / 2.0;  // 90 degrees

        Eigen::Isometry3d transform = joint.calculate_transform(variables);

        // Should have rotation but no translation
        REQUIRE_THAT(transform.translation().x(), WithinAbs(0.0, 1e-10));
        REQUIRE_THAT(transform.translation().y(), WithinAbs(0.0, 1e-10));
        REQUIRE_THAT(transform.translation().z(), WithinAbs(0.0, 1e-10));

        // Check that we have a 90-degree rotation around Z-axis
        Eigen::AngleAxisd angle_axis(transform.rotation());
        REQUIRE_THAT(angle_axis.angle(), WithinAbs(M_PI / 2.0, 1e-10));
    }

    SECTION("Full rotation") {
        Eigen::VectorXd variables(1);
        variables << 2.0 * M_PI;  // 360 degrees

        Eigen::Isometry3d transform = joint.calculate_transform(variables);

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
        Eigen::VectorXd variables(3);
        variables << M_PI / 4.0, 1.0, 2.0;  // Only first should be used

        Eigen::Isometry3d transform = joint.calculate_transform(variables);

        // Should use only the first variable
        Eigen::AngleAxisd angle_axis(transform.rotation());
        REQUIRE_THAT(angle_axis.angle(), WithinAbs(M_PI / 4.0, 1e-10));
    }

    SECTION("Empty variables vector") {
        Eigen::VectorXd variables(0);

        Eigen::Isometry3d transform = joint.calculate_transform(variables);

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

TEST_CASE("Parent link to joint origin transform", "[joint][parent_transform]") {
    Joint joint("origin_joint");

    SECTION("Default parent transform") {
        Eigen::Isometry3d transform = joint.parent_link_to_joint_origin();

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

TEST_CASE("Joint limits", "[joint][limits]") {
    Joint joint("limited_joint");

    SECTION("Default limits") {
        auto [min_limit, max_limit] = joint.limits();

        REQUIRE_THAT(min_limit, WithinAbs(-M_PI, 1e-10));
        REQUIRE_THAT(max_limit, WithinAbs(M_PI, 1e-10));
    }

    SECTION("Within limits check") {
        REQUIRE(joint.is_within_limits(0.0));
        REQUIRE(joint.is_within_limits(M_PI - 0.1));
        REQUIRE(joint.is_within_limits(-M_PI + 0.1));
        REQUIRE(joint.is_within_limits(M_PI));
        REQUIRE(joint.is_within_limits(-M_PI));
    }

    SECTION("Outside limits check") {
        REQUIRE_FALSE(joint.is_within_limits(M_PI + 0.1));
        REQUIRE_FALSE(joint.is_within_limits(-M_PI - 0.1));
        REQUIRE_FALSE(joint.is_within_limits(2.0 * M_PI));
        REQUIRE_FALSE(joint.is_within_limits(-2.0 * M_PI));
    }
}

TEST_CASE("Memory safety and edge cases", "[joint][safety]") {
    SECTION("Multiple joints don't interfere") {
        Joint joint1("joint_1");
        Joint joint2("joint_2");
        Joint joint3("joint_3");

        REQUIRE(joint1.name() == "joint_1");
        REQUIRE(joint2.name() == "joint_2");
        REQUIRE(joint3.name() == "joint_3");

        // All should have same properties but different names
        REQUIRE(joint1.index() == joint2.index());
        REQUIRE(joint2.index() == joint3.index());
    }

    SECTION("Joint survives scope changes") {
        std::unique_ptr<Joint> joint_ptr;

        {
            Joint temp_joint("scoped_joint");
            joint_ptr = std::make_unique<Joint>(std::move(temp_joint));
        }

        // Should still be valid after temp_joint goes out of scope
        REQUIRE(joint_ptr->name() == "scoped_joint");

        Eigen::VectorXd variables(1);
        variables << 0.5;

        // Should still be able to calculate transforms
        Eigen::Isometry3d transform = joint_ptr->calculate_transform(variables);
        REQUIRE_THAT(transform.translation().norm(), WithinAbs(0.0, 1e-10));
    }
}

TEST_CASE("Performance considerations", "[joint][performance]") {
    Joint joint("perf_joint");

    SECTION("Repeated transform calculations") {
        Eigen::VectorXd variables(1);

        // Multiple calculations should be consistent
        for (int i = 0; i < 100; ++i) {
            variables << static_cast<double>(i) * 0.01;

            Eigen::Isometry3d transform1 = joint.calculate_transform(variables);
            Eigen::Isometry3d transform2 = joint.calculate_transform(variables);

            // Should be identical
            REQUIRE_THAT((transform1.matrix() - transform2.matrix()).norm(),
                        WithinAbs(0.0, 1e-15));
        }
    }

    SECTION("Large variable vectors") {
        Eigen::VectorXd large_variables(1000);
        large_variables.setConstant(0.5);

        // Should handle large vectors gracefully (only using first element)
        Eigen::Isometry3d transform = joint.calculate_transform(large_variables);
        REQUIRE_THAT(transform.translation().norm(), WithinAbs(0.0, 1e-10));

        // Should be same as single-element vector
        Eigen::VectorXd single_variable(1);
        single_variable << 0.5;
        Eigen::Isometry3d single_transform = joint.calculate_transform(single_variable);

        REQUIRE_THAT((transform.matrix() - single_transform.matrix()).norm(),
                    WithinAbs(0.0, 1e-15));
    }
}
