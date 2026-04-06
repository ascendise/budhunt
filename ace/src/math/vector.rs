use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::{vec2, vec3, vec4};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self::Output {
        vec2!(-self.x, -self.y)
    }
}
impl Neg for &Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self::Output {
        -self.clone()
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn cross(&self, rhs: &Self) -> Vec3 {
        Vec3 {
            x: (self.y * rhs.z) - (self.z * rhs.y),
            y: (self.z * rhs.x) - (self.x * rhs.z),
            z: (self.x * rhs.y) - (self.y * rhs.x),
        }
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    pub fn normalize(&self) -> Self {
        let magnitude = self.magnitude();
        Vec3 {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
        }
    }

    pub fn magnitude(&self) -> f32 {
        let sum = self.x.powi(2) + self.y.powi(2) + self.z.powi(2);
        sum.sqrt()
    }
}
impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        vec3!(-self.x, -self.y, -self.z)
    }
}
impl Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        -self.clone()
    }
}
impl Add for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}
impl Add<f32> for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}
impl Add<f32> for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: f32) -> Self::Output {
        &self + rhs
    }
}
impl Sub for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}
impl Sub<f32> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}
impl Sub<f32> for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}
impl Mul for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}
impl Mul for Vec3 {
    type Output = Vec3;

    /// Component-wise multiplication
    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}
impl Mul<f32> for &Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}
impl Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        &self * rhs
    }
}
impl Div<f32> for &Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}
impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f32) -> Self::Output {
        &self / rhs
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn magnitude(&self) -> f32 {
        let sum = self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2);
        sum.sqrt()
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z) + (self.w * rhs.w)
    }

    pub fn normalize(&self) -> Vec4 {
        let magnitude = self.magnitude();
        Vec4 {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
            w: self.w / magnitude,
        }
    }
}
impl Neg for Vec4 {
    type Output = Vec4;

    fn neg(self) -> Self::Output {
        vec4!(-self.x, -self.y, -self.z, -self.w)
    }
}
impl Neg for &Vec4 {
    type Output = Vec4;

    fn neg(self) -> Self::Output {
        -self.clone()
    }
}
impl Add for &Vec4 {
    type Output = Vec4;

    fn add(self, rhs: Self) -> Self::Output {
        Vec4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}
impl Add for Vec4 {
    type Output = Vec4;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}
impl Sub for &Vec4 {
    type Output = Vec4;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}
impl Sub for Vec4 {
    type Output = Vec4;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}
impl Mul for &Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec4 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }
}
impl Mul for Vec4 {
    type Output = Vec4;

    /// Component-wise multiplication
    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

impl Add<f32> for &Vec4 {
    type Output = Vec4;

    fn add(self, rhs: f32) -> Self::Output {
        Vec4 {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
            w: self.w + rhs,
        }
    }
}
impl Add<f32> for Vec4 {
    type Output = Vec4;

    fn add(self, rhs: f32) -> Self::Output {
        &self + rhs
    }
}
impl Sub<f32> for &Vec4 {
    type Output = Vec4;

    fn sub(self, rhs: f32) -> Self::Output {
        Vec4 {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
            w: self.w - rhs,
        }
    }
}
impl Sub<f32> for Vec4 {
    type Output = Vec4;

    fn sub(self, rhs: f32) -> Self::Output {
        &self - rhs
    }
}
impl Mul<f32> for &Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec4 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}
impl Mul<f32> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: f32) -> Self::Output {
        &self * rhs
    }
}
impl Div<f32> for &Vec4 {
    type Output = Vec4;

    fn div(self, rhs: f32) -> Self::Output {
        Vec4 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}
impl Div<f32> for Vec4 {
    type Output = Vec4;

    fn div(self, rhs: f32) -> Self::Output {
        &self / rhs
    }
}
