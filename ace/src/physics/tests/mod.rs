use crate::{math, physics::Collider, vec3};

mod collider_tests;
mod collision_system_tests;
mod physics_system_tests;

const CUBE: [math::Vec3; 8] = [
    vec3!(-0.5, -0.5, 0.5),
    vec3!(0.5, -0.5, 0.5),
    vec3!(0.5, 0.5, 0.5),
    vec3!(-0.5, 0.5, 0.5),
    vec3!(-0.5, 0.5, -0.5),
    vec3!(0.5, 0.5, -0.5),
    vec3!(-0.5, -0.5, -0.5),
    vec3!(0.5, -0.5, -0.5),
];
pub fn cube(size: f32) -> Collider {
    cube_at(&vec3!(0.0), size)
}

pub fn cube_at(position: &math::Vec3, size: f32) -> Collider {
    let mut cube = vec![];
    for vertex in CUBE {
        let value = vertex + position * size;
        cube.push(value);
    }
    Collider::new(cube)
}
