use super::Vec2;

use std::ops::Mul;

/// 2x2 column-major matrix
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct Mat2 {
    pub m: [f32; 4],
}

impl Mat2 {
    pub fn new() -> Self {
        Self { m: [1.0, 0.0, 0.0, 1.0] }
    }

    pub fn rotation_matrix(angle: f32) -> Self {
        Self { m: [angle.cos(), angle.sin(), -angle.sin(), angle.cos()] }
    }
}

impl Mul<Vec2> for Mat2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: rhs.x * self.m[0] + rhs.y * self.m[2],
            y: rhs.x * self.m[1] + rhs.y * self.m[3]
        }
    }
}