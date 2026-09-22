use crate::{
    math::{self},
    vec3, vec4,
};

#[cfg(test)]
mod tests;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Transform {
    pub position: math::Vec3,
    pub rotation: math::Matrix4,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: vec3!(0.0),
            rotation: math::Matrix4::new(1.0),
        }
    }
}
impl Transform {
    pub fn new(position: math::Vec3) -> Self {
        Self {
            position,
            rotation: math::Matrix4::new(1.0),
        }
    }
    /// Moves the current [Transform::position]
    pub fn translate(&mut self, position: &math::Vec3) {
        self.position += position;
    }
    /// Sets rotation based on view direction of the entity.
    pub fn rotate_fpv(&mut self, direction: &math::Vec3) {
        self.rotation = math::rotation_fpv(direction);
    }

    /// Returns a unit vector rotated by [Transform::rotation]
    pub fn direction(&self) -> math::Vec3 {
        let direction = self.rotation * vec4!(0.0, 0.0, -1.0, 0.0);
        direction.into_vec()
    }

    pub fn model_matrix(&self) -> math::Matrix4 {
        let translation = math::Matrix4::translation(&self.position);
        translation * self.rotation
    }

    /// Returns new set of transformed vertices
    pub fn apply(&self, vertices: &[math::Vec3]) -> Vec<math::Vec3> {
        let model_matrix = self.model_matrix();
        vertices
            .iter()
            .map(|v| model_matrix * vec4!(v, 1.0))
            .map(|v| v.into_vec())
            .collect()
    }

    pub fn apply2(&self, vertices: &[math::Vec3]) -> Vec<math::Vec3> {
        vertices.iter().map(|v| v + self.position).collect()
    }
}
