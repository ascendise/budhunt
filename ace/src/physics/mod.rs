use crate::math;

#[cfg(test)]
mod tests;

pub struct Collider {
    vertices: Vec<math::Vec3>,
}
impl Collider {
    pub fn intersects(&self, other: &Collider) -> bool {
        todo!()
    }
}
