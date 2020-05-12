use super::{
    clampf,
    Vec2,
    Ray,
};

use parser;

#[derive(Debug)]
pub enum PrimitiveGeometry {
    Plane {
        p1: Vec2,
        p2: Vec2,
        n: Vec2,
    },
    Cylinder {
        radius: f32,
    },
}

impl PrimitiveGeometry {
    #[inline]
    fn create_one_sided_plane(p1: Vec2, p2: Vec2) -> Self {
        let p1 = Vec2 {
            x: clampf(p1.x, 0.0, 1.0),
            y: clampf(p1.y, 0.0, 1.0)
        };
        let p2 = Vec2 {
            x: clampf(p2.x, 0.0, 1.0),
            y: clampf(p2.y, 0.0, 1.0)
        };
        let mut p1p2 = p2 - p1;
        p1p2.normalize();
        Self::Plane { p1, p2, n: p1p2.orthogonal(false) }
    }

    pub fn new(raw_primitive: parser::geometry::PrimitiveGeometry) -> Self {
        match raw_primitive {
            parser::geometry::PrimitiveGeometry::Plane{ p1, p2 } => {
                Self::create_one_sided_plane(p1, p2)
            },
            parser::geometry::PrimitiveGeometry::Cylinder{ radius } => {
                Self::Cylinder { radius: clampf(radius, 0.0, 0.5) }
            }
        }
    }

    /// ray/one-sided segment intersection
    fn ray_osseg_intersection(ray: Ray, p1: Vec2, p2: Vec2, n: Vec2) -> Option<f32> {
        // wrong side
        if n.dot(ray.dir) >= 0.0 {
            return None;
        }

        // equation of ray is : p + tr
        // with p: ray.origin,
        //      r: ray.dir,
        //      t: scalar parameter

        // equation of segment p1p2 is: q + us
        // with q: p1,
        //      s: vector p2p1 (p2 - p1),
        //      u: scalar parameter

        // we need to find u and t such that:
        // p + tr = q + us
        //      AND
        // 0 <= u <= 1

        // (v x u is cross product of u and v)
        // t = (q − p) × s / (r × s)
        // u = (q − p) × r / (r × s)

        let p2p1 = p2 - p1;
        // r x s
        let rxs = ray.dir.cross(p2p1);

        if rxs != 0.0 {
            // q - p
            let qp = p1 - ray.origin; 
            
            let u = qp.cross(ray.dir) / rxs;

            if u >= 0.0 && u <= 1.0 {
                let t = qp.cross(p2p1) / rxs;
                return Some(t);
            }
        }
        None
    }

    /// this method can return:
    /// * None : there are no intersections
    /// * Some((t0, None)) : there is one intersection (the ray is tangent to the circle)
    /// * Some((t0, Some(t1))) : there are two intersections and t0 <= t1
    fn ray_circle_intersection(ray: Ray, center: Vec2, radius: f32) -> Option<(f32, Option<f32>)> {
        
        let center_orig = ray.origin - center;
        
        // quadratic equation : at^2 + bt + c = 0 
        let a = ray.dir.dot(ray.dir);
        let b = 2.0 * ray.dir.dot(center_orig);
        let c = center_orig.dot(center_orig) - radius * radius;

        // solving:
        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            None
        } else if disc == 0.0 {
            Some((-b / (2.0 * a), None))
        } else {
            let (t0, t1) = {
                let t0 = (-b - disc.sqrt()) / (2.0 * a);
                let t1 = (-b + disc.sqrt()) / (2.0 * a);
                if t0 <= t1 {
                    (t0, t1)
                } else {
                    (t1, t0)
                }
            };
            Some((t0, Some(t1)))
        }
    }

    pub fn ray_intersection(&self, ray: Ray, tile_x: usize, tile_y: usize) -> Option<f32> {
        let mut t: Option<f32> = None;
        match self {
            &Self::Plane{ p1, p2, n } => {
                let tile = Vec2{ x: tile_x as f32, y: tile_y as f32 };
                t = Self::ray_osseg_intersection(ray, p1 + tile, p2 + tile, n);
            },
            &Self::Cylinder{ radius } => {
                let center = Vec2 { x: tile_x as f32 + 0.5, y: tile_y as f32 + 0.5 };
                if let Some((t0, _)) = Self::ray_circle_intersection(ray, center, radius) {
                    t = Some(t0)
                }
            }
        }
        if t.is_some() && t.unwrap() >= 0.0 {
            return t;
        }
        None
    }
}