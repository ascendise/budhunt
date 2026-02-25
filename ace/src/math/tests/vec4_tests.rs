use crate::math::{
    Vec4,
    tests::{assert_f32_eq, assert_vec4_eq},
};
use test_case::test_case;

#[test_case(Vec4::new(1.0, 2.0, 3.0, 4.0), 3.0, Vec4::new(4.0, 5.0, 6.0, 7.0))]
#[test_case(
    Vec4::new(4.0, 3.0, 2.0, 1.0),
    -1.5,
    Vec4::new(2.5, 1.5, 0.5, -0.5)
)]
#[test_case(Vec4::new(1.0, 2.5, 0.0, 10.0), 0.0, Vec4::new(1.0, 2.5, 0.0, 10.0))]
pub fn add_scalar_should_return_vec4_sum(left: Vec4, right: f32, expected: Vec4) {
    // Act
    let result = left + right;
    // Assert
    assert_vec4_eq(expected, result);
}

#[test_case(Vec4::new(1.0, 2.0, 3.0, 4.0), 3.0, Vec4::new(-2.0, -1.0, 0.0, 1.0))]
#[test_case(
    Vec4::new(4.0, 3.0, 2.0, 1.0),
    -1.5,
    Vec4::new(5.5, 4.5, 3.5, 2.5)
)]
#[test_case(Vec4::new(1.0, 2.5, 0.0, 10.0), 0.0, Vec4::new(1.0, 2.5, 0.0, 10.0))]
pub fn sub_scalar_should_return_vec4_sum(left: Vec4, right: f32, expected: Vec4) {
    // Act
    let result = left - right;
    // Assert
    assert_vec4_eq(expected, result);
}

#[test_case(Vec4::new(1.0, 2.0, 3.0, 4.0), 3.0, Vec4::new(3.0, 6.0, 9.0, 12.0))]
#[test_case(
    Vec4::new(4.0, 3.0, 2.0, 1.0),
    -1.5,
    Vec4::new(-6.0, -4.5, -3.0, -1.5)
)]
#[test_case(Vec4::new(1.0, 2.5, 0.0, 10.0), 0.0, Vec4::new(0.0, 0.0, 0.0, 0.0))]
pub fn mul_scalar_should_return_vec4_sum(left: Vec4, right: f32, expected: Vec4) {
    // Act
    let result = left * right;
    // Assert
    assert_vec4_eq(expected, result);
}

#[test_case(Vec4::new(1.0, 2.0, 3.0, 4.0), 2.0, Vec4::new(0.5, 1.0, 1.5, 2.0))]
#[test_case(
    Vec4::new(4.0, 3.0, 2.0, 1.0),
    -2.0,
    Vec4::new(-2.0, -1.5, -1.0, -0.5)
)]
#[test_case(Vec4::new(1.0, 2.5, 0.0, 10.0), 0.5, Vec4::new(2.0, 5.0, 0.0, 20.0))]
pub fn div_scalar_should_return_vec4_sum(left: Vec4, right: f32, expected: Vec4) {
    // Act
    let result = left / right;
    // Assert
    assert_vec4_eq(expected, result);
}

#[test_case(
    Vec4::new(1.0, 2.0, 3.0, 4.0),
    Vec4::new(4.0, 3.0, 2.0, 1.0),
    Vec4::new(5.0, 5.0, 5.0, 5.0)
)]
#[test_case(
    Vec4::new(4.0, 3.0, 2.0, 1.0),
    Vec4::new(1.0, 2.0, 3.0, 4.0),
    Vec4::new(5.0, 5.0, 5.0, 5.0)
)]
#[test_case(
    Vec4::new(1.0, 2.5, 0.0, 10.0),
    Vec4::new(-2.0, 0.0, 0.0, 4.0),
    Vec4::new(-1.0, 2.5, 0.0, 14.0)
)]
pub fn add_should_return_vec4_sum(left: Vec4, right: Vec4, expected: Vec4) {
    // Act
    let result = left + right;
    // Assert
    assert_vec4_eq(expected, result);
}

