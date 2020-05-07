pub mod controls {

    mod control_manager {

        pub use std::error::Error;

        pub use std::fmt::Debug;

        pub use std::rc::{ Rc, Weak };
        pub use std::cell::{ RefCell, Cell };

        pub use sdl2::{ 
            event::Event,
            EventPump,
            keyboard::{
                Scancode,
                Mod,
            },
            mouse::{
                MouseButton,
                MouseState,
            },
        };

        
        trait Control: Debug {
            fn get_event(&self) -> &Event;
            fn call_action(&mut self, event: Event);
        }

        // dynamic or static ?
        struct ControlBinding<T>
            where T: FnMut(Event)
        {
            event: Event,
            action: T,
        }
        
        impl<T: FnMut(Event)> Debug for ControlBinding<T> {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                let _ = write!(fmt, "ControlBinding {{ event: {:?} }}", self.event);
                Ok(())
            }
        }

        impl<T: FnMut(Event)> Control for ControlBinding<T> {
            fn get_event(&self) -> &Event {
                &self.event
            }
            
            fn call_action(&mut self, event: Event) {
                (self.action)(event);
            }
        }
        
        #[derive(Debug)]
        pub struct ControlManager<'a> {
            quit_controls: Vec<Option<Box<dyn Control + 'a>>>,
            window_controls: Vec<Option<Box<dyn Control + 'a>>>,
            keydown_controls: Vec<Option<Box<dyn Control + 'a>>>,
            keyup_controls: Vec<Option<Box<dyn Control + 'a>>>,
            mousemotion_controls: Vec<Option<Box<dyn Control + 'a>>>,
            mousedown_controls: Vec<Option<Box<dyn Control + 'a>>>,
            mouseup_controls: Vec<Option<Box<dyn Control + 'a>>>,
            mousewheel_controls: Vec<Option<Box<dyn Control + 'a>>>,
        }
        
        impl<'a> ControlManager<'a> {
            pub fn new() -> Rc<RefCell<Self>> {
                Rc::new(RefCell::new(Self {
                    quit_controls: vec![],
                    window_controls: vec![],
                    keydown_controls: vec![],
                    keyup_controls: vec![],
                    mousemotion_controls: vec![],
                    mousedown_controls: vec![],
                    mouseup_controls: vec![],
                    mousewheel_controls: vec![],
                }))
            }
        
            pub fn call_loop(&mut self, event_pump: &mut EventPump) {
                for event in event_pump.poll_iter() {
                    if let Some((controls, trimmed_event)) = self.get_controls(&event) {
                        if let Some(control) = controls.iter_mut().find(|b| b.is_some() && *b.as_ref().unwrap().get_event() == trimmed_event) {
                            control.as_mut().unwrap().call_action(event);
                        }
                    }
                }
            }
        
            fn get_controls(&mut self, event: &Event) -> Option<(&mut Vec<Option<Box<dyn Control + 'a>>>, Event)> {
                match *event {
                    Event::Quit { .. } => {
                        let trimmed = Event::Quit{ timestamp: 0 };
                        Some((&mut self.quit_controls, trimmed))
                    },
                    Event::Window { win_event, .. } => {
                        let trimmed = Event::Window { timestamp: 0, window_id: 0, win_event };
                        Some((&mut self.window_controls, trimmed))
                    },
                    Event::KeyDown { scancode, .. } => {
                        let trimmed = Event::KeyDown { timestamp: 0, window_id: 0, keycode: None, scancode, keymod: Mod::NOMOD, repeat: false };
                        Some((&mut self.keydown_controls, trimmed))
                    },
                    Event::KeyUp { scancode, .. } => {
                        let trimmed = Event::KeyUp { timestamp: 0, window_id: 0, keycode: None, scancode, keymod: Mod::NOMOD, repeat: false };
                        Some((&mut self.keyup_controls, trimmed))
                    },
                    Event::MouseMotion{ mousestate: _, x: _, y: _, xrel: _, yrel: _, .. } => {
                        let trimmed = Event::MouseMotion{ timestamp: 0, window_id: 0, which: 0, mousestate: MouseState::from_sdl_state(MouseButton::Unknown as u32), x: 0, y: 0, xrel: 0, yrel: 0 };
                        Some((&mut self.mousemotion_controls, trimmed))
                    },
                    Event::MouseButtonDown { mouse_btn, clicks, x: _x, y: _y, .. } => {
                        // store x and y for later use
                        let trimmed = Event::MouseButtonDown { timestamp: 0, window_id: 0, which: 0, mouse_btn, clicks, x: 0, y: 0 };
                        Some((&mut self.mousedown_controls, trimmed))
                    },
                    Event::MouseButtonUp { mouse_btn, clicks, x: _x, y: _y, .. } => {
                        let trimmed = Event::MouseButtonUp { timestamp: 0, window_id: 0, which: 0, mouse_btn, clicks, x: 0, y: 0 };
                        Some((&mut self.mouseup_controls, trimmed))
                    },
                    Event::MouseWheel { x: _x, y: _y, direction, .. } => {
                        let trimmed = Event::MouseWheel { timestamp: 0, window_id: 0, which: 0, x: 0, y: 0, direction };
                        Some((&mut self.mousewheel_controls, trimmed))
                    },
                    _ => { None }
                }
            }
        
            pub fn add_control(&mut self, event: Event, action: Box<dyn FnMut(Event) + 'a>) -> Result<usize, Box<dyn Error>> {
                let controls = self.get_controls(&event);
                
                if controls.is_none() {
                    return Err("event unknown".into());
                }
        
                let (controls, event) = controls.unwrap();
                let mut free_spot: Option<usize> = None;
        
                for (idx, control) in controls.iter().enumerate() {
                    if let Some(control) = control {
                        if event == *control.get_event() {
                            return Err("a control is already bound to this event".into());
                        }
                    } else if free_spot.is_none() {
                        free_spot = Some(idx);
                    }
                }
                if let Some(idx) = free_spot {
                    controls[idx] = Some(Box::new(ControlBinding { event, action }));
                    Ok(idx)
                } else {
                    controls.push(Some(Box::new(ControlBinding { event, action })));
                    Ok(controls.len() - 1)
                }
            }
        
            pub fn replace_control(&mut self, event: Event, idx: usize, action: Box<dyn FnMut(Event) + 'a>) {
                let (controls, event) = self.get_controls(&event).unwrap();
                let binding = Box::new(ControlBinding { event: event.clone(), action });
                let old_control = controls[idx].replace(binding);
                assert!(old_control.is_some(), "missing control");
                assert_eq!(*old_control.unwrap().get_event(), event, "events do not match");
            }
        
            pub fn remove_control(&mut self, event: Event, idx: usize) {
                let (controls, _) = self.get_controls(&event).unwrap();
                assert!(controls[idx].take().is_some(), "missing control to remove");
            }
        }
    }

    use self::control_manager::{
        Debug,
        ControlManager,
    };

    pub use self::control_manager::{
        Error,
        { Rc, Weak },
        { RefCell, Cell },
        Event,
        EventPump,
        Scancode,
        Mod,
        MouseButton,
        MouseState,
    };

    pub struct ControlHandle<'a> {
        manager: Weak<RefCell<ControlManager<'a>>>,
        event: Event,
        idx: usize
    }

    impl<'a> Debug for ControlHandle<'a> {
        fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            let _ = write!(fmt, "ControlHandle {{ event: {:?}, idx {} }}", self.event, self.idx);
            Ok(())
        }
    }

    impl<'a> Drop for ControlHandle<'a> {
        fn drop(&mut self) {
            let manager = self.manager.upgrade().unwrap();
            manager.borrow_mut().remove_control(self.event.clone(), self.idx);
        }
    }

    impl<'a> ControlHandle<'a> {
        pub fn replace(&mut self, action: Box<dyn FnMut(Event) + 'a>) {
            self.manager.upgrade().unwrap().borrow_mut().replace_control(self.event.clone(), self.idx, action);
        }
    
        // optional
        pub fn remove(self) {
            std::mem::drop(self);
        }
    }

    #[repr(usize)]
    #[derive(Clone, Copy)]
    pub enum ControlManagerType {
        Game = 0,
        Menu = 1,
        // etc.
    }

    pub struct ControlHandler<'a> {
        control_managers: [Rc<RefCell<ControlManager<'a>>>; 2],
        active_manager: ControlManagerType
    }

    impl<'a> ControlHandler<'a> {
        pub fn new() -> Self {
            Self {
                control_managers: [ControlManager::new(), ControlManager::new()],
                active_manager: ControlManagerType::Game
            }
        }

        pub fn call_loop(&mut self, event_pump: &mut EventPump) {
            let manager = &mut self.control_managers[self.active_manager as usize];
            manager.borrow_mut().call_loop(event_pump);
        }

        #[must_use = "the control is dropped when the ControlHandle is dropped"]
        pub fn add_control(&mut self, ctrl_manager_type: ControlManagerType, event: Event, action: Box<dyn FnMut(Event) + 'a>) -> Result<ControlHandle<'a>, Box<dyn Error>> {
            let manager = &mut self.control_managers[ctrl_manager_type as usize];
            let idx = manager.borrow_mut().add_control(event.clone(), action)?;
            Ok(ControlHandle { manager: Rc::downgrade(manager), event, idx })
        }
    }
}
