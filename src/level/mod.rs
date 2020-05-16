use sdl2::{
    video::Window,
    render::Canvas,
};

use parser;
pub use parser::Error;

use parser::{
    *,
    geometry::GeometryMap,
    material::MaterialMap,
};

use math_2d::{
    Vec2,
    Ray,
};

use skybox::*;

pub mod material;
pub mod geometry;

pub use self::{
    geometry::GeometrySet,
    material::MaterialSet,
};

use self::geometry::SpawnSymbolSet;

pub struct Spawn {
    pub player: usize,
    pub coordinates: Vec2
}

struct MapData<'a> {
    geo_map: Vec<String>,
    mtl_map: Vec<String>,
    geo_set: &'a GeometrySet,
    mtl_set: &'a MaterialSet,
    geo_set_symbols: String,
    mtl_set_symbols: String,
    spawn_symbol_set: SpawnSymbolSet,
}

#[derive(Debug, Default)]
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

impl Map {
    fn new(data: MapData) -> Result<(Self, Vec<Spawn>), Box<dyn Error>> {
        if let Some(symbol) = check_for_duplicate_symbols(&format!("{}{}", data.geo_set_symbols, data.spawn_symbol_set.symbols)) {
            return Err(format!("duplicate geometry/spawn symbol '{}'", symbol).into());
        }
        if let Some(symbol) = check_for_duplicate_symbols(&data.mtl_set_symbols) {
            return Err(format!("duplicate material/portal symbol '{}'", symbol).into());
        }

        let height = data.geo_map.len();
        if height == 0 {
            return Err("empty geometry map".into());
        }
        if height != data.mtl_map.len() {
            return Err("map dimensions mismatch: different number of lines".into());
        }

        let width = data.geo_map[0].chars().count();
        if width == 0 {
            return Err("empty geometry line".into());
        }

        let mut tiles = Vec::with_capacity(height);
        let mut spawns = Vec::with_capacity(Level::MAX_PLAYERS);
        for (line, (geo_symbol_line, mtl_symbol_line)) in data.geo_map.iter().zip(data.mtl_map).enumerate() {
            if geo_symbol_line.chars().count() != width {
                return Err(format!("geometry map line {}: different line length", line).into());
            }
            if mtl_symbol_line.chars().count() != width {
                return Err(format!("material map line {}: different line length", line).into());
            }
            tiles.push(Vec::with_capacity(geo_symbol_line.chars().count()));
            for (col, (geo_symbol, mtl_symbol)) in geo_symbol_line.chars().zip(mtl_symbol_line.chars()).enumerate() {
                let mut tile = MapTile::default();
                
                if let Some(idx) = data.geo_set_symbols.chars().position(|symbol| { symbol == geo_symbol }) {
                    tile.geo_idx = idx + 1;
                } else if let Some(idx) = data.spawn_symbol_set.symbols.chars().position(|symbol| { symbol == geo_symbol }) {
                    spawns.push(Spawn {
                        player: data.spawn_symbol_set.players[idx],
                        coordinates: Vec2 {
                            x: col as f32 + 0.5,
                            y: line as f32 + 0.5,
                        },
                    });
                    tile.geo_idx = 0; // "empty" geometry
                } else {
                    return Err(format!("unknown geometry/spawn symbol '{}' at tile {}:{}", geo_symbol, line, col).into());
                }

                if let Some(idx) = data.mtl_set_symbols.chars().position(|symbol| { symbol == mtl_symbol }) {
                    if data.mtl_set.material_properties[idx].portal.is_some() && data.geo_set.geometries[tile.geo_idx].is_none() {
                        return Err(format!("Tile {}:{} : A portal must be associated to a non-empty geometric tile", line, col).into());
                    }
                    tile.mtl_idx = idx;
                } else {
                    return Err(format!("unknown material/portal symbol '{}' at tile {}:{}", mtl_symbol, line, col).into());
                }

                tiles[line].push(tile);
            }
        }
        Ok((Self { height, width, tiles }, spawns))
    }
}

pub struct Level<'a> {
    pub geo_set: GeometrySet,
    pub mtl_set: MaterialSet,
    pub spawns: Vec<Spawn>,
    skybox: Option<Skybox<'a>>,
    pub map: Map,
}

impl<'a> Level<'a> {
    const MAX_PLAYERS: usize = 4;

