use nalgebra::{Isometry3, Translation3, Unit, UnitQuaternion, Vector3};

/// Represents a robot joint with its kinematic properties
#[derive(Clone, Debug)]
pub struct Joint {
    /// Name of the joint
    name: String,

    /// Transform from parent link to joint origin
    parent_link_to_joint_origin: Isometry3<f64>,

    /// Index of the parent link
    parent_link_index: usize,

    /// Index of the child link
    child_link_index: usize,

    /// Index of this joint in the kinematic chain
    index: usize,

    /// Index of the degree of freedom for this joint
    dof_index: usize,

    /// Joint axis (for revolute joints)
    axis: Vector3<f64>,
}

impl Joint {
    /// Create a new joint with the given name
    ///
    /// # Arguments
    /// * `name` - Name identifier for the joint
    ///
    /// # Example
    /// ```rust
    /// use robot_joint::Joint;
    ///
    /// let joint = Joint::new("shoulder_joint".to_string());
    /// assert_eq!(joint.name(), "shoulder_joint");
    /// ```
    pub fn new(name: String) -> Self {
        Self {
            name,
            parent_link_to_joint_origin: Isometry3::identity(),
            parent_link_index: 0,
            child_link_index: 1,
            index: 0,
            dof_index: 0,
            axis: Vector3::z(), // Default to Z-axis rotation
        }
    }

    /// Create a new joint with full configuration
    pub fn new_with_config(
        name: String,
        parent_link_to_joint_origin: Isometry3<f64>,
        parent_link_index: usize,
        child_link_index: usize,
        index: usize,
        dof_index: usize,
        axis: Vector3<f64>,
    ) -> Self {
        Self {
            name,
            parent_link_to_joint_origin,
            parent_link_index,
            child_link_index,
            index,
            dof_index,
            axis: axis.normalize(),
        }
    }

    /// Get the joint name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the parent link to joint origin transform
    pub fn parent_link_to_joint_origin(&self) -> &Isometry3<f64> {
        &self.parent_link_to_joint_origin
    }

    /// Get the parent link index
    pub fn parent_link_index(&self) -> usize {
        self.parent_link_index
    }

    /// Get the child link index
    pub fn child_link_index(&self) -> usize {
        self.child_link_index
    }

    /// Get the joint index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the DOF index
    pub fn dof_index(&self) -> usize {
        self.dof_index
    }

    /// Get the joint axis
    pub fn axis(&self) -> &Vector3<f64> {
        &self.axis
    }

    /// Calculate the transform for this joint given joint variables
    ///
    /// # Arguments
    /// * `variables` - Joint variable values (angles for revolute joints)
    ///
    /// # Returns
    /// The resulting transform as an Isometry3<f64>
    ///
    /// # Example
    /// ```rust
    /// use robot_joint::Joint;
    /// use std::f64::consts::PI;
    ///
    /// let joint = Joint::new("elbow_joint".to_string());
    /// let variables = vec![PI / 2.0]; // 90 degrees
    /// let transform = joint.calculate_transform(&variables);
    /// ```
    pub fn calculate_transform(&self, variables: &[f64]) -> Isometry3<f64> {
        if variables.is_empty() {
            return self.parent_link_to_joint_origin;
        }

        // For revolute joint, rotate around the axis by the joint variable
        let angle = variables[0];
        let rotation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(self.axis), angle);
        let joint_transform = Isometry3::from_parts(Translation3::identity(), rotation);

        // Combine with the parent link to joint origin transform
        self.parent_link_to_joint_origin * joint_transform
    }

    /// Calculate the transform matrix as a flat array (column-major)
    ///
    /// This is useful for FFI where we need to pass the matrix data
    /// as a contiguous array of doubles.
    pub fn calculate_transform_matrix(&self, variables: &[f64]) -> [f64; 16] {
        let transform = self.calculate_transform(variables);
        let matrix = transform.to_matrix();

        // Convert to column-major array (Eigen's default)
        let mut result = [0.0; 16];
        for col in 0..4 {
            for row in 0..4 {
                result[col * 4 + row] = matrix[(row, col)];
            }
        }
        result
    }

    /// Get joint limits (placeholder implementation)
    pub fn limits(&self) -> (f64, f64) {
        (-std::f64::consts::PI, std::f64::consts::PI)
    }

    /// Check if a joint position is within limits
    pub fn is_within_limits(&self, position: f64) -> bool {
        let (min, max) = self.limits();
        position >= min && position <= max
    }

    /// Set the joint axis
    pub fn set_axis(&mut self, axis: Vector3<f64>) {
        self.axis = axis.normalize();
    }

    /// Set the parent link to joint origin transform
    pub fn set_parent_link_to_joint_origin(&mut self, transform: Isometry3<f64>) {
        self.parent_link_to_joint_origin = transform;
    }
}

impl Default for Joint {
    fn default() -> Self {
        Self::new("unnamed_joint".to_string())
    }
}

