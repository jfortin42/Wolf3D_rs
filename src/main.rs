extern crate wolf3d_rs;
use wolf3d_rs::*;

fn main() -> Result<(), Box<dyn Error>> {
    
    let yml = clap::load_yaml!("clap.yml");
    let matches = App::from_yaml(yml).get_matches();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    // let width = value_t!(matches.value_of("width"), u32).unwrap_or(1024);
    // let height = value_t!(matches.value_of("height"), u32).unwrap_or(768);
    // let fullscreen = matches.is_present("fullscreen");
    let level_name = matches.value_of("level_name").unwrap();
   
    let level = Level::new(level_name)?;
    if level.spawns.len() == 0 {
        return Err("no spawn available".into());
    }
    let game_loop = Cell::new(true);
    let mut control_handler = ControlHandler::new();
    
    let mut player = Player::new(level.spawns[0].coordinates, Vec2 { x: 0.0, y: 1.0 }, &mut control_handler)?;

    sdl_context.mouse().set_relative_mouse_mode(true);
    timer::init_timer(sdl_context.timer()?);

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(70, 75, 90));
    canvas.clear();
    canvas.present();

    let event_quit = Event::KeyDown {
        timestamp: 0, window_id: 0, keycode: None, scancode: Some(Scancode::Escape), keymod: Mod::NOMOD, repeat: false
    };
    let quit_action = Box::new(|_: Event| {
        game_loop.set(false);
        println!("Escape key pressed: Quit!");
    });
    let _quit_handle = control_handler
        .add_control(ControlManagerType::Game, event_quit, quit_action)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    while game_loop.get() {
        timer::update();
        canvas.set_draw_color(Color::RGB(70, 75, 90));
        canvas.clear();
        level.draw(&mut canvas, player.position, player.direction)?;
        control_handler.call_loop(&mut event_pump);
        player.update(&level.map, &level.geo_set);
        canvas.present();
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}