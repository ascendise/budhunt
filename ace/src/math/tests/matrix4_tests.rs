use crate::assert_float_eq;
use crate::math::{Vec4, matrix::Matrix4};
use pretty_assertions::assert_eq;

#[test]
pub fn new_should_return_identity_matrix() {
    // Arrange
    // Act
    let result = Matrix4::new(1.0);
    // Assert
    let expected: Matrix4 = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
    .into();
    assert_float_eq!(Matrix4 expected, result);
}

#[test]
pub fn transpose_should_flip_matrix_ordering() {
    // Arrange
    let matrix: Matrix4 = [
        [1.1, 1.2, 1.3, 1.4],
        [2.1, 2.2, 2.3, 2.4],
        [3.1, 3.2, 3.3, 3.4],
        [4.1, 4.2, 4.3, 4.4],
    ]
    .into();
    // Act
    let transposed = matrix.transpose();
    // Assert
    let expected: Matrix4 = [
        [1.1, 2.1, 3.1, 4.1],
        [1.2, 2.2, 3.2, 4.2],
        [1.3, 2.3, 3.3, 4.3],
        [1.4, 2.4, 3.4, 4.4],
    ]
    .into();
    assert_float_eq!(Matrix4 expected, transposed);
}

#[test]
pub fn inverse_should_return_inverted_matrix() {
    // Arrange
    let matrix = Matrix4::new(2.0);
    // Act
    let inverse = matrix.inverse();
    // Assert
    let expected = Matrix4::new(1.0) * 0.5;
    assert_float_eq!(Matrix4 expected, inverse);
}

#[test]
pub fn inverse_should_return_inverted_matrix_2() {
    // Arrange
    let matrix: Matrix4 = [
        [1.0, 1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0, 1.0],
        [-1.0, 1.0, 1.0, 1.0],
    ]
    .into();
    // Act
    let inverse = matrix.inverse();
    // Assert
    let expected: Matrix4 = [
        [-4.0, -4.0, -4.0, 4.0],
        [-4.0, -4.0, 4.0, -4.0],
        [-4.0, 4.0, -4.0, -4.0],
        [4.0, -4.0, -4.0, -4.0],
    ]
    .into();
    let expected = expected * -(1.0 / 16.0);
    assert_float_eq!(Matrix4 expected, inverse);
}

#[test]
pub fn inverse_should_return_inverted_matrix_3() {
    // Arrange
    let matrix: Matrix4 = [
        [1.0, 2.0, 2.0, 3.0],
        [2.0, 5.0, -1.0, 2.0],
        [4.0, -1.0, 1.0, 3.0],
        [1.0, 3.0, 2.0, 3.0],
    ]
    .into();
    // Act
    let inverse = matrix.inverse();
    // Assert
    let expected: Matrix4 = [
        [-37.0, -3.0, 7.0, 32.0],
        [-17.0, 0.0, 0.0, 17.0],
        [-43.0, -9.0, 4.0, 45.0],
        [58.0, 7.0, -5.0, -52.0],
    ]
    .into();
    let expected = expected * (1.0 / 17.0);
    assert_float_eq!(Matrix4 expected, inverse);
}

#[test]
pub fn add_should_return_matrix_sum() {
    // Arrange
    let left: Matrix4 = [
        [1.0, 2.0, 3.0, 4.0],
        [2.0, 3.0, 4.0, 1.0],
        [3.0, 4.0, 1.0, 2.0],
        [4.0, 1.0, 2.0, 3.0],
    ]
    .into();
    let right: Matrix4 = [
        [5.0, 6.0, 7.0, 8.0],
        [6.0, 7.0, 8.0, 5.0],
        [7.0, 8.0, 5.0, 6.0],
        [8.0, 5.0, 6.0, 7.0],
    ]
    .into();
    // Act
    let result = left + right;
    // Assert
    let expected: Matrix4 = [
        [6.0, 8.0, 10.0, 12.0],
        [8.0, 10.0, 12.0, 6.0],
        [10.0, 12.0, 6.0, 8.0],
        [12.0, 6.0, 8.0, 10.0],
    ]
    .into();
    assert_float_eq!(Matrix4 expected, result);
}

