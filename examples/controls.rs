extern crate wolf3d_rs;
use wolf3d_rs::*;

fn main() {
    println!("controls example");
    let game_loop = Cell::new(true);

    let mut control_handler = ControlHandler::new();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
     
    let _window = video_subsystem.window("Press Escape for quit", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    
    let event_quit = Event::KeyDown {
        timestamp: 0, window_id: 0, keycode: None, scancode: Some(Scancode::Escape), keymod: Mod::NOMOD, repeat: false
    };
    let quit_action = Box::new(|| {
        game_loop.set(false);
        println!("Escape key pressed: Quit!");
    });
    let _quit_handle = control_handler
        .add_control(ControlManagerType::Game, event_quit, quit_action)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    while game_loop.get() {
        control_handler.call_loop(&mut event_pump);
        // The rest of the game loop goes here...  
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}