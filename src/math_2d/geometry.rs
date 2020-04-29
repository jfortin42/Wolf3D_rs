use super::{
    parser,
    Point,
    Vec2,
};

#[derive(Debug)]
pub enum PrimitiveGeometry {
    Plane {
        p1: Point,
        p2: Point,
        n: Vec2,
    },
    Cylinder {
        radius: f32,
    },
}

impl PrimitiveGeometry {
    #[inline]
    fn create_one_sided_plane(p1: Point, p2: Point) -> Self {
        let mut p1p2 = p2 - p1;
        p1p2.normalize();
        Self::Plane { p1, p2, n: p1p2.orthogonal() }
    }

    pub fn new(raw_primitive: parser::geometry::PrimitiveGeometry) -> Self {
        match raw_primitive {
            parser::geometry::PrimitiveGeometry::Plane{ p1, p2 } => {
                Self::create_one_sided_plane(p1, p2)
            },
            parser::geometry::PrimitiveGeometry::Cylinder{ radius } => {
                Self::Cylinder { radius }
            }
        }
    }
}