#[test_case(
    Vec4::new(1.0, 2.0, 3.0, 4.0),
    Vec4::new(4.0, 3.0, 2.0, 1.0),
    Vec4::new(-3.0, -1.0, 1.0, 3.0)
)]
#[test_case(
    Vec4::new(4.0, 3.0, 2.0, 1.0),
    Vec4::new(1.0, 2.0, 3.0, 4.0),
    Vec4::new(3.0, 1.0, -1.0, -3.0)
)]
#[test_case(
    Vec4::new(1.0, 2.5, 0.0, 10.0),
    Vec4::new(-2.0, 0.0, 0.0, 4.5),
    Vec4::new(3.0, 2.5, 0.0, 5.5)
)]
pub fn sub_should_subtract_right_vec_from_left(left: Vec4, right: Vec4, expected: Vec4) {
    // Act
    let result = left - right;
    // Assert
    assert_vec4_eq(expected, result);
}

#[test_case(
    Vec4::new(1.0, 2.0, 3.0, 4.0),
    Vec4::new(4.0, 3.0, 2.0, 1.0),
    Vec4::new(4.0, 6.0, 6.0, 4.0)
)]
#[test_case(
    Vec4::new(1.0, 2.5, 0.0, 10.0),
    Vec4::new(-2.0, 0.5, 1.0, 4.5),
    Vec4::new(-2.0, 1.25, 0.0, 45.0)
)]
pub fn mul_should_return_compoent_wise_multiplication(left: Vec4, right: Vec4, expected: Vec4) {
    // Act
    let result = left * right;
    // Assert
    assert_vec4_eq(expected, result);
}

#[test_case(Vec4::new(1.0, 2.0, 3.0, 1.0), Vec4::new(4.0, 3.0, 2.0, 4.0), 20.0)]
#[test_case(Vec4::new(4.0, 3.0, 2.0, 1.0), Vec4::new(1.0, 2.0, 3.0, 4.0), 20.0)]
#[test_case(
    Vec4::new(1.0, 2.5, 0.0, 1.0),
    Vec4::new(-2.0, 1.0, 3.0, 1.0),
    1.5
)]
#[test_case(Vec4::new(1.0, 0.0, 0.0, 0.0), Vec4::new(0.0, 1.0, 0.0, 0.0), 0.0 ; "orthogonal")]
#[test_case(Vec4::new(-1.0, 0.0, 0.0, 0.0), Vec4::new(1.0, 0.0, 0.0, 0.0), -1.0 ; "parallel")]
pub fn dot_should_return_dot_product_of_vectors(left: Vec4, right: Vec4, expected: f32) {
    // Act
    let result = left.dot(&right);
    // Assert
    assert_f32_eq(expected, result);
}

#[test_case(Vec4::new(1.0, 2.0, 3.0, 4.0), 30.0f32.sqrt())]
#[test_case(Vec4::new(4.0, 3.0, 2.0, 1.0), 30.0f32.sqrt())]
#[test_case(Vec4::new(1.0, -2.5, 0.0, 1.0), 2.8722813)]
pub fn magnitude_should_return_length_of_vector(vec: Vec4, expected: f32) {
    // Act
    let result = vec.magnitude();
    // Assert
    assert_f32_eq(expected, result);
}

#[test]
pub fn negate_should_negate_all_elements_of_vector() {
    // Arrange
    let vec = Vec4::new(1.0, 2.0, 3.0, 4.0);
    // Act
    let result = vec.negate();
    // Assert
    let expected = Vec4::new(-1.0, -2.0, -3.0, -4.0);
    assert_vec4_eq(expected, result);
}

#[test_case(Vec4::new(1.0, 1.0, 1.0, 1.0), Vec4::new(0.5, 0.5, 0.5, 0.5))]
#[test_case(Vec4::new(2.0, 4.0, 1.0, 3.0), Vec4::new((2.0f32/15.0f32).sqrt(), 2.0 * (2.0f32/15.0f32).sqrt(), 1.0/30.0f32.sqrt(), (3.0f32/10.0f32).sqrt()))]
#[test_case(Vec4::new(-1.5, 2.0, -2.0, 0.0), Vec4::new(-0.46852127f32, 0.62469506f32, -0.62469506, 0.0))]
pub fn normalize_should_return_vector_with_length_1(vec: Vec4, expected: Vec4) {
    // Arrange
    // Act
    let result = vec.normalize();
    // Assert
    assert_vec4_eq(expected, result);
}
