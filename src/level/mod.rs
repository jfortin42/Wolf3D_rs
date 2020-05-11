use parser;
pub use parser::Error;

use parser::{
    *,
    geometry::GeometryMap,
    material::MaterialMap,
};

use sdl2::render::{
    RenderTarget,
    Canvas
};

use math_2d::{
    Vec2,
    Ray,
};

pub mod material;
pub mod geometry;

pub use self::{
    geometry::GeometrySet,
    material::MaterialSet,
};

use self::geometry::SpawnSymbolSet;

#[derive(Debug)]
pub struct Spawn {
    pub player: usize,
    pub coordinates: Vec2
}

#[derive(Default, Debug)]
pub struct MapTile {
    pub geo_idx: usize,
    pub mtl_idx: usize,
}

#[derive(Debug)]
pub struct Map {
    pub height: usize,
    pub width: usize,
    pub tiles: Vec<Vec<MapTile>>,
}

#[derive(Debug)]
pub struct Level {
    pub geo_set: GeometrySet,
    pub mtl_set: MaterialSet,
    pub spawns: Vec<Spawn>,
    pub map: Map,
}

impl Level {
    const MAX_PLAYERS: usize = 4;

    fn parse_maps(&mut self, geo_map: Vec<String>, mtl_map: Vec<String>, geo_set_symbols: String, mtl_set_symbols: String, spawn_symbol_set: SpawnSymbolSet) -> Result<(), Box<dyn Error>> {
        self.map.height = geo_map.len();
        if self.map.height == 0 {
            return Err("empty geometry map".into());
        }
        if self.map.height != mtl_map.len() {
            return Err("map dimensions mismatch: different number of lines".into());
        }

        self.map.width = geo_map[0].chars().count();
        if self.map.width == 0 {
            return Err("empty geometry line".into());
        }
        for (line, (geo_symbol_line, mtl_symbol_line)) in geo_map.iter().zip(mtl_map).enumerate() {
            if geo_symbol_line.chars().count() != self.map.width {
                return Err(format!("geometry map line {}: different line length", line).into());
            }
            if mtl_symbol_line.chars().count() != self.map.width {
                return Err(format!("material map line {}: different line length", line).into());
            }
            self.map.tiles.push(Vec::with_capacity(geo_symbol_line.chars().count()));
            for (col, (geo_symbol, mtl_symbol)) in geo_symbol_line.chars().zip(mtl_symbol_line.chars()).enumerate() {
                let mut tile = MapTile::default();
                
                if let Some(idx) = geo_set_symbols.chars().position(|symbol| { symbol == geo_symbol }) {
                    tile.geo_idx = idx + 1;
                } else if let Some(idx) = spawn_symbol_set.symbols.chars().position(|symbol| { symbol == geo_symbol }) {
                    self.spawns.push(Spawn {
                        player: spawn_symbol_set.players[idx],
                        coordinates: Vec2 {
                            x: col as f32 + 0.5,
                            y: line as f32 + 0.5,
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

                self.map.tiles[line].push(tile);
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
            map: Map {
                height: 0,
                width: 0,
                tiles: Vec::with_capacity(raw_geo_map.map.len()),
            },
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

    fn negative_step(coord: &mut usize) {
        *coord -= 1;
    }

    fn positive_step(coord: &mut usize) {
        *coord += 1;
    }

    fn negative_on_border(_: &Map, coord: usize) -> bool {
        coord == 0
    }

    fn positive_on_x_border(map: &Map, tile_x: usize) -> bool {
        tile_x == map.width - 1
    }
    
    fn positive_on_y_border(map: &Map, tile_y: usize) -> bool {
        tile_y == map.height - 1
    }

    pub fn draw<T: RenderTarget>(&self, canvas: &mut Canvas<T>, pos: Vec2, dir: Vec2) -> Result<(), Box<dyn Error>> {
        // let fov = 66.0; // = 2 * atan(0.66/1.0) = 66° -> config
        let fov: f32 = 66.0; // -> config
        let half_plane_len = (fov / 2.0).to_radians().tan();// * 2.0; // camera plane length
        let half_plane = dir.orthogonal(true) * half_plane_len; // camera plane vector

        let width: usize = 800; // config
        let height: usize = 600; // config
        'main: for x in 0..width {
            // calculate ray:
            let camera_x = 2.0 * (x as f32) / (width as f32) - 1.0; //x-coordinate in camera space
            let ray_dir = dir + half_plane * camera_x;
            let ray_len = ray_dir.length();
            let mut ray_dir_norm = ray_dir;
            ray_dir_norm.normalize();
            let ray = Ray::new(pos, ray_dir_norm);

            // which tile of the map we're in
            let mut tile_x = pos.x as usize;
            let mut tile_y = pos.y as usize;
            // distances from one x or y-side to next x or y-side
            let delta_dist = Vec2 {
                x: (1.0 / ray_dir.x).abs(),
                y: (1.0 / ray_dir.y).abs()
            };
            // side_dist: distances from current position to next x or y-side
            let mut side_dist = Vec2::default();
            
            // setting step functions and initial side_dist depending on the ray directions
            let (on_x_border, x_step):
                (fn(&Map, usize) -> bool, fn(&mut usize)) = {
                if ray_dir.x < 0.0 {
                    side_dist.x = (pos.x - tile_x as f32) * delta_dist.x;
                    (Self::negative_on_border, Self::negative_step)
                } else {
                    side_dist.x = (tile_x as f32 + 1.0 - pos.x) * delta_dist.x;
                    (Self::positive_on_x_border, Self::positive_step)
                }
            };
            let (on_y_border, y_step):
                (fn(&Map, usize) -> bool, fn(&mut usize)) = {
                if ray_dir.y < 0.0 {
                    side_dist.y = (pos.y - tile_y as f32) * delta_dist.y;
                    (Self::negative_on_border, Self::negative_step)
                } else {
                    side_dist.y = (tile_y as f32 + 1.0 - pos.y) * delta_dist.y;
                    (Self::positive_on_y_border, Self::positive_step)
                }
            };
            
            // wall_dist = perpendicular wall dist
            let wall_dist = loop {
                // jump to next tile, either in x-direction or in y-direction
                if side_dist.x <= side_dist.y && !on_x_border(&self.map, tile_x) {
                    side_dist.x += delta_dist.x;
                    x_step(&mut tile_x);
                } else if side_dist.y < side_dist.x && !on_y_border(&self.map, tile_y) {
                    side_dist.y += delta_dist.y;
                    y_step(&mut tile_y);
                } else {
                    continue 'main;
                }
                
                // check if ray has hit a wall
                let tile_geometries = &self.geo_set.geometries[self.map.tiles[tile_y][tile_x].geo_idx];
                if tile_geometries.is_some() {
                    let mut t_min: Option<f32> = None;
                    for geometry in tile_geometries.as_ref().unwrap() {
                        if let Some(t) = geometry.ray_intersection(ray, tile_x, tile_y) {
                            if t_min.is_none() || (t < t_min.unwrap()) {
                                t_min.replace(t);
                            }
                        }
                    }
                    if t_min.is_some() {
                        break t_min.unwrap() / ray_len;
                    }
                }
            };

            // height of line to draw on screen
            let line_height = ((height as f32 / wall_dist) * 1.3) as usize;

            // calculate lowest and highest pixel to fill in current stripe
            let draw_start = {
                let start = (height as isize - line_height as isize) / 2;
                if start < 0 { 0 } else { start as usize }
            };
            let draw_end = {
                let end = height / 2 + line_height / 2;
                if end >= height { height - 1 } else { end }
            };

            // choose wall color
            let color = sdl2::pixels::Color::RGB(0x24, 0x70, 0x48);

            let p1 = sdl2::rect::Point::new(x as i32, draw_start as i32); // i32 and usize...
            let p2 = sdl2::rect::Point::new(x as i32, draw_end as i32); // i32 and usize...
            // draw the pixels of the stripe as a vertical line
            Self::draw_vertical_line(canvas, p1, p2, color)?;
        }
        Ok(())
    }

    fn draw_vertical_line<T: RenderTarget>(canvas: &mut Canvas<T>, p1: sdl2::rect::Point, p2: sdl2::rect::Point, color: sdl2::pixels::Color) -> Result<(), Box<dyn Error>> {
        canvas.set_draw_color(color);
        Ok(canvas.draw_line(p1, p2)?)
    }
}