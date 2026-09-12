use crate::{vec2, vec4};
use pretty_assertions::assert_eq;

mod matrix4_tests;
mod vec2_tests;
mod vec3_tests;
mod vec4_tests;

#[test]
pub fn into_vec_should_convert_bigger_vec_to_smaller_vec() {
    // Arrange
    let vec4 = vec4!(1.0, 2.0, 3.0, 4.0);
    // Act
    let vec2 = vec4.into_vec::<2>();
    // Assert
    assert_eq!(vec2!(1.0, 2.0), vec2);
}

#[test]
pub fn into_vec_should_convert_smaller_vec_to_bigger_vec() {
    // Arrange
    let vec2 = vec2!(1.0, 2.0);
    // Act
    let vec4 = vec2.into_vec::<4>();
    // Assert
    assert_eq!(vec4!(1.0, 2.0, 0.0, 0.0), vec4);
}

#[test]
pub fn into_vec_with_should_fill_rest_of_new_vector_with_value() {
    // Arrange
    let vec2 = vec2!(1.0, 2.0);
    // Act
    let vec4 = vec2.into_vec_with::<4>(3.0);
    // Assert
    assert_eq!(vec4!(1.0, 2.0, 3.0, 3.0), vec4);
}