impl std::fmt::Display for Joint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Joint '{}' (index: {}, dof: {}, parent: {} -> child: {})",
            self.name, self.index, self.dof_index, self.parent_link_index, self.child_link_index
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Translation3, UnitQuaternion, Vector3};
    use std::f64::consts::PI;

    #[test]
    fn test_joint_creation() {
        let joint = Joint::new("test_joint".to_string());
        assert_eq!(joint.name(), "test_joint");
        assert_eq!(joint.index(), 0);
        assert_eq!(joint.dof_index(), 0);
        assert_eq!(joint.parent_link_index(), 0);
        assert_eq!(joint.child_link_index(), 1);
    }

    #[test]
    fn test_joint_with_config() {
        let translation = Translation3::new(1.0, 2.0, 3.0);
        let rotation = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), PI / 4.0);
        let transform = Isometry3::from_parts(translation, rotation);
        let axis = Vector3::new(1.0, 1.0, 0.0);

        let joint = Joint::new_with_config(
            "configured_joint".to_string(),
            transform,
            5,
            6,
            10,
            15,
            axis,
        );

        assert_eq!(joint.name(), "configured_joint");
        assert_eq!(joint.parent_link_index(), 5);
        assert_eq!(joint.child_link_index(), 6);
        assert_eq!(joint.index(), 10);
        assert_eq!(joint.dof_index(), 15);
        assert!((joint.axis().norm() - 1.0).abs() < 1e-10); // Should be normalized
    }

    #[test]
    fn test_calculate_transform_identity() {
        let joint = Joint::new("identity_joint".to_string());
        let variables = vec![0.0];
        let transform = joint.calculate_transform(&variables);

        // Should be identity when angle is 0
        let identity = Isometry3::identity();
        let translation_diff = (transform.translation.vector - identity.translation.vector).norm();
        let rotation_diff = transform.rotation.angle() - identity.rotation.angle();

        assert!(translation_diff < 1e-10);
        assert!(rotation_diff.abs() < 1e-10);
    }

    #[test]
    fn test_calculate_transform_90_degrees() {
        let joint = Joint::new("rotation_joint".to_string());
        let variables = vec![PI / 2.0];
        let transform = joint.calculate_transform(&variables);

        // Should have 90-degree rotation around Z-axis
        assert!((transform.rotation.angle() - PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_transform_matrix() {
        let joint = Joint::new("matrix_joint".to_string());
        let variables = vec![0.0];
        let matrix_array = joint.calculate_transform_matrix(&variables);

        // Should be identity matrix in column-major format
        let expected_identity = [
            1.0, 0.0, 0.0, 0.0, // Column 0
            0.0, 1.0, 0.0, 0.0, // Column 1
            0.0, 0.0, 1.0, 0.0, // Column 2
            0.0, 0.0, 0.0, 1.0, // Column 3
        ];

        for (i, (&actual, &expected)) in matrix_array
            .iter()
            .zip(expected_identity.iter())
            .enumerate()
        {
            assert!(
                (actual - expected).abs() < 1e-10,
                "Mismatch at index {}: {} vs {}",
                i,
                actual,
                expected
            );
        }
    }

    #[test]
    fn test_limits() {
        let joint = Joint::new("limited_joint".to_string());
        let (min, max) = joint.limits();

        assert_eq!(min, -PI);
        assert_eq!(max, PI);

        assert!(joint.is_within_limits(0.0));
        assert!(joint.is_within_limits(PI - 0.1));
        assert!(joint.is_within_limits(-PI + 0.1));
        assert!(!joint.is_within_limits(PI + 0.1));
        assert!(!joint.is_within_limits(-PI - 0.1));
    }

    #[test]
    fn test_display() {
        let joint = Joint::new("display_joint".to_string());
        let display_str = format!("{}", joint);
        assert!(display_str.contains("display_joint"));
        assert!(display_str.contains("index: 0"));
    }

    #[test]
    fn test_axis_normalization() {
        let mut joint = Joint::new("axis_joint".to_string());
        let unnormalized_axis = Vector3::new(3.0, 4.0, 0.0);

        joint.set_axis(unnormalized_axis);

        // Should be normalized to unit length
        assert!((joint.axis().norm() - 1.0).abs() < 1e-10);

        // Should maintain direction
        let normalized_expected = unnormalized_axis.normalize();
        let diff = (joint.axis() - normalized_expected).norm();
        assert!(diff < 1e-10);
    }

    #[test]
    fn test_transform_with_offset() {
        let mut joint = Joint::new("offset_joint".to_string());

        // Set an offset transform
        let offset = Isometry3::translation(1.0, 2.0, 3.0);
        joint.set_parent_link_to_joint_origin(offset);

        let variables = vec![0.0];
        let result_transform = joint.calculate_transform(&variables);

        // Should include the offset
        let translation = result_transform.translation.vector;
        assert!((translation.x - 1.0).abs() < 1e-10);
        assert!((translation.y - 2.0).abs() < 1e-10);
        assert!((translation.z - 3.0).abs() < 1e-10);
    }
}
