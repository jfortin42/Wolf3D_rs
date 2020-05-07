use super::Vec2;

#[derive(Default, Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vec2,
    pub dir: Vec2,
}

impl Ray {
    pub fn new(origin: Vec2, dir: Vec2) -> Self {
        Self { origin, dir }
    }
}