use crate::{assert_float_eq, math::Vec2, vec2};
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test]
pub fn vec2_should_return_vector_with_all_arguments() {
    // Act
    let vec = vec2!(1.0, 2.0);
    // Assert
    assert_eq!(vec, Vec2::new(1.0, 2.0));
}

#[test]
pub fn vec2_should_fill_vector_with_argument() {
    // Act
    let vec = vec2!(1.0);
    // Assert
    assert_eq!(vec, Vec2::new(1.0, 1.0));
}

#[test_case(Vec2::new(1.0, 2.0), Vec2::new(2.0, 2.0), Vec2::new(3.0, 4.0))]
#[test_case(
    Vec2::new(1.5, 2.0),
    Vec2::new(2.2, -2.0),
    Vec2::new(3.7, 0.0)
)]
pub fn add_vec2_should_return_sum_of_vectors(lhs: Vec2, rhs: Vec2, expected: Vec2) {
    // Arrange
    // Act
    let result = lhs + rhs;
    // Assert
    assert_float_eq!(Vec2 expected, result);
}

#[test_case(Vec2::new(1.0, 2.0), 2.0, Vec2::new(3.0, 4.0))]
#[test_case(
    Vec2::new(1.5, 2.0),
    -1.5,
    Vec2::new(0.0, 0.5)
)]
pub fn add_scalar_should_return_transformed_vector(lhs: Vec2, rhs: f32, expected: Vec2) {
    // Arrange
    // Act
    let result = lhs + rhs;
    // Assert
    assert_float_eq!(Vec2 expected, result);
}

#[test_case(
    Vec2::new(1.0, 2.0),
    Vec2::new(2.0, 2.0),
    Vec2::new(-1.0, 0.0)
)]
#[test_case(
    Vec2::new(1.5, 2.0),
    Vec2::new(2.2, -2.0),
    Vec2::new(-0.7, 4.0)
)]
pub fn sub_vec2_should_return_distance(lhs: Vec2, rhs: Vec2, expected: Vec2) {
    // Arrange
    // Act
    let result = lhs - rhs;
    // Assert
    assert_float_eq!(Vec2 expected, result);
}

#[test_case(
    Vec2::new(1.0, 2.0),
    2.0,
    Vec2::new(-1.0, 0.0)
)]
#[test_case(
    Vec2::new(1.5, 2.0),
    -1.5,
    Vec2::new(3.0, 3.5)
)]
pub fn sub_scalar_should_return_transformed_vector(lhs: Vec2, rhs: f32, expected: Vec2) {
    // Arrange
    // Act
    let result = lhs - rhs;
    // Assert
    assert_float_eq!(Vec2 expected, result);
}
#[test_case(Vec2::new(1.0, 2.0), Vec2::new(3.0, 2.0), Vec2::new(3.0, 4.0))]
#[test_case(
    Vec2::new(1.5, 2.0),
    Vec2::new(2.2, -2.0),
    Vec2::new(3.3, -4.0)
)]
pub fn mul_vec3_should_do_component_wise_multiplication(lhs: Vec2, rhs: Vec2, expected: Vec2) {
    // Arrange
    // Act
    let result = lhs * rhs;
    // Assert
    assert_float_eq!(Vec2 expected, result);
}

#[test_case(Vec2::new(1.0, 2.0), 2.0, Vec2::new(2.0, 4.0))]
#[test_case(
    Vec2::new(1.5, 2.0),
    -1.5,
    Vec2::new(-2.25, -3.0)
)]
pub fn mul_scalar_should_return_scaled_vector(lhs: Vec2, rhs: f32, expected: Vec2) {
    // Arrange
    // Act
    let result = lhs * rhs;
    // Assert
    assert_float_eq!(Vec2 expected, result);
}

#[test_case(Vec2::new(1.0, 2.0), 2.0, Vec2::new(0.5, 1.0))]
#[test_case(
    Vec2::new(1.5, 2.0),
    -2.5,
    Vec2::new(-0.6, -0.8)
)]
pub fn div_scalar_should_return_scaled_vector(lhs: Vec2, rhs: f32, expected: Vec2) {
    // Arrange
    // Act
    let result = lhs / rhs;
    // Assert
    assert_float_eq!(Vec2 expected, result);
}
