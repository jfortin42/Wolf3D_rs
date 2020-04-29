use super::parser;
pub use parser::Error;

use parser::{
    *,
    geometry::GeometryMap,
    material::MaterialMap,
};

pub mod material;
pub mod geometry;

pub use self::{
    geometry::GeometrySet,
    material::MaterialSet,
};

use self::geometry::{
    Point,
    SpawnSymbolSet,
};

#[derive(Debug)]
pub struct Spawn {
    pub player: usize,
    pub coordinates: Point
}

#[derive(Default, Debug)]
pub struct MapTile {
    pub geo_idx: usize,
    pub mtl_idx: usize,
}

#[derive(Debug)]
pub struct Level {
    pub geo_set: GeometrySet,
    pub mtl_set: MaterialSet,
    pub spawns: Vec<Spawn>,
    pub map: Vec<Vec<MapTile>>,
}

impl Level {
    const MAX_PLAYERS: usize = 4;

    fn parse_maps(&mut self, geo_map: Vec<String>, mtl_map: Vec<String>, geo_set_symbols: String, mtl_set_symbols: String, spawn_symbol_set: SpawnSymbolSet) -> Result<(), Box<dyn Error>> {
        if geo_map.len() != mtl_map.len() {
            return Err("map dimensions mismatch: different number of lines".into());
        }

        for (line, (geo_symbol_line, mtl_symbol_line)) in geo_map.iter().zip(mtl_map).enumerate() {
            self.map.push(Vec::with_capacity(geo_symbol_line.chars().count()));
            if geo_symbol_line.chars().count() != mtl_symbol_line.chars().count() {
                return Err("map dimensions mismatch: different line lengths".into());
            }
            for (col, (geo_symbol, mtl_symbol)) in geo_symbol_line.chars().zip(mtl_symbol_line.chars()).enumerate() {
                let mut tile: MapTile = Default::default();
                
                if let Some(idx) = geo_set_symbols.chars().position(|symbol| { symbol == geo_symbol }) {
                    tile.geo_idx = idx + 1;
                } else if let Some(idx) = spawn_symbol_set.symbols.chars().position(|symbol| { symbol == geo_symbol }) {
                    self.spawns.push(Spawn {
                        player: spawn_symbol_set.players[idx],
                        coordinates: Point {
                            x: line as f32 + 0.5,
                            y: col as f32 + 0.5,
                        },
                    });
                    tile.geo_idx = 0; // "empty" geometry
                } else {
                    return Err(format!("unknown geometry/spawn symbol '{}' at tile {}:{}", geo_symbol, line, col).into());
                }

                if let Some(idx) = mtl_set_symbols.chars().position(|symbol| { symbol == mtl_symbol }) {
                    if self.mtl_set.material_properties[idx].portal.is_some() && self.geo_set.geometries[tile.geo_idx].is_none() {
                        return Err(format!("Tile {}:{} : A portal must be associated to a non-empty geometric tile", line, col).into());
                    }
                    tile.mtl_idx = idx;
                } else {
                    return Err(format!("unknown material/portal symbol '{}' at tile {}:{}", mtl_symbol, line, col).into());
                }

                self.map[line].push(tile);
            }
        }
        Ok(())
    }

    pub fn new<P: AsRef<Path>>(level_name: P) -> Result<Level, Box<dyn Error>> {
        let dirs = [&Path::new("maps"), level_name.as_ref()];

        let geo_map_reader = read_assets_file(&dirs, "geometry_map")?;
        let raw_geo_map: GeometryMap = serde_json::from_reader(geo_map_reader)?;

        let (geo_set, geo_set_symbols) = GeometrySet::new(&raw_geo_map.geometry_set)?;
        let spawn_symbol_set = SpawnSymbolSet::new(&raw_geo_map.spawn_symbol_set)?;

        let mtl_map_reader = read_assets_file(&dirs, "material_map")?;
        let raw_mtl_map: MaterialMap = serde_json::from_reader(mtl_map_reader)?;

        let (mtl_set, mtl_set_symbols) = MaterialSet::new(&raw_mtl_map)?;

        let mut level = Self {
            geo_set,
            mtl_set,
            spawns: Vec::with_capacity(Self::MAX_PLAYERS),
            map: Vec::with_capacity(raw_geo_map.map.len()),
        };
        if let Some(symbol) = check_for_duplicate_symbols(&format!("{}{}", geo_set_symbols, spawn_symbol_set.symbols)) {
            return Err(format!("duplicate geometry/spawn symbol '{}'", symbol).into());
        }
        if let Some(symbol) = check_for_duplicate_symbols(&mtl_set_symbols) {
            return Err(format!("duplicate material/portal symbol '{}'", symbol).into());
        }

        level.parse_maps(raw_geo_map.map, raw_mtl_map.map, geo_set_symbols, mtl_set_symbols, spawn_symbol_set)?;
        Ok(level)
    }
}