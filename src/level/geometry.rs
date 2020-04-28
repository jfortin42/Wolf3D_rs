use super::*;

use std::ops::Sub;

pub use parser::geometry::Point;

#[derive(Debug)]
pub struct SpawnSymbolSet {
    pub symbols: String,
    pub players: Vec<usize>
}

impl SpawnSymbolSet {
    pub fn new(set_name: &str) -> Result<Self, Box<dyn Error>> {
        let reader = read_assets_file(&["spawn_symbol_sets"], set_name)?;
        let raw_set: parser::geometry::SpawnSymbolSet = serde_json::from_reader(reader)?;

        let n = raw_set.spawn_symbol_set.len();
        let mut set = Self {
            symbols: String::with_capacity(n),
            players: Vec::with_capacity(n),
        };

        for spawn_symbol in raw_set.spawn_symbol_set {
            set.symbols.push(spawn_symbol.symbol);
            set.players.push(spawn_symbol.player);
        }
        Ok(set)
    }
}


#[derive(Default, Debug)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
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

impl Sub for Point {
    type Output = Vec2;

    fn sub(self, other: Self) -> Self::Output {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

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

#[derive(Debug)]
pub struct GeometrySet {
    pub names: Vec<String>,
    pub geometries: Vec<Option<Vec<PrimitiveGeometry>>>,
}

impl GeometrySet {
    pub fn new(set_name: &str) -> Result<(Self, String), Box<dyn Error>> {
        let reader = read_assets_file(&["geometry_sets"], set_name)?;
        let raw_set: parser::geometry::GeometrySet = serde_json::from_reader(reader)?;

        let n = raw_set.geometry_set.len();
        let mut set = Self {
            names: Vec::with_capacity(n + 1),
            geometries: Vec::with_capacity(n + 1),
        };
        let mut symbols = String::with_capacity(n);

        // empty geometry at index 0
        set.names.push(String::from("empty"));
        set.geometries.push(None);

        for geometry in raw_set.geometry_set {
            symbols.push(geometry.symbol);
            set.names.push(geometry.name);
            let primitives = {
                if geometry.primitives.is_some() {
                    Some(geometry.primitives.unwrap().into_iter().map(|raw_prim| { PrimitiveGeometry::new(raw_prim) }).collect())
                } else {
                    None
                }
            };
            set.geometries.push(primitives);
        }
        Ok((set, symbols))
    }
}
