use super::{
    Deserialize,
    Vec2,
};

use std::ops::Sub;

#[derive(Deserialize, Default, PartialEq, Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Sub for Point {
    type Output = Vec2;

    fn sub(self, other: Self) -> Self::Output {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}