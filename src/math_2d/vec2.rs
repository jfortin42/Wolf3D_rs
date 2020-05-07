use serde::Deserialize;

use std::ops::{
    Add,
    AddAssign,
    Sub,
    Mul,
};

#[derive(Deserialize, Default, PartialEq, Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&mut self) {
        let n = self.length();

        if n != 0.0 {
            self.x /= n;
            self.y /= n;
        }
    }

    pub fn scale(&mut self, scale: f32) {
        self.x *= scale;
        self.y *= scale;
    }

    /// 90° rotation for 2D coordinate system with an inverted y axis
    pub fn orthogonal(&self, clockwise: bool) -> Self {
        if clockwise {
            Self { x: -self.y, y: self.x }
        } else {
            Self { x: self.y, y: -self.x }
        }
    }

    /// dot product
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// 2D cross-product returns z coordinate of the resulting vector, which is also the magnitude of the area formed by the two vectors.
    /// If the vectors are collinear or parallel, the result is 0 
    pub fn cross(&self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}