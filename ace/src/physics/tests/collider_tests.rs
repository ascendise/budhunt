use crate::{physics::Collider, vec3};

#[test]
pub fn intersects_should_return_true_when_shapes_collide() {
    // Arrange
    let plane_horizontal = vec![
        vec3!(1.0, 0.0, 1.0),
        vec3!(1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, 1.0),
    ];
    let plane_horizontal = Collider {
        vertices: plane_horizontal,
    };
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
    let intersects = plane_horizontal.intersects(&plane_vertical);
    // Assert
    assert!(intersects, "Collision between planes not detected!");
}

#[test]
pub fn intersects_should_return_false_when_shapes_do_not_collide() {
    // Arrange
    let plane_horizontal = vec![
        vec3!(1.0, 0.0, 1.0),
        vec3!(1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, -1.0),
        vec3!(-1.0, 0.0, 1.0),
    ];
    let plane_horizontal = Collider {
        vertices: plane_horizontal,
    };
    let plane_horizontal2 = vec![
        vec3!(1.0, 1.0, 1.0),
        vec3!(1.0, 1.0, -1.0),
        vec3!(-1.0, 1.0, -1.0),
        vec3!(-1.0, 1.0, 1.0),
    ];
    let plane_horizontal2 = Collider {
        vertices: plane_horizontal2,
    };
    // Act
    let does_not_intersect = !plane_horizontal.intersects(&plane_horizontal2);
    // Assert
    assert!(does_not_intersect, "False positive on collision detection!");
}
