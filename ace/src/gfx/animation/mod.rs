use crate::math;

#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    timestamp: f32,
    transform: Transform,
    node: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    translation: math::Vec3,
    rotation: math::Vec3,
    scaling: math::Vec3,
}
