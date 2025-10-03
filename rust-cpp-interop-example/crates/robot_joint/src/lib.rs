//! # Robot Joint Library
//!
//! A pure Rust library for representing and manipulating robot joints.
//! This library demonstrates clean Rust API design that will be exposed
//! to C++ through various interop patterns.
//!
//! ## Features
//!
//! - Joint representation with transforms
//! - Forward kinematics calculations
//! - Integration with nalgebra for linear algebra
//!
//! ## Example
//!
//! ```rust
//! use robot_joint::Joint;
//! use nalgebra::Vector1;
//!
//! let joint = Joint::new("shoulder_joint".to_string());
//! let variables = Vector1::new(1.57); // 90 degrees
//! let transform = joint.calculate_transform(variables.as_slice());
//! ```

pub mod joint;

pub use joint::Joint;
pub use nalgebra::{Isometry3, Vector3};

/// Common result type for this library
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for robot joint operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid joint configuration
    #[error("Invalid joint configuration: {message}")]
    InvalidConfiguration { message: String },

    /// Invalid variable count
    #[error("Expected {expected} variables, got {actual}")]
    InvalidVariableCount { expected: usize, actual: usize },

    /// Mathematical error in calculations
    #[error("Mathematical error: {message}")]
    MathError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joint_creation() {
        let joint = Joint::new("test_joint".to_string());
        assert_eq!(joint.name(), "test_joint");
        assert_eq!(joint.index(), 0);
    }

    #[test]
    fn test_joint_transform() {
        let joint = Joint::new("revolute_joint".to_string());
        let variables = vec![0.0];

        let transform = joint.calculate_transform(&variables);

        // At zero position, should be identity
        let identity = Isometry3::identity();
        let diff = (transform.translation.vector - identity.translation.vector).norm();
        assert!(diff < 1e-10, "Translation should be near identity");
    }

    #[test]
    fn test_joint_transform_nonzero() {
        let joint = Joint::new("revolute_joint".to_string());
        let variables = vec![std::f64::consts::PI / 2.0]; // 90 degrees

        let transform = joint.calculate_transform(&variables);

        // Should not be identity for non-zero rotation
        let identity = Isometry3::identity();
        let rotation_diff =
            (transform.rotation.quaternion() - identity.rotation.quaternion()).norm();
        assert!(rotation_diff > 1e-6, "Should have non-zero rotation");
    }

    #[test]
    fn test_joint_debug() {
        let joint = Joint::new("debug_joint".to_string());
        let debug_str = format!("{:?}", joint);
        assert!(debug_str.contains("debug_joint"));
    }
}
