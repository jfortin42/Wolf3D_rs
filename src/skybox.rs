use parser::*;

use sdl2::image::LoadTexture;
pub use sdl2::render::TextureCreator;
pub use sdl2::video::WindowContext;
use sdl2::render::{
    Texture,
    Canvas
};

use sdl2::video::{Window};
use sdl2::rect::Rect;

use math_2d::{
    Vec2,
};

pub struct Skybox <'a> {
    texture: Texture<'a>,
}

impl<'a> Skybox<'a> {
    pub fn new<P: AsRef<Path>>(name: P, texture_creator: &'a TextureCreator<WindowContext>) -> Result<Skybox<'a> ,Box<dyn Error>> {
        let mut path = PathBuf::from("assets");
        path.push("skybox");
        path.push(name);

        let texture = texture_creator.load_texture(path)?;
        Ok(Self { texture })
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, dir: Vec2, fov: f32) {
        let tex_size = self.texture.query();
        let width_per_deg = tex_size.width as f32 / 360.0;
        let (win_width, win_height) = canvas.window().drawable_size();
        let dir = if dir.y >= 0.0 {
            dir.x.acos().to_degrees()
        } else {
            360.0 - (dir.x.acos().to_degrees())
        };
        let fov_right = (dir + fov / 2.0) % 360.0;

        let (fov_left, x, ratio) = if fov_right < fov {
            let fov_left = fov - fov_right;
            let ratio = fov_left / fov;
            let stencil = Rect::new(
                0,
                0,
                (fov_right * width_per_deg) as u32,
                tex_size.height
            );
            let dest = Rect::new(
                (ratio * win_width as f32) as i32,
                0,
                ((1.0 - ratio) * win_width as f32) as u32,
                win_height / 2,
            );
            canvas.copy(&self.texture, stencil, dest).unwrap();
            (fov_left,
                tex_size.width as i32 - (fov_left * width_per_deg) as i32,
                ratio)
        } else {
            (fov, 
                ((fov_right - fov) * width_per_deg) as i32,
                1.0)
        };

        let stencil = Rect::new(
            x,
            0,
            (fov_left * width_per_deg) as u32,
            tex_size.height
        );

        let dest = Rect::new(
            0,
            0,
            (ratio * win_width as f32) as u32,
            win_height / 2,
        );
        canvas.copy(&self.texture, stencil, dest).unwrap();
    }
}