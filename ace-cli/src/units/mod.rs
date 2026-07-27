use std::f32::consts::PI;

use ace::{vec2, vec3};

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq)]
pub struct UV {
    pub u: f32,
    pub v: f32,
}
impl UV {
    pub fn from_screen_coordinates(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            u: x as f32 / width as f32,
            v: y as f32 / height as f32,
        }
    }

    pub fn to_screen_coordinates(&self, width: u32, height: u32) -> ace::math::Vec2 {
        let x = (self.u * width as f32).clamp(0.0, (width - 1) as f32);
        let y = (self.v * height as f32).clamp(0.0, (height - 1) as f32);
        vec2!(x, y)
    }

    pub fn to_spheric(&self) -> Spheric {
        let azimuth = (PI / 2.0) - (PI * self.u);
        let inclination = PI * self.v;
        Spheric {
            azimuth,
            inclination,
        }
    }

    pub fn from_spheric(spheric: &Spheric) -> Self {
        let u = (PI / 2.0 - spheric.azimuth) / PI;
        let v = spheric.inclination / PI;
        Self { u, v }
    }
}

#[derive(Debug, PartialEq)]
pub struct Spheric {
    pub azimuth: f32,
    pub inclination: f32,
}
impl Spheric {
    pub fn to_cartesian(&self) -> Cartesian {
        let azimuth = self.azimuth;
        let inclination = self.inclination;
        vec3!(
            inclination.sin() * azimuth.cos(),
            inclination.sin() * azimuth.sin(),
            inclination.cos()
        )
    }

    pub fn from_cartesian(cartesian: &Cartesian) -> Self {
        let radius = cartesian.magnitude();
        let azimuth = f32::atan2(cartesian.y, cartesian.x);
        let inclination = (cartesian.z / radius).acos();
        Self {
            azimuth,
            inclination,
        }
    }
}

pub type Cartesian = ace::math::Vec3;
