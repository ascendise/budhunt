use std::vec;

use crate::{
    physics::{
        Collider,
        tests::{cube, cube_at},
    },
    vec3,
};

#[test]
pub fn intersects_should_return_true_when_two_points_collide() {
    // Arrange
    let point1 = Collider::new(vec![vec3!(0.0, 1.0, 0.0)]);
    let point2 = Collider::new(vec![vec3!(0.0, 1.0, 0.0)]);
    // Act
    let intersects = point1.intersects(&point2);
    // Assert
    assert!(
        intersects,
        "Collision between intersecting points was not detected"
    );
}

#[test]
pub fn intersects_should_return_false_when_two_points_do_not_collide() {
    // Arrange
    let point1 = Collider::new(vec![vec3!(0.0, 0.0, 0.0)]);
    let point2 = Collider::new(vec![vec3!(1.0, 1.0, 1.0)]);
    // Act
    let does_not_intersect = !point1.intersects(&point2);
    // Assert
    assert!(does_not_intersect, "False positive on collision detection");
}

#[test]
pub fn intersects_should_return_true_when_two_lines_collide() {
    // Arrange
    let line1 = Collider::line(vec3!(0.0, -0.5, 0.0), &vec3!(0.0, 1.0, 0.0));
    let line2 = Collider::line(vec3!(-0.5, 0.0, 0.0), &vec3!(0.5, 0.0, 0.0));
    // Act
    let intersects = line1.intersects(&line2);
    // Assert
    assert!(
        intersects,
        "Collision between intersecting lines was not detected"
    );
}

#[test]
pub fn intersects_should_return_false_when_two_lines_do_not_collide() {
    // Arrange
    let line1 = Collider::line(vec3!(0.0, -0.5, 0.0), &vec3!(0.0, 1.0, 0.0));
    let line2 = Collider::line(vec3!(0.1, -0.5, 0.0), &vec3!(0.1, 1.0, 0.0));
    // Act
    let does_not_intersect = !line1.intersects(&line2);
    // Assert
    assert!(does_not_intersect, "False positive between lines");
}

#[test]
pub fn intersects_should_return_true_when_line_collides_with_cube() {
    // Arrange
    let cube = cube_at(&vec3!(0.0, 0.0, 10.0), 1.0);
    let line = Collider {
        vertices: vec![vec3!(0.0, 0.5, 0.0), vec3!(0.0, 0.5, 20.0)],
    };
    // Act
    let intersects = cube.intersects(&line);
    // Assert
    assert!(
        intersects,
        "Collision between cube and line was not detected"
    );
}

#[test]
pub fn intersects_should_return_false_when_line_does_not_collide_with_cube() {
    // Arrange
    let cube = cube_at(&vec3!(0.0, 2.0, 10.0), 1.0);
    let line = Collider {
        vertices: vec![vec3!(0.0, 0.5, 0.0), vec3!(0.0, 0.5, 20.0)], // going under the cube
    };
    // Act
    let does_not_intersect = !cube.intersects(&line);
    // Assert
    assert!(does_not_intersect, "False positive between cube and line",);
}

#[test]
pub fn intersects_should_return_true_when_cubes_collide() {
    // Arrange
    let cube = cube(1.0);
    let other_cube = cube_at(&vec3!(0.9, 0.0, 0.0), 1.0);
    // Act
    let intersects = cube.intersects(&other_cube);
    // Assert
    assert!(intersects, "Collision between cubes was not detected");
}

#[test]
pub fn intersects_should_return_false_when_cubes_do_not_collide() {
    // Arrange
    let cube = cube(1.0);
    let other_cube = cube_at(&vec3!(1.1, 0.0, 0.0), 1.0);
    // Act
    let does_not_intersect = !cube.intersects(&other_cube);
    // Assert
    assert!(does_not_intersect, "False positive between cubes");
}
