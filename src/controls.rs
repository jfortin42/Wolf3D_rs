pub use sdl2::event::Event;
use sdl2::event::EventPollIterator;
use sdl2::keyboard::Mod;
use sdl2::keyboard::Scancode;

struct Binding {
    event: Event,
    action: Box<dyn Fn()>,
}

#[repr(u8)]
enum ControlManip {
    Add = 0,
    Call = 1,
}

pub struct Controls {
    quit_binding: Option<Binding>,
    _appterminating_binding: Option<Binding>,
    _applowmemory_binding: Option<Binding>,
    _appwillenterbackground_binding: Option<Binding>,
    _appdidenterbackground_binding: Option<Binding>,
    _appwillenterforeground_binding: Option<Binding>,
    _appdidenterforeground_binding: Option<Binding>,
    window_bindings: Vec<Binding>,
    keydown_bindings: Vec<Binding>,
    keyup_bindings: Vec<Binding>,
    _mouseup_bindings: Vec<Binding>,
    _mousedown_bindings: Vec<Binding>,
    _mousewheel_bindings: Vec<Binding>,
}

type LeaveRequested = bool;

impl Controls {
    pub fn new() -> Self {
        let mut controls = Controls {
                    quit_binding: None,
                    _appterminating_binding: None,
                    _applowmemory_binding: None,
                    _appwillenterbackground_binding: None,
                    _appdidenterbackground_binding: None,
                    _appwillenterforeground_binding: None,
                    _appdidenterforeground_binding: None,
                    window_bindings: Vec::new(),
                    keydown_bindings: Vec::new(),
                    keyup_bindings: Vec::new(),
                    _mouseup_bindings: Vec::new(),
                    _mousedown_bindings: Vec::new(),
                    _mousewheel_bindings: Vec::new()
                };

        let evt = Event::KeyDown { timestamp: 0, window_id: 0, keycode: None, scancode: Some(Scancode::W), keymod: Mod::NOMOD, repeat: false };
        let action =  || { println!("forward"); };
        controls.add(evt.clone(), action);

        controls
    }

    fn add_single_binding<T: Fn() + 'static>(binding: & mut Option<Binding>, event: Event, action: T) {
        *binding = Some(Binding { event, action: Box::new(action) });
    }

    fn call_action<T: Fn() + 'static>(binding: & mut Option<Binding>, _e: Event, _a: T) {
        if let Some(b) = binding {
            (b.action)();
        }
    }

    fn add_binding_to_vec<T: Fn() + 'static>(vec: & mut Vec<Binding>, event: Event, action: T) {
        match vec.iter_mut().find(|b| b.event == event) {
            Some(binding) => { binding.action = Box::new(action); },
            None => { vec.push(Binding { event, action: Box::new(action) }); }
        }
    }

    fn call_action_from_vec<T: Fn() + 'static>(vec: & mut Vec<Binding>, event: Event, _: T) {
        if let Some(binding) = vec.iter().find(|b| b.event == event) {
            (binding.action)();
        }
    }

    fn manip_controls<T: Fn() + 'static>(&mut self, manip: ControlManip, event: Event, action: T) {
        let manip_single = [ Self::add_single_binding, Self::call_action ];
        let manip_vec = [ Self::add_binding_to_vec, Self::call_action_from_vec ];

        match event {
            Event::Quit { .. } => {
                manip_single[manip as usize](& mut self.quit_binding, event, action); 
            },
            Event::Window { win_event, .. } => {
                let event = Event::Window { timestamp: 0, window_id: 0, win_event };
                manip_vec[manip as usize](& mut self.window_bindings, event, action);
            },
            Event::KeyDown { scancode, .. } => {
                let event = Event::KeyDown { timestamp: 0, window_id: 0, keycode: None, scancode, keymod: Mod::NOMOD, repeat: false };
                manip_vec[manip as usize](& mut self.keydown_bindings, event, action);
            },
            Event::KeyUp { scancode, .. } => {
                let event = Event::KeyUp { timestamp: 0, window_id: 0, keycode: None, scancode, keymod: Mod::NOMOD, repeat: false };
                manip_vec[manip as usize](& mut self.keyup_bindings, event, action);
            }
            _ => {},
        }
    }

    pub fn add<T: Fn() + 'static>(&mut self, event: Event, action: T) {
        self.manip_controls(ControlManip::Add, event, action);
    }

    pub fn call_loop(&mut self, event_poll_iterator: EventPollIterator) -> LeaveRequested {
        for event in event_poll_iterator {
           self.manip_controls(ControlManip::Call, event, || {});
        }
        false
    }
}
