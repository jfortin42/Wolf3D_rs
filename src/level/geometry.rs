use super::*;

use math_2d::PrimitiveGeometry;

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
