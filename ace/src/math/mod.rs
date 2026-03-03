mod matrix;
#[cfg(feature = "test_utils")]
pub mod test_utils;
#[cfg(test)]
mod tests;
mod vector;

use std::f32::consts::PI;

pub use matrix::Matrix4;
pub use vector::Vec2;
pub use vector::Vec3;
pub use vector::Vec4;

#[macro_export]
macro_rules! vec2 {
    ($x:expr, $y:expr) => {{ $crate::math::Vec2::new($x, $y) }};
    ($x:expr) => {
        $crate::math::Vec2::new($x, $x)
    };
}

#[macro_export]
macro_rules! vec3 {
    ($x:expr, $y:expr, $z:expr) => {{ $crate::math::Vec3::new($x, $y, $z) }};
    ($x:expr) => {
        $crate::math::Vec3::new($x, $x, $x)
    };
}

#[macro_export]
macro_rules! vec4 {
    ($x:expr, $y:expr, $z:expr, $w:expr) => {{ $crate::math::Vec4::new($x, $y, $z, $w) }};
    ($x:expr) => {
        $crate::math::Vec4::new($x, $x, $x, $x)
    };
}

pub fn projection(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Matrix4 {
    let fov_tan = (fov / 2.0).tan();
    [
        [1.0 / (aspect_ratio * fov_tan), 0.0, 0.0, 0.0],
        [0.0, 1.0 / (fov_tan), 0.0, 0.0],
        [
            0.0,
            0.0,
            (far + near) / (near - far),
            (2.0 * far * near) / (near - far),
        ],
        [0.0, 0.0, -1.0, 0.0],
    ]
    .into()
}

pub fn look_at(eye: &Vec3, center: &Vec3, up: &Vec3) -> Matrix4 {
    let direction = (eye - center).normalize();
    let right = up.cross(&direction).normalize();
    let up = direction.cross(&right);
    let lhs: Matrix4 = [
        [right.x, right.y, right.z, 0.0],
        [up.x, up.y, up.z, 0.0],
        [direction.x, direction.y, direction.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
    .into();
    let mut rhs = Matrix4::new(1.0);
    rhs[0][3] = -eye.x;
    rhs[1][3] = -eye.y;
    rhs[2][3] = -eye.z;
    lhs * rhs
}

pub fn radians(degree: f32) -> f32 {
    PI / 180.0 * degree
}

pub fn rotation(radians: &Vec3) -> Matrix4 {
    let mut rotation = Matrix4::new(1.0);
    rotation = rotation * rotation_x(radians.x);
    rotation = rotation * rotation_y(radians.y);
    rotation = rotation * rotation_z(radians.z);
    rotation
}

fn rotation_x(radians: f32) -> Matrix4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, radians.cos(), radians.sin(), 0.0],
        [0.0, -radians.sin(), radians.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
    .into()
}

fn rotation_y(radians: f32) -> Matrix4 {
    [
        [radians.cos(), 0.0, -radians.sin(), 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [radians.sin(), 0.0, radians.cos(), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
    .into()
}

fn rotation_z(radians: f32) -> Matrix4 {
    [
        [radians.cos(), radians.sin(), 0.0, 0.0],
        [-radians.sin(), radians.cos(), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
    .into()
}

pub fn translate(x: f32, y: f32, z: f32) -> Matrix4 {
    [
        [1.0, 0.0, 0.0, x],
        [0.0, 1.0, 0.0, y],
        [0.0, 0.0, 1.0, z],
        [0.0, 0.0, 0.0, 1.0],
    ]
    .into()
}
