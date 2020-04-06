extern crate wolf3d_rs;
use wolf3d_rs::*;

use wolf3d_rs::sdl2::{ 
    Sdl,
    EventSubsystem,
    VideoSubsystem,
    video::Window,
};

use std::error::Error;

struct SdlEnv {
    event_pump: EventPump,
    event_subsystem: EventSubsystem,
    _window: Window, //
    _video_subsystem: VideoSubsystem, //
    _sdl_context: Sdl
}

impl SdlEnv {
    fn new() -> Self {
        let _sdl_context = sdl2::init().unwrap();
        let _video_subsystem = _sdl_context.video().unwrap();

        let _window = _video_subsystem.window("rust-sdl2 demo", 800, 600)
            .position_centered().build().unwrap();
        
        let event_subsystem = _sdl_context.event().unwrap();
        let event_pump = _sdl_context.event_pump().unwrap();
        Self { event_pump, event_subsystem, _window, _video_subsystem, _sdl_context }
    }
}

struct TestEnv<'a> {
    size: usize,
    evidence: Vec<Rc<Cell<bool>>>,
    events: Vec<Event>,
    handles: Vec<Option<ControlHandle<'a>>>,
    ctrl_handler: ControlHandler<'a>
}

impl<'a> TestEnv<'a> {
    fn new(size: usize) -> Self {
        let mut evidence: Vec<Rc<Cell<bool>>> = Vec::with_capacity(size);
        let mut events: Vec<Event> = Vec::with_capacity(size);
        let mut handles: Vec<Option<ControlHandle>> = Vec::with_capacity(size);
        for i in 0..size {
            evidence.push(Rc::new(Cell::new(false)));
            let scancode = Some(Scancode::from_i32(Scancode::A as i32 + i as i32).unwrap());
            let event = Event::KeyDown {
                timestamp: 0, window_id: 0, keycode: None, scancode, keymod: Mod::NOMOD, repeat: false
            };
            events.push(event);
            handles.push(None);
        }
        let ctrl_handler = ControlHandler::new();
        Self { size, evidence, events, handles, ctrl_handler }
    }

    fn create_action(evidence: Rc<Cell<bool>>) -> Box<dyn FnMut() + 'a> {
        Box::new(move || evidence.set(true))
    }

    fn add_controls(&mut self, indexes: &[usize]) -> Result<(), Box<dyn Error>> {
        for i in 0..indexes.len() {
            let event = self.events[indexes[i]].clone();
            let action = Self::create_action(Rc::clone(&self.evidence[indexes[i]]));
            let handle = self.ctrl_handler.add_control(ControlManagerType::Game, event, action)?;
            self.handles[indexes[i]] = Some(handle);
        }
        Ok(())
    }

    fn remove_controls(&mut self, indexes: &[usize]) -> Result<(), Box<dyn Error>> {
        for i in 0..indexes.len() {
            self.handles[indexes[i]].take().unwrap();
        }
        Ok(())
    }

    fn replace_control(&mut self, dest: usize, src: usize) {
        let action = Self::create_action(Rc::clone(&self.evidence[src]));
        self.handles[dest].as_mut().unwrap().replace(action);
    }

    fn call_controls(&mut self, sdl_env: &mut SdlEnv, event_indexes: &[usize]) {
        for i in 0..event_indexes.len() {
            sdl_env.event_subsystem.push_event(self.events[event_indexes[i]].clone()).unwrap();
        }
        self.ctrl_handler.call_loop(&mut sdl_env.event_pump);
    }

    fn check_evidences(&self, indexes: &[usize], cmp: bool) -> bool {
        for i in 0..indexes.len() {
            if self.evidence[indexes[i]].get() != cmp { return false; }
        }
        true
    }

    fn reset_evidences(&self) {
        for i in 0..self.size {
            self.evidence[i].set(false);
        }
    }

    fn debug_handles(&self, indexes: &[usize]) -> String {
        let mut output = String::new();
        for i in 0..indexes.len() {
            output.push_str(format!("{:#?}\n", self.handles[indexes[i]]).as_str());
        }
        output
    }
}

#[test]
fn control_ok() {
    let mut sdl_env = SdlEnv::new();
    let mut env = TestEnv::new(3);

    // pushing three controls to the manager's keydown_controls list
    env.add_controls(&[0, 1, 2]).unwrap(); // empty -> Some Some Some
    env.call_controls(&mut sdl_env, &[0, 1, 2]);
    assert!(env.check_evidences(&[0, 1, 2], true), env.debug_handles(&[0, 1, 2]));
    
    env.reset_evidences();
    // removing second control which was pushed to the list
    env.remove_controls(&[1]).unwrap(); // Some Some Some -> Some None Some
    env.call_controls(&mut sdl_env, &[0, 1, 2]);
    assert!(env.check_evidences(&[0, 2], true), env.debug_handles(&[0, 2]));
    assert!(env.check_evidences(&[1], false));

    env.reset_evidences();
    env.add_controls(&[1]).unwrap(); // Some None Some -> Some Some Some
    // // removing second control which was added on a free spot of the list
    env.remove_controls(&[1]).unwrap(); // Some Some Some -> Some None Some
    env.call_controls(&mut sdl_env, &[0, 1, 2]);
    assert!(env.check_evidences(&[0, 2], true), env.debug_handles(&[0, 2]));
    assert!(env.check_evidences(&[1], false));

    env.reset_evidences();
    // replacing third control with second one
    env.replace_control(2, 1); // Some(0) None Some(2) -> Some(0) None Some(1)
    env.call_controls(&mut sdl_env, &[0, 1, 2]);
    assert!(env.check_evidences(&[0, 1], true), env.debug_handles(&[0]));
    assert!(env.check_evidences(&[2], false), env.debug_handles(&[2]));
}

#[test]
fn control_err() {
    let mut env = TestEnv::new(2);

    // adding control on an already bound event
    env.add_controls(&[0]).unwrap();
    let err = env.add_controls(&[0]).unwrap_err();
    assert_eq!(err.to_string(), "a control is already bound to this event");

    // adding a control on an unknown event
    let event_unknown = Event::Unknown { timestamp: 0, type_: 0 };
    let err = env.ctrl_handler.add_control(ControlManagerType::Game, event_unknown, Box::new(|| {})).unwrap_err();
    assert_eq!(err.to_string(), "event unknown");
}