//! Manual FFI bindings for the robot_joint library
//!
//! This module provides a C-compatible interface to the pure Rust robot_joint
//! library, following the patterns from the blog post series on Rust/C++ interop.
//!
//! The approach uses:
//! - `#[no_mangle]` functions with C calling convention
//! - Opaque pointer types for safe memory management
//! - Manual type conversions between Rust and C types
//! - Box allocation/deallocation patterns

use robot_joint::Joint;
use std::ffi::{CStr, CString, c_char, c_double, c_uint};
use std::ptr;

/// Opaque handle to a Rust Joint object
/// This allows C++ code to hold references to Rust objects safely
pub struct RobotJointHandle {
    joint: Joint,
}

/// C-compatible representation of a 4x4 transformation matrix
/// Data is stored in column-major order (compatible with Eigen)
#[repr(C)]
pub struct Mat4d {
    pub data: [c_double; 16],
}

/// Create a new robot joint with the given name
///
/// # Safety
/// The returned pointer must be freed using `robot_joint_free`
/// The name pointer must be valid and null-terminated
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_new(name: *const c_char) -> *mut RobotJointHandle {
    if name.is_null() {
        return ptr::null_mut();
    }

    let name_cstr = unsafe { CStr::from_ptr(name) };
    let name_str = match name_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let joint = Joint::new(name_str.to_string());
    let handle = RobotJointHandle { joint };

    Box::into_raw(Box::new(handle))
}

/// Free a robot joint handle
///
/// # Safety
/// The joint pointer must be a valid pointer returned from `robot_joint_new`
/// and must not be used after this call
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_free(joint: *mut RobotJointHandle) {
    if !joint.is_null() {
        unsafe {
            drop(Box::from_raw(joint));
        }
    }
}

/// Get the name of a joint
///
/// # Safety
/// The joint pointer must be valid
/// The returned string pointer is valid until the joint is freed or modified
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_get_name(joint: *const RobotJointHandle) -> *const c_char {
    if joint.is_null() {
        return ptr::null();
    }

    let handle = unsafe { &*joint };
    let name_cstring = match CString::new(handle.joint.name()) {
        Ok(s) => s,
        Err(_) => return ptr::null(),
    };

    // Leak the string so it stays valid for the caller
    // This is a simplified approach; in production, you might want
    // a more sophisticated string management strategy
    name_cstring.into_raw()
}

/// Free a string returned by robot_joint_get_name
///
/// # Safety
/// The string pointer must be a valid pointer returned from `robot_joint_get_name`
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

/// Get the index of a joint
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_get_index(joint: *const RobotJointHandle) -> c_uint {
    if joint.is_null() {
        return 0;
    }

    let handle = unsafe { &*joint };
    handle.joint.index() as c_uint
}

/// Get the parent link index of a joint
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_get_parent_link_index(joint: *const RobotJointHandle) -> c_uint {
    if joint.is_null() {
        return 0;
    }

    let handle = unsafe { &*joint };
    handle.joint.parent_link_index() as c_uint
}

/// Get the child link index of a joint
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_get_child_link_index(joint: *const RobotJointHandle) -> c_uint {
    if joint.is_null() {
        return 0;
    }

    let handle = unsafe { &*joint };
    handle.joint.child_link_index() as c_uint
}

/// Get the DOF index of a joint
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_get_dof_index(joint: *const RobotJointHandle) -> c_uint {
    if joint.is_null() {
        return 0;
    }

    let handle = unsafe { &*joint };
    handle.joint.dof_index() as c_uint
}

/// Calculate the transformation matrix for given joint variables
///
/// # Safety
/// - joint pointer must be valid
/// - variables pointer must point to at least `size` elements
/// - The returned Mat4d contains the transformation matrix in column-major order
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_calculate_transform(
    joint: *const RobotJointHandle,
    variables: *const c_double,
    size: c_uint,
) -> Mat4d {
    // Return identity matrix on error
    let identity = Mat4d {
        data: [
            1.0, 0.0, 0.0, 0.0, // Column 0
            0.0, 1.0, 0.0, 0.0, // Column 1
            0.0, 0.0, 1.0, 0.0, // Column 2
            0.0, 0.0, 0.0, 1.0, // Column 3
        ],
    };

    if joint.is_null() || variables.is_null() {
        return identity;
    }

    let handle = unsafe { &*joint };
    let variables_slice = unsafe { std::slice::from_raw_parts(variables, size as usize) };

    let transform = handle.joint.calculate_transform(variables_slice);
    let matrix = transform.to_matrix();

    // Convert nalgebra matrix to column-major array (Eigen compatible)
    let mut result = Mat4d { data: [0.0; 16] };
    for col in 0..4 {
        for row in 0..4 {
            result.data[col * 4 + row] = matrix[(row, col)];
        }
    }

    result
}

