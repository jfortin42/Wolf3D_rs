use super::*;

use parser::material::PortalSymbolSet;

pub use parser::material::PhysicsProperties;

#[derive(Debug)]
pub enum Shading {
    Texture(String), //Texture(SDL_Texture),
    Color(u32),
}

#[derive(Debug)]
pub enum Portal {
    ToLevel(String),
    ToEnd,
}

#[derive(Debug)]
pub struct MaterialProperties {
    pub physics_properties: PhysicsProperties,
    pub portal: Option<Portal>
}

#[derive(Debug)]
pub struct MaterialSet {
    pub material_names: Vec<String>,
    pub material_properties: Vec<MaterialProperties>,
    pub material_shadings: Vec<Shading>,
}

impl MaterialSet {
    const N_PORTALS: usize = 4;

    fn add_portals(&mut self, mtl_map: &MaterialMap, symbols: &mut String) -> Result<(), Box<dyn Error>> { 
        let reader = read_assets_file(&["portal_symbol_sets"], &mtl_map.portal_symbol_set)?;
        let portal_symbol_set: PortalSymbolSet = serde_json::from_reader(reader)?;

        let mut add_portal = |portal_type, level: &Option<String>, symbol: Option<char>| {
            if symbol.is_some() {
                let portal = match portal_type {
                    Portal::ToLevel(portal_name) if level.is_some() => {
                        self.material_names.push(portal_name);
                        Some(Portal::ToLevel(level.as_ref().unwrap().clone()))
                    },
                    Portal::ToEnd => {
                        self.material_names.push(String::from("end"));
                        Some(Portal::ToEnd)
                    },
                    _ => { return; }
                };
                symbols.push(symbol.unwrap());
                self.material_properties.push(MaterialProperties {
                    physics_properties: PhysicsProperties::Absorption,
                    portal
                });
                //self.material_shadings.push(Shading::Texture(SDL_LoadTexture(name + png)); ou Color
            }
        };
        add_portal(Portal::ToLevel(String::from("previous_level")), &mtl_map.previous_level, portal_symbol_set.previous_level_symbol);
        add_portal(Portal::ToLevel(String::from("next_level")), &mtl_map.next_level, portal_symbol_set.next_level_symbol);
        add_portal(Portal::ToLevel(String::from("bonus_level")), &mtl_map.bonus_level, portal_symbol_set.bonus_level_symbol);
        add_portal(Portal::ToEnd, &None, portal_symbol_set.end_symbol);
        Ok(())
    }

    pub fn new(mtl_map: &MaterialMap) -> Result<(Self, String), Box<dyn Error>> {
        let reader = read_assets_file(&["material_sets"], &mtl_map.material_set)?;
        let raw_set: parser::material::MaterialSet = serde_json::from_reader(reader)?;
        
        let n = raw_set.material_set.len() + Self::N_PORTALS;
        let mut set = Self {
            material_names: Vec::with_capacity(n),
            material_properties: Vec::with_capacity(n),
            material_shadings: Vec::with_capacity(n)
        };
        let mut symbols = String::with_capacity(n);

        for material in raw_set.material_set {
            symbols.push(material.symbol);
            set.material_names.push(material.name);
            set.material_properties.push(MaterialProperties {
                physics_properties: material.properties.physics_properties,
                portal: None,
            });
            /*let shading = match mtl.shading {
                parser::material::Shading::Texture(file) => {
                    let reader = 
                    // tex = SDL_LoadTexture(path)?;
                    Shading::Texture(tex)
                },
                parser::material::Shading::Color(clr) => {
                    //Shading::Color(clr.parse().unwrap()) ??
                    Shading::Color(0)
                }
            };
            set.material_shadings.push(shading);*/
        }
        set.add_portals(mtl_map, &mut symbols)?;
        Ok((set, symbols))
    }
}