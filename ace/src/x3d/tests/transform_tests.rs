use crate::{assert_float_eq, math, vec3, x3d::Transform};
use pretty_assertions::assert_eq;

#[test]
pub fn translate_should_move_position() {
    // Arrange
    let mut transform = Transform::new(vec3!(1.0));
    // Act
    transform.translate(&vec3!(2.0));
    // Assert
    assert_float_eq!(Vec3 vec3!(3.0), transform.position);
}

#[test]
pub fn apply_should_translate_vertex() {
    // Arrange
    let transform = Transform::new(vec3!(2.0));
    // Act
    let vertex = vec3!(1.0);
    let transformed = transform.apply(&[vertex]);
    // Assert
    assert_float_eq!(Vec3 vec3!(3.0), transformed[0]);
}

#[test]
pub fn apply_should_rotate_vertex() {
    // Arrange
    let rotate_180 = math::rotation(&vec3!(math::radians(180.0), 0.0, 0.0));
    let transform = Transform {
        position: vec3!(0.0),
        rotation: rotate_180,
    };
    // Act
    let front_vertex = vec3!(0.0, 0.0, -1.0);
    let transformed = transform.apply(&[front_vertex]);
    // Assert
    assert_float_eq!(Vec3 vec3!(0.0, 8.742278e-8, 1.0), transformed[0]); // close enough
}
