use crate::{physics::Collider, vec3};

#[test]
pub fn intersects_should_return_true_when_2d_planes_collide() {
    // Arrange
    let plane = vec![
        vec3!(1.0, 1.0, 0.0),
        vec3!(1.0, -1.0, 0.0),
        vec3!(-1.0, -1.0, 0.0),
        vec3!(-1.0, 1.0, 0.0),
    ];
    let plane = Collider { vertices: plane };
    let plane2 = vec![
        vec3!(1.0, 1.0, 0.0),
        vec3!(1.0, 0.0, 0.0),
        vec3!(0.0, 0.0, 0.0),
        vec3!(0.0, 1.0, 0.0),
    ];
    let plane2 = Collider { vertices: plane2 };
    // Act
    let intersects = plane.intersects(&plane2);
    // Assert
    assert!(intersects, "Collision between planes not detected!");
}

#[test]
pub fn intersects_should_return_true_when_3d_planes_collide() {
    // Arrange
    let plane = vec![
        vec3!(1.0, 0.0, 1.0),
        vec3!(1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, 1.0),
    ];
    let plane = Collider { vertices: plane };
    let plane_vertical = vec![
        vec3!(1.0, 1.0, 0.0),
        vec3!(1.0, -1.0, 0.0),
        vec3!(-1.0, -1.0, 0.0),
        vec3!(-1.0, 1.0, 0.0),
    ];
    let plane_vertical = Collider {
        vertices: plane_vertical,
    };
    // Act
    let intersects = plane.intersects(&plane_vertical);
    // Assert
    assert!(intersects, "Collision between planes not detected!");
}

#[test]
pub fn intersects_should_return_false_when_2d_planes_do_not_collide() {
    // Arrange
    let plane = vec![
        vec3!(1.0, 0.0, 1.0),
        vec3!(1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, 1.0),
    ];
    let plane = Collider { vertices: plane };
    let plane2 = vec![
        vec3!(1.0, 1.0, 1.0),
        vec3!(1.0, 1.0, -1.0),
        vec3!(-1.0, 1.0, -1.0),
        vec3!(-1.0, 1.0, 1.0),
    ];
    let plane2 = Collider { vertices: plane2 };
    // Act
    let does_not_intersect = !plane.intersects(&plane2);
    // Assert
    assert!(does_not_intersect, "False positive on collision detection!");
}

#[test]
pub fn intersects_should_return_false_when_3d_planes_do_not_collide() {
    // Arrange
    let plane = vec![
        vec3!(1.0, 1.0, 0.0),
        vec3!(1.0, -1.0, 0.0),
        vec3!(-1.0, -1.0, 0.0),
        vec3!(-1.0, 1.0, 0.0),
    ];
    let plane = Collider { vertices: plane };
    let plane2 = vec![
        vec3!(1.0, 1.0, 1.0),
        vec3!(1.0, -1.0, 1.0),
        vec3!(-1.0, -1.0, 1.0),
        vec3!(-1.0, 1.0, 1.0),
    ];
    let plane2 = Collider { vertices: plane2 };
    // Act
    let does_not_intersect = !plane.intersects(&plane2);
    // Assert
    assert!(does_not_intersect, "False positive on collision detection!");
}
