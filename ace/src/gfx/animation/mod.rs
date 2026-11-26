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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Transform {
    Translation(math::Vec3),
    Rotation(math::Vec4), //TODO: actually Quaternions
    Scale(math::Vec3),
}

//#[derive(Debug, Clone, PartialEq)]
//pub struct Transform {
//    translation: math::Vec3,
//    rotation: math::Vec3,
//    scaling: math::Vec3,
//}
