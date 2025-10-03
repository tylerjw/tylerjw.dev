//! Cxx-based bindings for the robot_joint library
//!
//! This module provides C++ interop using the cxx crate, which generates
//! safe bindings between Rust and C++. This approach reduces unsafe code
//! compared to manual FFI but requires type conversions for library types.

use std::fmt;

/// Wrapper around the pure Rust Joint for cxx interop
///
/// We use a newtype pattern because cxx requires types to be defined
/// in the same crate where the bridge is declared.
#[derive(Clone, Debug)]
pub struct Joint(robot_joint::Joint);

#[cxx::bridge(namespace = "robot_joint")]
mod ffi {
    extern "Rust" {
        type Joint;

        // Factory function
        fn new_joint(name: &str) -> Box<Joint>;

        // Property getters
        fn name(self: &Joint) -> String;
        fn parent_link_index(self: &Joint) -> usize;
        fn child_link_index(self: &Joint) -> usize;
        fn index(self: &Joint) -> usize;
        fn dof_index(self: &Joint) -> usize;

        // Transform calculations
        fn calculate_transform(self: &Joint, variables: &[f64]) -> Vec<f64>;
        fn parent_link_to_joint_origin(self: &Joint) -> Vec<f64>;

        // Joint limits
        fn is_within_limits(self: &Joint, position: f64) -> bool;
        fn get_limits(self: &Joint) -> Vec<f64>; // Returns [min, max]

        // Utility functions
        fn to_string(self: &Joint) -> String;
    }
}

/// Create a new joint with the given name
fn new_joint(name: &str) -> Box<Joint> {
    Box::new(Joint(robot_joint::Joint::new(name.to_string())))
}

impl Joint {
    /// Get the joint name
    fn name(&self) -> String {
        self.0.name().to_string()
    }

    /// Get the parent link index
    fn parent_link_index(&self) -> usize {
        self.0.parent_link_index()
    }

    /// Get the child link index
    fn child_link_index(&self) -> usize {
        self.0.child_link_index()
    }

    /// Get the joint index
    fn index(&self) -> usize {
        self.0.index()
    }

    /// Get the DOF index
    fn dof_index(&self) -> usize {
        self.0.dof_index()
    }

    /// Calculate transform and return as Vec<f64> (16 elements, column-major)
    fn calculate_transform(&self, variables: &[f64]) -> Vec<f64> {
        convert::vec_from_isometry3(self.0.calculate_transform(variables))
    }

    /// Get parent link to joint origin transform as Vec<f64>
    fn parent_link_to_joint_origin(&self) -> Vec<f64> {
        convert::vec_from_isometry3(*self.0.parent_link_to_joint_origin())
    }

    /// Check if position is within joint limits
    fn is_within_limits(&self, position: f64) -> bool {
        self.0.is_within_limits(position)
    }

    /// Get joint limits as [min, max]
    fn get_limits(&self) -> Vec<f64> {
        let (min, max) = self.0.limits();
        vec![min, max]
    }

    /// Convert to string representation
    fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

impl fmt::Display for Joint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Conversion utilities for interop between Rust and C++ types
mod convert {
    use nalgebra::{DMatrix, Isometry3};

    #[allow(dead_code)]

    /// Convert Isometry3<f64> to Vec<f64> (16 elements, column-major)
    pub fn vec_from_isometry3(transform: Isometry3<f64>) -> Vec<f64> {
        let matrix = transform.to_matrix();
        // Convert to column-major ordering (Eigen's default)
        let mut result = Vec::with_capacity(16);
        for col in 0..4 {
            for row in 0..4 {
                result.push(matrix[(row, col)]);
            }
        }
        result
    }

    /// Convert slice of f64 to Isometry3<f64>
    /// Expects 16 elements in column-major order
    #[allow(dead_code)]
    pub fn isometry3_from_slice(data: &[f64]) -> Option<Isometry3<f64>> {
        if data.len() != 16 {
            return None;
        }

        // Convert from column-major to nalgebra's matrix format
        let mut matrix_data = [0.0; 16];
        for col in 0..4 {
            for row in 0..4 {
                matrix_data[row * 4 + col] = data[col * 4 + row];
            }
        }

        let matrix = nalgebra::Matrix4::from_row_slice(&matrix_data);

        // Extract translation from the last column
        let translation =
            nalgebra::Translation3::new(matrix[(0, 3)], matrix[(1, 3)], matrix[(2, 3)]);

        // Extract rotation matrix (top-left 3x3)
        let rotation_matrix = matrix.fixed_view::<3, 3>(0, 0).into_owned();
        let rotation = nalgebra::UnitQuaternion::from_matrix(&rotation_matrix);

        Some(nalgebra::Isometry3::from_parts(translation, rotation))
    }

    /// Convert Vec<f64> to Isometry3<f64>
    #[allow(dead_code)]
    pub fn isometry3_from_vec(data: Vec<f64>) -> Option<Isometry3<f64>> {
        isometry3_from_slice(&data)
    }

    /// Convert multiple Isometry3 transforms to a flat Vec<f64>
    #[allow(dead_code)]
    pub fn vec_from_vec_isometry3(transforms: Vec<Isometry3<f64>>) -> Vec<f64> {
        transforms
            .into_iter()
            .flat_map(vec_from_isometry3)
            .collect()
    }

