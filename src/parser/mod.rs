pub use std::error::Error;
pub use std::fs::File;
pub use std::io::{
    Read,
    BufReader,
};
pub use std::path::{
    Path,
    PathBuf,
};

use math_2d::Vec2;

use serde::Deserialize;

pub mod material;
pub mod geometry;

pub fn read_assets_file<P, S>(dirs: &[P], file_name: S) -> Result<BufReader<impl Read>, Box<dyn Error>>
    where
        P: AsRef<Path>,
        S: AsRef<Path>
{
    let mut file = PathBuf::from("assets");
    for dir in dirs {
        file.push(dir);
    }
    file.push(file_name);
    file.set_extension("json");
    let file = File::open(file)?;
    Ok(BufReader::new(file))
}

pub fn check_for_duplicate_symbols(symbols: &str) -> Option<char> {
    let symbols: Vec<char> = symbols.chars().collect();
    if symbols.len() < 2 {
        return None;
    }
    if let Some(idx) = (1..symbols.len()).find(|&i| symbols[i..].contains(&symbols[i - 1])) {
        Some(symbols[idx - 1])
    } else {
        None
    }
}