/// Get the parent link to joint origin transformation matrix
///
/// # Safety
/// joint pointer must be valid
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_get_parent_link_to_joint_origin(
    joint: *const RobotJointHandle,
) -> Mat4d {
    let identity = Mat4d {
        data: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    };

    if joint.is_null() {
        return identity;
    }

    let handle = unsafe { &*joint };
    let transform = handle.joint.parent_link_to_joint_origin();
    let matrix = transform.to_matrix();

    let mut result = Mat4d { data: [0.0; 16] };
    for col in 0..4 {
        for row in 0..4 {
            result.data[col * 4 + row] = matrix[(row, col)];
        }
    }

    result
}

/// Check if a joint position is within limits
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_is_within_limits(
    joint: *const RobotJointHandle,
    position: c_double,
) -> bool {
    if joint.is_null() {
        return false;
    }

    let handle = unsafe { &*joint };
    handle.joint.is_within_limits(position)
}

/// Get joint limits
///
/// # Safety
/// joint pointer must be valid
/// min_limit and max_limit must be valid pointers
#[unsafe(no_mangle)]
pub extern "C" fn robot_joint_get_limits(
    joint: *const RobotJointHandle,
    min_limit: *mut c_double,
    max_limit: *mut c_double,
) {
    if joint.is_null() || min_limit.is_null() || max_limit.is_null() {
        return;
    }

    let handle = unsafe { &*joint };
    let (min, max) = handle.joint.limits();

    unsafe {
        *min_limit = min;
        *max_limit = max;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_create_and_free_joint() {
        let name = CString::new("test_joint").unwrap();
        let joint = robot_joint_new(name.as_ptr());
        assert!(!joint.is_null());

        robot_joint_free(joint);
        // If we reach here without segfaulting, the test passes
    }

    #[test]
    fn test_joint_properties() {
        let name = CString::new("test_joint").unwrap();
        let joint = robot_joint_new(name.as_ptr());
        assert!(!joint.is_null());

        let index = robot_joint_get_index(joint);
        assert_eq!(index, 0);

        let parent_index = robot_joint_get_parent_link_index(joint);
        assert_eq!(parent_index, 0);

        let child_index = robot_joint_get_child_link_index(joint);
        assert_eq!(child_index, 1);

        let dof_index = robot_joint_get_dof_index(joint);
        assert_eq!(dof_index, 0);

        robot_joint_free(joint);
    }

    #[test]
    fn test_calculate_transform() {
        let name = CString::new("transform_test").unwrap();
        let joint = robot_joint_new(name.as_ptr());
        assert!(!joint.is_null());

        let variables = vec![0.0];
        let transform = robot_joint_calculate_transform(joint, variables.as_ptr(), 1);

        // Should be identity matrix for zero rotation
        let expected_identity = [
            1.0, 0.0, 0.0, 0.0, // Column 0
            0.0, 1.0, 0.0, 0.0, // Column 1
            0.0, 0.0, 1.0, 0.0, // Column 2
            0.0, 0.0, 0.0, 1.0, // Column 3
        ];

        for (i, (&actual, &expected)) in transform
            .data
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

        robot_joint_free(joint);
    }

    #[test]
    fn test_joint_limits() {
        let name = CString::new("limits_test").unwrap();
        let joint = robot_joint_new(name.as_ptr());
        assert!(!joint.is_null());

        let mut min_limit = 0.0;
        let mut max_limit = 0.0;
        robot_joint_get_limits(joint, &mut min_limit, &mut max_limit);

        assert_eq!(min_limit, -std::f64::consts::PI);
        assert_eq!(max_limit, std::f64::consts::PI);

        assert!(robot_joint_is_within_limits(joint, 0.0));
        assert!(!robot_joint_is_within_limits(joint, 4.0));

        robot_joint_free(joint);
    }

    #[test]
    fn test_null_safety() {
        // Test that functions handle null pointers gracefully
        assert!(robot_joint_new(std::ptr::null()).is_null());

        let identity = robot_joint_calculate_transform(std::ptr::null(), std::ptr::null(), 0);
        // Should return identity matrix
        assert!((identity.data[0] - 1.0).abs() < 1e-10);
        assert!((identity.data[5] - 1.0).abs() < 1e-10);

        assert_eq!(robot_joint_get_index(std::ptr::null()), 0);
        assert!(!robot_joint_is_within_limits(std::ptr::null(), 0.0));

        // These should not crash
        robot_joint_free(std::ptr::null_mut());
        robot_joint_free_string(std::ptr::null_mut());
    }
}
