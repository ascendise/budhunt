use crate::math;

#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    pub timestamp: f32,
    pub transform: Transform,
    pub node: usize,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Transform {
    pub translation: math::Vec3,
    pub rotation: math::Vec3,
    pub scaling: math::Vec3,
}
