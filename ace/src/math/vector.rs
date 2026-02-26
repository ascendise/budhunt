use std::ops::{Add, Div, Mul, Sub};

//TODO: Macros for easier creation of vectors
// E.g. vecn[1.0, 2.0, 3.0] => Vec3
// vec2[1.0] => Vec2 { 1.0, 1.0 }

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
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
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn magnitude(&self) -> f32 {
        let sum = self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2);
        sum.sqrt()
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z) + (self.w * rhs.w)
    }

    pub fn negate(&self) -> Vec4 {
        self * -1.0
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
