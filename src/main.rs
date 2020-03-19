extern crate sdl2;

use sdl2::{pixels::Color};
use std::time::Duration;

mod controls;
use controls::Controls;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();

    let mut controls: controls::Controls = Controls::new();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
        canvas.clear();
        if controls.call_loop(event_pump.poll_iter()) == true {
            break;
        }
        // The rest of the game loop goes here...
        
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    // free all SDL2
}