    /// Convert flat Vec<f64> to multiple Isometry3 transforms
    #[allow(dead_code)]
    pub fn vec_isometry3_from_vec(data: Vec<f64>) -> Vec<Isometry3<f64>> {
        data.chunks(16).filter_map(isometry3_from_slice).collect()
    }

    /// Convert DMatrix to Vec<f64> for Jacobian matrices
    #[allow(dead_code)]
    pub fn vec_from_matrix6x(matrix: DMatrix<f64>) -> Vec<f64> {
        matrix.as_slice().to_vec()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use nalgebra::{Isometry3, Translation3, Unit, UnitQuaternion, Vector3};
        use std::f64::consts::PI;

        #[test]
        fn test_isometry3_round_trip() {
            let original = Isometry3::from_parts(
                Translation3::new(1.0, 2.0, 3.0),
                UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::z()), PI / 4.0),
            );

            let vec = vec_from_isometry3(original);
            let recovered = isometry3_from_vec(vec).unwrap();

            let diff = (original.to_matrix() - recovered.to_matrix()).norm();
            assert!(
                diff < 1e-10,
                "Round-trip conversion failed: diff = {}",
                diff
            );
        }

        #[test]
        fn test_identity_conversion() {
            let identity = Isometry3::identity();
            let vec = vec_from_isometry3(identity);
            let recovered = isometry3_from_vec(vec.clone()).unwrap();

            let expected_vec = vec![
                1.0, 0.0, 0.0, 0.0, // Column 0
                0.0, 1.0, 0.0, 0.0, // Column 1
                0.0, 0.0, 1.0, 0.0, // Column 2
                0.0, 0.0, 0.0, 1.0, // Column 3
            ];

            for (actual, expected) in vec.iter().zip(expected_vec.iter()) {
                assert!((actual - expected).abs() < 1e-15);
            }

            assert!((identity.to_matrix() - recovered.to_matrix()).norm() < 1e-15);
        }

        #[test]
        fn test_invalid_slice_length() {
            let short_vec = vec![1.0, 2.0, 3.0];
            assert!(isometry3_from_slice(&short_vec).is_none());

            let long_vec = vec![0.0; 20];
            assert!(isometry3_from_slice(&long_vec).is_none());
        }

        #[test]
        fn test_multiple_transforms() {
            let transforms = vec![
                Isometry3::identity(),
                Isometry3::translation(1.0, 0.0, 0.0),
                Isometry3::from_parts(
                    Translation3::new(0.0, 1.0, 0.0),
                    UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::z()), PI / 2.0),
                ),
            ];

            let flat_vec = vec_from_vec_isometry3(transforms.clone());
            let recovered = vec_isometry3_from_vec(flat_vec);

            assert_eq!(recovered.len(), transforms.len());

            for (original, recovered) in transforms.iter().zip(recovered.iter()) {
                let diff = (original.to_matrix() - recovered.to_matrix()).norm();
                assert!(
                    diff < 1e-10,
                    "Multi-transform conversion failed: diff = {}",
                    diff
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joint_creation() {
        let joint = new_joint("test_joint_cxx");
        assert_eq!(joint.name(), "test_joint_cxx");
        assert_eq!(joint.index(), 0);
        assert_eq!(joint.parent_link_index(), 0);
        assert_eq!(joint.child_link_index(), 1);
        assert_eq!(joint.dof_index(), 0);
    }

    #[test]
    fn test_joint_transform() {
        let joint = new_joint("transform_test_cxx");
        let variables = vec![0.0];
        let transform_vec = joint.calculate_transform(&variables);

        // Should be identity matrix
        let expected_identity = vec![
            1.0, 0.0, 0.0, 0.0, // Column 0
            0.0, 1.0, 0.0, 0.0, // Column 1
            0.0, 0.0, 1.0, 0.0, // Column 2
            0.0, 0.0, 0.0, 1.0, // Column 3
        ];

        assert_eq!(transform_vec.len(), 16);
        for (actual, expected) in transform_vec.iter().zip(expected_identity.iter()) {
            assert!((actual - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn test_joint_limits() {
        let joint = new_joint("limits_test_cxx");
        let limits = joint.get_limits();

        assert_eq!(limits.len(), 2);
        assert_eq!(limits[0], -std::f64::consts::PI);
        assert_eq!(limits[1], std::f64::consts::PI);

        assert!(joint.is_within_limits(0.0));
        assert!(!joint.is_within_limits(4.0));
    }

    #[test]
    fn test_joint_display() {
        let joint = new_joint("display_test_cxx");
        let display_str = joint.to_string();
        assert!(display_str.contains("display_test_cxx"));
    }

    #[test]
    fn test_parent_transform() {
        let joint = new_joint("parent_test_cxx");
        let parent_transform = joint.parent_link_to_joint_origin();

        // Should be identity by default
        assert_eq!(parent_transform.len(), 16);
        let expected_identity = vec![
            1.0, 0.0, 0.0, 0.0, // Column 0
            0.0, 1.0, 0.0, 0.0, // Column 1
            0.0, 0.0, 1.0, 0.0, // Column 2
            0.0, 0.0, 0.0, 1.0, // Column 3
        ];

        for (actual, expected) in parent_transform.iter().zip(expected_identity.iter()) {
            assert!((actual - expected).abs() < 1e-10);
        }
    }
}