#[test]
pub fn sub_should_return_subtracted_matrix() {
    // Arrange
    let left: Matrix4 = [
        [1.0, 2.0, 3.0, 4.0],
        [2.0, 3.0, 4.0, 1.0],
        [3.0, 4.0, 1.0, 2.0],
        [4.0, 1.0, 2.0, 3.0],
    ]
    .into();
    let right: Matrix4 = [
        [5.0, 6.0, 7.0, 8.0],
        [6.0, 7.0, 8.0, 5.0],
        [7.0, 8.0, 5.0, 6.0],
        [8.0, 5.0, 6.0, 7.0],
    ]
    .into();
    // Act
    let result = left - right;
    // Assert
    let expected: Matrix4 = [
        [-4.0, -4.0, -4.0, -4.0],
        [-4.0, -4.0, -4.0, -4.0],
        [-4.0, -4.0, -4.0, -4.0],
        [-4.0, -4.0, -4.0, -4.0],
    ]
    .into();
    assert_float_eq!(Matrix4 expected, result);
}

#[test]
pub fn mul_scalar_should_return_matrix_product() {
    // Arrange
    let matrix: Matrix4 = [
        [1.0, 2.0, 3.0, 4.0],
        [2.0, 3.0, 4.0, 1.0],
        [3.0, 4.0, 1.0, 2.0],
        [4.0, 1.0, 2.0, 3.0],
    ]
    .into();
    // Act
    let result = matrix * 0.5;
    // Assert
    let expected: Matrix4 = [
        [0.5, 1.0, 1.5, 2.0],
        [1.0, 1.5, 2.0, 0.5],
        [1.5, 2.0, 0.5, 1.0],
        [2.0, 0.5, 1.0, 1.5],
    ]
    .into();
    assert_float_eq!(Matrix4 expected, result);
}

#[test]
pub fn mul_matrix_should_return_matrix_product() {
    // Arrange
    let left: Matrix4 = [
        [1.0, 2.0, 3.0, 4.0],
        [2.0, 3.0, 4.0, 1.0],
        [3.0, 4.0, 1.0, 2.0],
        [4.0, 1.0, 2.0, 3.0],
    ]
    .into();
    let right: Matrix4 = [
        [5.0, 6.0, 7.0, 8.0],
        [6.0, 7.0, 8.0, 5.0],
        [7.0, 8.0, 5.0, 6.0],
        [8.0, 5.0, 6.0, 7.0],
    ]
    .into();
    // Act
    let result = left * right;
    // Assert
    let expected: Matrix4 = [
        [70.0, 64.0, 62.0, 64.0],
        [64.0, 70.0, 64.0, 62.0],
        [62.0, 64.0, 70.0, 64.0],
        [64.0, 62.0, 64.0, 70.0],
    ]
    .into();
    assert_float_eq!(Matrix4 expected, result);
}

#[test]
pub fn mul_matrix_should_return_matrix_product2() {
    // Arrange
    let left = Matrix4::new(1.0);
    let mut right = Matrix4::new(1.0);
    right[0][3] = -0.0;
    right[1][3] = -0.0;
    right[2][3] = -3.0;
    // Act
    let result = left * right;
    // Assert
    let expected: Matrix4 = [
        [1.0, 0.0, 0.0, -0.0],
        [0.0, 1.0, 0.0, -0.0],
        [0.0, 0.0, 1.0, -3.0],
        [0.0, 0.0, 0.0, -1.0],
    ]
    .into();
    assert_float_eq!(Matrix4 expected, result);
}

#[test]
pub fn mul_vec_should_return_matrix_product() {
    // Arrange
    let left: Matrix4 = [
        [1.0, 2.0, 3.0, 4.0],
        [2.0, 3.0, 4.0, 1.0],
        [3.0, 4.0, 1.0, 2.0],
        [4.0, 1.0, 2.0, 3.0],
    ]
    .into();
    let right = Vec4::new(1.0, 2.0, 3.0, 4.0);
    // Act
    let result = left * right;
    // Assert
    let expected = Vec4::new(30.0, 24.0, 22.0, 24.0);
    assert_float_eq!(Vec4 expected, result);
}
