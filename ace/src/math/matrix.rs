use std::ops::{Add, Index, IndexMut, Mul, Sub};

use crate::math::{Vec3, Vec4};
#[derive(Debug, PartialEq, Clone)]
pub struct Matrix4 {
    pub data: [[f32; 4]; 4],
}
impl Matrix4 {
    pub fn new(identity: f32) -> Self {
        [
            [identity, 0.0, 0.0, 0.0],
            [0.0, identity, 0.0, 0.0],
            [0.0, 0.0, identity, 0.0],
            [0.0, 0.0, 0.0, identity],
        ]
        .into()
    }

    pub fn translation(position: &Vec3) -> Self {
        let mut identity = Self::new(1.0);
        identity[0][3] = position.x;
        identity[1][3] = position.y;
        identity[2][3] = position.z;
        identity
    }

    pub fn transpose(&self) -> Matrix4 {
        let mut matrix = Matrix4::new(0.0);
        for (r, row) in self.data.iter().enumerate() {
            for (c, col) in row.iter().enumerate() {
                matrix[c][r] = *col
            }
        }
        matrix
    }

    pub fn inverse(&self) -> Matrix4 {
        let determinant = self.determinant();
        let adjugate = self.adjugate();
        adjugate * (1.0 / determinant)
    }

    fn determinant(&self) -> f32 {
        let sub_matrices = [
            [
                [self[1][1], self[1][2], self[1][3]],
                [self[2][1], self[2][2], self[2][3]],
                [self[3][1], self[3][2], self[3][3]],
            ],
            [
                [self[0][1], self[0][2], self[0][3]],
                [self[2][1], self[2][2], self[2][3]],
                [self[3][1], self[3][2], self[3][3]],
            ],
            [
                [self[0][1], self[0][2], self[0][3]],
                [self[1][1], self[1][2], self[1][3]],
                [self[3][1], self[3][2], self[3][3]],
            ],
            [
                [self[0][1], self[0][2], self[0][3]],
                [self[1][1], self[1][2], self[1][3]],
                [self[2][1], self[2][2], self[2][3]],
            ],
        ];
        (self[0][0] * Self::determinant_3x3(sub_matrices[0]))
            - (self[1][0] * Self::determinant_3x3(sub_matrices[1]))
            + (self[2][0] * Self::determinant_3x3(sub_matrices[2]))
            - (self[3][0] * Self::determinant_3x3(sub_matrices[3]))
    }

    pub fn determinant_3x3(mat: [[f32; 3]; 3]) -> f32 {
        (mat[0][0] * mat[1][1] * mat[2][2])
            + (mat[0][1] * mat[1][2] * mat[2][0])
            + (mat[0][2] * mat[1][0] * mat[2][1])
            - (mat[0][2] * mat[1][1] * mat[2][0])
            - (mat[0][1] * mat[1][0] * mat[2][2])
            - (mat[0][0] * mat[1][2] * mat[2][1])
    }

    fn adjugate(&self) -> Matrix4 {
        let signs = [
            [1.0, -1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0, 1.0],
        ];
        let mut adjugate = Matrix4::new(0.0);
        for r in 0..=3 {
            for c in 0..=3 {
                let submatrix = self.to_3x3_matrix(r, c);
                adjugate[c][r] = signs[c][r] * Self::determinant_3x3(submatrix);
            }
        }
        adjugate
    }

    fn to_3x3_matrix(&self, remove_row: usize, remove_column: usize) -> [[f32; 3]; 3] {
        let mut matrix = [[0f32; 3]; 3];
        for r in 0..matrix.len() {
            let row = if r >= remove_row {
                self[r + 1]
            } else {
                self[r]
            };
            for c in 0..matrix[r].len() {
                let value = if c >= remove_column {
                    row[c + 1]
                } else {
                    row[c]
                };
                matrix[r][c] = value
            }
        }
        matrix
    }
}
impl From<[[f32; 4]; 4]> for Matrix4 {
    fn from(data: [[f32; 4]; 4]) -> Self {
        Self { data }
    }
}

