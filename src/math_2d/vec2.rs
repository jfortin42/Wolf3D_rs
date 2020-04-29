#[derive(Default, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn normalize(&mut self) {
        let n = (self.x * self.x + self.y * self.y).sqrt();

        if n != 0.0 {
            self.x /= n;
            self.y /= n;
        }
    }

    /// counter-clockwise rotation for 2D coordinate system with an inverted y axis
    pub fn orthogonal(&self) -> Self {
        Self { x: self.y, y: -self.x }
    }

    /// dot product
    pub fn dot(&self, other: &Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }
}