    pub fn new<P: AsRef<Path>>(level_name: P, tex_creator: &'a TextureCreator<WindowContext>) -> Result<Self, Box<dyn Error>> {
        let dirs = [&Path::new("maps"), level_name.as_ref()];

        let geo_map_reader = read_assets_file(&dirs, "geometry_map")?;
        let raw_geo_map: GeometryMap = serde_json::from_reader(geo_map_reader)?;

        let (geo_set, geo_set_symbols) = GeometrySet::new(&raw_geo_map.geometry_set)?;
        let spawn_symbol_set = SpawnSymbolSet::new(&raw_geo_map.spawn_symbol_set)?;

        let mtl_map_reader = read_assets_file(&dirs, "material_map")?;
        let raw_mtl_map: MaterialMap = serde_json::from_reader(mtl_map_reader)?;

        let (mtl_set, mtl_set_symbols) = MaterialSet::new(&raw_mtl_map)?;

        let (map, spawns) = Map::new(MapData {
            geo_map: raw_geo_map.map,
            mtl_map: raw_mtl_map.map,
            geo_set: &geo_set,
            geo_set_symbols,
            mtl_set: &mtl_set,
            mtl_set_symbols,
            spawn_symbol_set,
        })?;
        let skybox = if raw_mtl_map.skybox.is_some() {
                Some(Skybox::new(raw_mtl_map.skybox.unwrap(), tex_creator)?)
            } else {
                None
            };
        Ok(Self { geo_set, mtl_set, spawns, skybox, map })
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

    fn get_wall_dist(&self, mut ray: Ray) -> Option<f32> {
        let ray_len = ray.dir.length();
        ray.dir.normalize();

        // which tile of the map we're in
        let mut tile_x = ray.origin.x as usize;
        let mut tile_y = ray.origin.y as usize;
        // distances from one x or y-side to next x or y-side
        let delta_dist = Vec2 {
            x: (1.0 / ray.dir.x).abs(),
            y: (1.0 / ray.dir.y).abs()
        };
        // side_dist: distances from current position to next x or y-side
        let mut side_dist = Vec2::default();
        
        // setting step functions and initial side_dist depending on the ray directions
        let (on_x_border, x_step):
            (fn(&Map, usize) -> bool, fn(&mut usize)) =
            if ray.dir.x < 0.0 {
                side_dist.x = (ray.origin.x - tile_x as f32) * delta_dist.x;
                (Self::negative_on_border, Self::negative_step)
            } else {
                side_dist.x = (tile_x as f32 + 1.0 - ray.origin.x) * delta_dist.x;
                (Self::positive_on_x_border, Self::positive_step)
            };
        let (on_y_border, y_step):
            (fn(&Map, usize) -> bool, fn(&mut usize)) =
            if ray.dir.y < 0.0 {
                side_dist.y = (ray.origin.y - tile_y as f32) * delta_dist.y;
                (Self::negative_on_border, Self::negative_step)
            } else {
                side_dist.y = (tile_y as f32 + 1.0 - ray.origin.y) * delta_dist.y;
                (Self::positive_on_y_border, Self::positive_step)
            };
        
        // returns perpendicular wall dist or none if ray goes out of bounds
        loop {
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
                    return Some(t_min.unwrap() / ray_len);
                }
            }

            // jump to next tile, either in x-direction or in y-direction
            if side_dist.x <= side_dist.y && !on_x_border(&self.map, tile_x) {
                side_dist.x += delta_dist.x;
                x_step(&mut tile_x);
            } else if side_dist.y < side_dist.x && !on_y_border(&self.map, tile_y) {
                side_dist.y += delta_dist.y;
                y_step(&mut tile_y);
            } else {
                return None;
            }
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, pos: Vec2, dir: Vec2, fov: f32) -> Result<(), Box<dyn Error>> {
        if let Some(ref skybox) = self.skybox.as_ref() {
            skybox.draw(canvas, dir, fov);
        }
        // half length of the camera plane: 
        let camera_plane_hlen = (fov / 2.0).to_radians().tan();
        
        let camera_plane_dir = dir.orthogonal(true) * camera_plane_hlen; // camera plane vector
        let (width, height) = canvas.window().drawable_size();

        for x in 0..width {
            let camera_x = 2.0 * (x as f32) / (width as f32) - 1.0; //x-coordinate in camera space
            let wall_dist = self.get_wall_dist(Ray::new(pos, dir + camera_plane_dir * camera_x));
            if wall_dist.is_none() {
                continue;
            }

            // height of line to draw on screen
            let line_height = ((height as f32 / wall_dist.unwrap()) * 1.3) as i32;

            // calculate lowest and highest pixel to fill in current stripe
            let draw_start = {
                let start = (height as i32 - line_height) / 2;
                if start < 0 { 0 } else { start }
            };
            let draw_end = {
                let end = height as i32 / 2 + line_height / 2;
                if end >= height as i32 { (height - 1) as i32 } else { end }
            };

            // draw the pixels of the stripe as a vertical line
            let color = sdl2::pixels::Color::RGB(0x24, 0x70, 0x48);
            let p1 = sdl2::rect::Point::new(x as i32, draw_start);
            let p2 = sdl2::rect::Point::new(x as i32, draw_end);
            canvas.set_draw_color(color);
            canvas.draw_line(p1, p2)?;
        }
        Ok(())
    }
}