#[allow(clippy::needless_range_loop)]
fn calc_new_matrix<F>(calc_element: F) -> [[f32; 4]; 4]
where
    F: Fn(usize, usize) -> f32,
{
    let mut matrix = [[0f32; 4]; 4];
    for y in 0..=3 {
        for i in 0..=3 {
            matrix[i][y] = calc_element(i, y);
        }
    }
    matrix
}

impl Index<usize> for Matrix4 {
    type Output = [f32; 4];

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}
impl IndexMut<usize> for Matrix4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
impl Add for &Matrix4 {
    type Output = Matrix4;

    fn add(self, rhs: Self) -> Self::Output {
        let matrix = calc_new_matrix(|i, y| self.data[i][y] + rhs.data[i][y]);
        matrix.into()
    }
}
impl Add for Matrix4 {
    type Output = Matrix4;

    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}
impl Sub for &Matrix4 {
    type Output = Matrix4;

    fn sub(self, rhs: Self) -> Self::Output {
        let matrix = calc_new_matrix(|i, y| self.data[i][y] - rhs.data[i][y]);
        matrix.into()
    }
}
impl Sub for Matrix4 {
    type Output = Matrix4;

    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}
impl Mul for &Matrix4 {
    type Output = Matrix4;

    #[allow(clippy::needless_range_loop)]
    fn mul(self, rhs: Self) -> Self::Output {
        let mut matrix = [[0f32; 4]; 4];
        for y in 0..=3 {
            let column = [rhs[0][y], rhs[1][y], rhs[2][y], rhs[3][y]];
            for i in 0..=3 {
                let row = self[i];
                matrix[i][y] = (row[0] * column[0])
                    + (row[1] * column[1])
                    + (row[2] * column[2])
                    + (row[3] * column[3]);
            }
        }
        matrix.into()
    }
}
impl Mul for Matrix4 {
    type Output = Matrix4;

    fn mul(self, rhs: Self) -> Self::Output {
        &self * &rhs
    }
}

impl Mul<&Vec4> for &Matrix4 {
    type Output = Vec4;

    fn mul(self, rhs: &Vec4) -> Self::Output {
        let x_row = self.data[0];
        let y_row = self.data[1];
        let z_row = self.data[2];
        let w_row = self.data[3];
        Vec4 {
            x: (x_row[0] * rhs.x) + (x_row[1] * rhs.y) + (x_row[2] * rhs.z) + (x_row[3] * rhs.w),
            y: (y_row[0] * rhs.x) + (y_row[1] * rhs.y) + (y_row[2] * rhs.z) + (y_row[3] * rhs.w),
            z: (z_row[0] * rhs.x) + (z_row[1] * rhs.y) + (z_row[2] * rhs.z) + (z_row[3] * rhs.w),
            w: (w_row[0] * rhs.x) + (w_row[1] * rhs.y) + (w_row[2] * rhs.z) + (w_row[3] * rhs.w),
        }
    }
}
impl Mul<Vec4> for &Matrix4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        self * &rhs
    }
}
impl Mul<&Vec4> for Matrix4 {
    type Output = Vec4;

    fn mul(self, rhs: &Vec4) -> Self::Output {
        &self * rhs
    }
}
impl Mul<Vec4> for Matrix4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        &self * &rhs
    }
}

impl Mul<f32> for &Matrix4 {
    type Output = Matrix4;

    fn mul(self, rhs: f32) -> Self::Output {
        let matrix = calc_new_matrix(|i, y| self.data[i][y] * rhs);
        matrix.into()
    }
}
impl Mul<f32> for Matrix4 {
    type Output = Matrix4;

    fn mul(self, rhs: f32) -> Self::Output {
        &self * rhs
    }
}
