use super::*;

#[derive(Deserialize, Debug)]
pub struct SpawnSymbol {
    pub player: usize,
    pub symbol: char,
}

#[derive(Deserialize, Debug)]
pub struct SpawnSymbolSet {
    pub spawn_symbol_set: Vec<SpawnSymbol>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PrimitiveGeometry {
    Plane { p1: Vec2, p2: Vec2 },
    Cylinder { radius: f32 },
}

#[derive(Deserialize, Debug)]
pub struct Geometry {
    pub name: String,
    pub symbol: char,
    pub primitives: Option<Vec<PrimitiveGeometry>>,
}

#[derive(Deserialize, Debug)]
pub struct GeometrySet {
    pub geometry_set: Vec<Geometry>,
}

#[derive(Deserialize, Debug)]
pub struct GeometryMap {
    pub geometry_set: String,
    pub spawn_symbol_set: String,
    pub map: Vec<String>,
}