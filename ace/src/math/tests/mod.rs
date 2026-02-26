use crate::math::{Vec4, matrix::Matrix4, vector::Vec3};
use pretty_assertions::assert_eq;

mod matrix4_tests;
mod vec2_tests;
mod vec3_tests;
mod vec4_tests;

pub fn assert_matrix4_eq(expected: Matrix4, actual: Matrix4) {
    let mut not_equal = false;
    for i in 0..=3 {
        for y in 0..=3 {
            if !float_is_near(expected.data[i][y], actual.data[i][y], f32::EPSILON) {
                not_equal = true;
                break;
            }
        }
    }
    if not_equal {
        assert_eq!(expected, actual, "Matrix4 do not match!");
    }
}

pub fn assert_vec4_eq(expected: Vec4, actual: Vec4) {
    if !(float_is_near(expected.x, actual.x, f32::EPSILON)
        && float_is_near(expected.y, actual.y, f32::EPSILON)
        && float_is_near(expected.z, actual.z, f32::EPSILON)
        && float_is_near(expected.w, actual.w, f32::EPSILON))
    {
        assert_eq!(expected, actual, "Vec4 do not match!");
    }
}
pub fn assert_vec3_eq(expected: Vec3, actual: Vec3) {
    if !(float_is_near(expected.x, actual.x, f32::EPSILON)
        && float_is_near(expected.y, actual.y, f32::EPSILON)
        && float_is_near(expected.z, actual.z, f32::EPSILON))
    {
        assert_eq!(expected, actual, "Vec3 do not match!");
    }
}

pub fn assert_f32_eq(expected: f32, actual: f32) {
    if !float_is_near(expected, actual, f32::EPSILON) {
        assert_eq!(expected, actual)
    }
}

// https://web.archive.org/web/20251227112838/https://floating-point-gui.de/errors/comparison/
fn float_is_near(expected: f32, actual: f32, epsilon: f32) -> bool {
    if expected == actual {
        return true;
    }
    let expected_abs = expected.abs();
    let actual_abs = actual.abs();
    let diff = (expected_abs - actual_abs).abs();
    if expected == 0.0 || actual == 0.0 || expected_abs + actual_abs < f32::MIN_POSITIVE {
        diff < epsilon * f32::MIN_POSITIVE
    } else {
        diff / f32::min(expected_abs + actual_abs, f32::MAX) < epsilon
    }
}
