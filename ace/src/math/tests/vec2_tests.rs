use crate::{math::Vec2, vec2};
use pretty_assertions::assert_eq;

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
