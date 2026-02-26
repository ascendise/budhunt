use crate::{
    math::{
        tests::{assert_f32_eq, assert_vec3_eq},
        vector::Vec3,
    },
    vec3,
};
use test_case::test_case;

#[test]
pub fn vec3_should_return_vector_with_all_arguments() {
    // Act
    let vec = vec3!(1.0, 2.0, 3.0);
    // Assert
    assert_vec3_eq(vec, Vec3::new(1.0, 2.0, 3.0));
}

#[test]
pub fn vec3_should_fill_vector_with_argument() {
    // Act
    let vec = vec3!(1.0);
    // Assert
    assert_vec3_eq(vec, Vec3::new(1.0, 1.0, 1.0));
}

#[test_case(
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0)
)]
#[test_case(
    Vec3::new(1.0, 2.0, 3.0),
    Vec3::new(3.0, 2.0, 1.0),
    Vec3::new(-4.0, 8.0, -4.0)
)]
pub fn cross_should_return_cross_product(lhs: Vec3, rhs: Vec3, expected: Vec3) {
    // Arrange
    // Act
    let result = lhs.cross(&rhs);
    // Assert
    assert_vec3_eq(expected, result);
}

#[test_case(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.0)]
#[test_case(Vec3::new(1.0, 2.0, 3.0), Vec3::new(3.0, 2.0, 1.0), 10.0)]
pub fn dot_should_return_dot_product(lhs: Vec3, rhs: Vec3, expected: f32) {
    // Arrange
    // Act
    let result = lhs.dot(&rhs);
    // Assert
    assert_f32_eq(expected, result);
}

#[test_case(Vec3::new(1.0, 0.0, 0.0), 1.0)]
#[test_case(Vec3::new(1.0, 1.0, 1.0), 1.7320508)]
#[test_case(Vec3::new(1.0, 2.0, 3.0), 3.7416575)]
pub fn magnitude_should_return_length(vec: Vec3, expected: f32) {
    // Arrange
    // Act
    let result = vec.magnitude();
    // Assert
    assert_f32_eq(expected, result);
}

#[test_case(Vec3::new(3.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0))]
#[test_case(Vec3::new(3.0, 1.0, 5.0), Vec3::new(0.50709254, 0.16903085, 0.8451542))]
pub fn normalize_should_return_vector_with_magnitude_1(vec: Vec3, expected: Vec3) {
    // Arrange
    // Act
    let result = vec.normalize();
    // Assert
    assert_vec3_eq(expected, result);
}

#[test_case(
    Vec3::new(1.0, 2.0, 3.0),
    Vec3::new(3.0, 2.0, 1.0),
    Vec3::new(4.0, 4.0, 4.0)
)]
#[test_case(
    Vec3::new(1.5, 2.0, 1.0),
    Vec3::new(2.2, -2.0, 9.0),
    Vec3::new(3.7, 0.0, 10.0)
)]
pub fn add_vec3_should_return_sum_of_vectors(lhs: Vec3, rhs: Vec3, expected: Vec3) {
    // Arrange
    // Act
    let result = lhs + rhs;
    // Assert
    assert_vec3_eq(expected, result);
}

#[test_case(Vec3::new(1.0, 2.0, 3.0), 2.0, Vec3::new(3.0, 4.0, 5.0))]
#[test_case(
    Vec3::new(1.5, 2.0, 1.0),
    -1.5,
    Vec3::new(0.0, 0.5, -0.5)
)]
pub fn add_scalar_should_return_transformed_vector(lhs: Vec3, rhs: f32, expected: Vec3) {
    // Arrange
    // Act
    let result = lhs + rhs;
    // Assert
    assert_vec3_eq(expected, result);
}

#[test_case(
    Vec3::new(1.0, 2.0, 3.0),
    Vec3::new(3.0, 2.0, 1.0),
    Vec3::new(-2.0, 0.0, 2.0)
)]
#[test_case(
    Vec3::new(1.5, 2.0, 1.0),
    Vec3::new(2.2, -2.0, 9.0),
    Vec3::new(-0.7, 4.0, -8.0)
)]
pub fn sub_vec3_should_return_distance(lhs: Vec3, rhs: Vec3, expected: Vec3) {
    // Arrange
    // Act
    let result = lhs - rhs;
    // Assert
    assert_vec3_eq(expected, result);
}

#[test_case(
    Vec3::new(1.0, 2.0, 3.0),
    2.0,
    Vec3::new(-1.0, 0.0, 1.0)
)]
#[test_case(
    Vec3::new(1.5, 2.0, 1.0),
    -1.5,
    Vec3::new(3.0, 3.5, 2.5)
)]
pub fn sub_scalar_should_return_transformed_vector(lhs: Vec3, rhs: f32, expected: Vec3) {
    // Arrange
    // Act
    let result = lhs - rhs;
    // Assert
    assert_vec3_eq(expected, result);
}
#[test_case(
    Vec3::new(1.0, 2.0, 3.0),
    Vec3::new(3.0, 2.0, 1.0),
    Vec3::new(3.0, 4.0, 3.0)
)]
#[test_case(
    Vec3::new(1.5, 2.0, 1.0),
    Vec3::new(2.2, -2.0, 9.0),
    Vec3::new(3.3, -4.0, 9.0)
)]
pub fn mul_vec3_should_do_component_wise_multiplication(lhs: Vec3, rhs: Vec3, expected: Vec3) {
    // Arrange
    // Act
    let result = lhs * rhs;
    // Assert
    assert_vec3_eq(expected, result);
}

#[test_case(Vec3::new(1.0, 2.0, 3.0), 2.0, Vec3::new(2.0, 4.0, 6.0))]
#[test_case(
    Vec3::new(1.5, 2.0, 1.0),
    -1.5,
    Vec3::new(-2.25, -3.0, -1.5)
)]
pub fn mul_scalar_should_return_scaled_vector(lhs: Vec3, rhs: f32, expected: Vec3) {
    // Arrange
    // Act
    let result = lhs * rhs;
    // Assert
    assert_vec3_eq(expected, result);
}

#[test_case(Vec3::new(1.0, 2.0, 3.0), 2.0, Vec3::new(0.5, 1.0, 1.5))]
#[test_case(
    Vec3::new(1.5, 2.0, 1.0),
    -2.5,
    Vec3::new(-0.6, -0.8, -0.4)
)]
pub fn div_scalar_should_return_scaled_vector(lhs: Vec3, rhs: f32, expected: Vec3) {
    // Arrange
    // Act
    let result = lhs / rhs;
    // Assert
    assert_vec3_eq(expected, result);
}
