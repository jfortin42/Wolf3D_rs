use super::*;

#[derive(Deserialize, Debug)]
pub struct PortalSymbolSet {
    pub previous_level_symbol: Option<char>,
    pub next_level_symbol: Option<char>,
    pub bonus_level_symbol: Option<char>,
    pub end_symbol: Option<char>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Shading {
    Texture(String),
    Color(String),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PhysicsProperties {
    Reflection,
    Refraction { index: f32 },
    Transmission,
    Absorption,
}

#[derive(Deserialize, Debug)]
pub struct MaterialProperties {
    pub physics_properties: PhysicsProperties,
}

#[derive(Deserialize, Debug)]
pub struct Material {
    pub name: String,
    pub symbol: char,
    pub properties: MaterialProperties,
    pub shading: Shading,
}

#[derive(Deserialize, Debug)]
pub struct MaterialSet {
    pub material_set: Vec<Material>,
}

#[derive(Deserialize, Debug)]
pub struct MaterialMap {
    pub material_set: String,
    pub portal_symbol_set: String,
    pub previous_level: Option<String>,
    pub next_level: Option<String>,
    pub bonus_level: Option<String>,
    pub map: Vec<String>,
}
