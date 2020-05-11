mod vec2;
mod ray;
mod mat2;
mod geometry;

pub use self::vec2::Vec2;

pub use self::ray::Ray;

pub use self::mat2::Mat2;

pub use self::geometry::PrimitiveGeometry;

pub fn clampf(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}