use std::{
    ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign},
    slice::{Iter, IterMut},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vec<const N: usize> {
    elements: [f32; N],
}
impl<const N: usize> Vec<N> {
    const fn new_n(elements: [f32; N]) -> Self {
        Self { elements }
    }
    pub const fn zero() -> Self {
        Self::new_n([0.0; N])
    }
    pub fn iter(&self) -> Iter<'_, f32> {
        self.elements.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, f32> {
        self.elements.iter_mut()
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        self.iter().enumerate().map(|(i, e)| e * rhs[i]).sum()
    }

    pub fn normalize(&self) -> Self {
        let magnitude = self.magnitude();
        self.iter().map(|e| e / magnitude).collect()
    }

    pub fn magnitude(&self) -> f32 {
        let sum: f32 = self.iter().map(|e| e.powi(2)).sum();
        sum.sqrt()
    }

    pub fn into_vec_with<const M: usize>(self, value: f32) -> Vec<M> {
        let mut new = Vec::<M>::new_n([value; M]);
        for (i, n) in new.iter_mut().take(N).enumerate() {
            *n = self[i];
        }
        new
    }

    pub fn into_vec<const M: usize>(self) -> Vec<M> {
        self.into_vec_with(0.0)
    }
}
impl<const N: usize> Default for Vec<N> {
    fn default() -> Self {
        Self::zero()
    }
}
impl<const N: usize> From<[f32; N]> for Vec<N> {
    fn from(elements: [f32; N]) -> Self {
        Self { elements }
    }
}
impl<const N: usize> From<Vec<N>> for [f32; N] {
    fn from(vec: Vec<N>) -> Self {
        vec.elements
    }
}

impl<const N: usize> IntoIterator for Vec<N> {
    type Item = f32;

    type IntoIter = std::array::IntoIter<f32, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}
impl<const N: usize> IntoIterator for &Vec<N> {
    type Item = f32;

    type IntoIter = std::array::IntoIter<f32, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}
impl<const N: usize> FromIterator<f32> for Vec<N> {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        let mut new_vec = Vec::<N>::zero();
        for (i, value) in iter.into_iter().enumerate() {
            if i >= N {
                break;
            }
            new_vec[i] = value;
        }
        new_vec
    }
}
impl<const N: usize> Index<usize> for Vec<N> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.elements[index]
    }
}
impl<const N: usize> IndexMut<usize> for Vec<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.elements[index]
    }
}

pub type Vec2 = Vec<2>;
impl Vec<2> {
    pub const fn new(x: f32, y: f32) -> Self {
        Self::new_n([x, y])
    }
    pub fn x(&self) -> f32 {
        self[0]
    }
    pub fn mut_x(&mut self) -> &mut f32 {
        &mut self[0]
    }
    pub fn y(&self) -> f32 {
        self[1]
    }
    pub fn mut_y(&mut self) -> &mut f32 {
        &mut self[1]
    }
}
pub type Vec3 = Vec<3>;
impl Vec<3> {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self::new_n([x, y, z])
    }
    pub fn x(&self) -> f32 {
        self[0]
    }
    pub fn mut_x(&mut self) -> &mut f32 {
        &mut self[0]
    }
    pub fn y(&self) -> f32 {
        self[1]
    }
    pub fn mut_y(&mut self) -> &mut f32 {
        &mut self[1]
    }
    pub fn z(&self) -> f32 {
        self[2]
    }
    pub fn mut_z(&mut self) -> &mut f32 {
        &mut self[2]
    }

    pub fn cross(&self, rhs: &Self) -> Vec3 {
        let x = (self.y() * rhs.z()) - (self.z() * rhs.y());
        let y = (self.z() * rhs.x()) - (self.x() * rhs.z());
        let z = (self.x() * rhs.y()) - (self.y() * rhs.x());
        Self::new(x, y, z)
    }
}
pub type Vec4 = Vec<4>;
impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self::new_n([x, y, z, w])
    }
    pub fn x(&self) -> f32 {
        self[0]
    }
    pub fn mut_x(&mut self) -> &mut f32 {
        &mut self[0]
    }
    pub fn y(&self) -> f32 {
        self[1]
    }
    pub fn mut_y(&mut self) -> &mut f32 {
        &mut self[1]
    }
    pub fn z(&self) -> f32 {
        self[2]
    }
    pub fn mut_z(&mut self) -> &mut f32 {
        &mut self[2]
    }
    pub fn w(&self) -> f32 {
        self[3]
    }
    pub fn mut_w(&mut self) -> &mut f32 {
        &mut self[3]
    }
}
impl<const N: usize> Neg for Vec<N> {
    type Output = Vec<N>;

    fn neg(self) -> Self::Output {
        (&self).neg()
    }
}
impl<const N: usize> Neg for &Vec<N> {
    type Output = Vec<N>;

    fn neg(self) -> Self::Output {
        let mut new_vec = Vec::<N>::zero();
        for (i, e) in self.iter().enumerate() {
            new_vec[i] = -e;
        }
        new_vec
    }
}
impl<const N: usize> AddAssign<&Vec<N>> for Vec<N> {
    fn add_assign(&mut self, rhs: &Vec<N>) {
        for (i, e) in rhs.iter().enumerate() {
            self[i] += e;
        }
    }
}
impl<const N: usize> AddAssign for Vec<N> {
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign(&rhs);
    }
}
impl<const N: usize> Add for &Vec<N> {
    type Output = Vec<N>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut vec = *self;
        vec += rhs;
        vec
    }
}
impl<const N: usize> Add<&Vec<N>> for Vec<N> {
    type Output = Vec<N>;

    fn add(self, rhs: &Vec<N>) -> Self::Output {
        (&self).add(rhs)
    }
}
impl<const N: usize> Add<Vec<N>> for &Vec<N> {
    type Output = Vec<N>;

    fn add(self, rhs: Vec<N>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<const N: usize> Add for Vec<N> {
    type Output = Vec<N>;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(&rhs)
    }
}
impl<const N: usize> AddAssign<f32> for Vec<N> {
    fn add_assign(&mut self, rhs: f32) {
        for e in &mut self.elements {
            *e += rhs;
        }
    }
}
impl<const N: usize> Add<f32> for &Vec<N> {
    type Output = Vec<N>;

    fn add(self, rhs: f32) -> Self::Output {
        let mut vec = *self;
        vec += rhs;
        vec
    }
}
impl<const N: usize> Add<f32> for Vec<N> {
    type Output = Vec<N>;

    fn add(self, rhs: f32) -> Self::Output {
        (&self).add(rhs)
    }
}

impl<const N: usize> SubAssign<&Vec<N>> for Vec<N> {
    fn sub_assign(&mut self, rhs: &Vec<N>) {
        for (i, e) in rhs.iter().enumerate() {
            self[i] -= e;
        }
    }
}
impl<const N: usize> SubAssign for Vec<N> {
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign(&rhs);
    }
}
impl<const N: usize> Sub for &Vec<N> {
    type Output = Vec<N>;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut vec = *self;
        vec -= rhs;
        vec
    }
}
impl<const N: usize> Sub<Vec<N>> for &Vec<N> {
    type Output = Vec<N>;

    fn sub(self, rhs: Vec<N>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<const N: usize> Sub for Vec<N> {
    type Output = Vec<N>;

    fn sub(self, rhs: Self) -> Self::Output {
        (&self).sub(&rhs)
    }
}
impl<const N: usize> Sub<&Vec<N>> for Vec<N> {
    type Output = Vec<N>;

    fn sub(self, rhs: &Vec<N>) -> Self::Output {
        (&self).sub(rhs)
    }
}
impl<const N: usize> SubAssign<f32> for Vec<N> {
    fn sub_assign(&mut self, rhs: f32) {
        for e in &mut self.elements {
            *e -= rhs;
        }
    }
}
impl<const N: usize> Sub<f32> for &Vec<N> {
    type Output = Vec<N>;

    fn sub(self, rhs: f32) -> Self::Output {
        let mut vec = *self;
        vec -= rhs;
        vec
    }
}
impl<const N: usize> Sub<f32> for Vec<N> {
    type Output = Vec<N>;

    fn sub(self, rhs: f32) -> Self::Output {
        (&self).sub(rhs)
    }
}

impl<const N: usize> MulAssign<&Vec<N>> for Vec<N> {
    fn mul_assign(&mut self, rhs: &Vec<N>) {
        for (i, e) in rhs.iter().enumerate() {
            self[i] *= e;
        }
    }
}
impl<const N: usize> Mul for &Vec<N> {
    type Output = Vec<N>;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut vec = *self;
        vec *= rhs;
        vec
    }
}
impl<const N: usize> Mul<Vec<N>> for &Vec<N> {
    type Output = Vec<N>;

    fn mul(self, rhs: Vec<N>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<const N: usize> Mul<&Vec<N>> for Vec<N> {
    type Output = Vec<N>;

    fn mul(self, rhs: &Vec<N>) -> Self::Output {
        (&self).mul(rhs)
    }
}
impl<const N: usize> Mul for Vec<N> {
    type Output = Vec<N>;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<const N: usize> MulAssign<f32> for Vec<N> {
    fn mul_assign(&mut self, rhs: f32) {
        for e in &mut self.elements {
            *e *= rhs;
        }
    }
}
impl<const N: usize> Mul<f32> for &Vec<N> {
    type Output = Vec<N>;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut vec = *self;
        vec *= rhs;
        vec
    }
}
impl<const N: usize> Mul<f32> for Vec<N> {
    type Output = Vec<N>;

    fn mul(self, rhs: f32) -> Self::Output {
        (&self).mul(rhs)
    }
}

impl<const N: usize> DivAssign<f32> for Vec<N> {
    fn div_assign(&mut self, rhs: f32) {
        for e in &mut self.elements {
            *e /= rhs;
        }
    }
}
impl<const N: usize> Div<f32> for &Vec<N> {
    type Output = Vec<N>;

    fn div(self, rhs: f32) -> Self::Output {
        let mut vec = *self;
        vec /= rhs;
        vec
    }
}
impl<const N: usize> Div<f32> for Vec<N> {
    type Output = Vec<N>;

    fn div(self, rhs: f32) -> Self::Output {
        (&self).div(rhs)
    }
}
