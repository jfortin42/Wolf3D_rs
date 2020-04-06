pub mod controls {

    mod control_manager {

        pub use std::rc::{ Rc, Weak };
        pub use std::cell::{ RefCell };
        pub use std::fmt::Debug;

        pub use sdl2::{ 
            event::Event,
            EventPump,
            keyboard::Mod
        };

        trait Control : Debug {
            fn get_event(&self) -> Event;
            fn call_action(&mut self);
        }
        
        // dynamic or static ?
        struct ControlBinding<T>
            where T: FnMut()
        {
            event: Event,
            action: T
        }
        
        impl<T: FnMut()> Debug for ControlBinding<T> {
            fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                let _ = write!(fmt, "ControlBinding {{ event: {:?} }}", self.event);
                Ok(())
            }
        }
        
        impl<T: FnMut()> Control for ControlBinding<T> {
            fn get_event(&self) -> Event {
                self.event.clone()
            }
            
            fn call_action(&mut self) {
                (self.action)();
            }
        }
        
        #[derive(Debug)]
        pub struct ControlManager<'a> {
            quit_controls: Vec<Option<Box<dyn Control + 'a>>>,
            window_controls: Vec<Option<Box<dyn Control + 'a>>>,
            keydown_controls: Vec<Option<Box<dyn Control + 'a>>>,
            keyup_controls: Vec<Option<Box<dyn Control + 'a>>>,
            mousedown_controls: Vec<Option<Box<dyn Control + 'a>>>,
            mouseup_controls: Vec<Option<Box<dyn Control + 'a>>>,
            mousewheel_controls: Vec<Option<Box<dyn Control + 'a>>>
        }
        
        impl<'a> ControlManager<'a> {
            pub fn new() -> Rc<RefCell<Self>> {
                Rc::new(RefCell::new(Self {
                    quit_controls: vec![],
                    window_controls: vec![],
                    keydown_controls: vec![],
                    keyup_controls: vec![],
                    mousedown_controls: vec![],
                    mouseup_controls: vec![],
                    mousewheel_controls: vec![]
                }))
            }
        
            pub fn call_loop(&mut self, event_pump: &mut EventPump) {
                for mut event in event_pump.poll_iter() {
                    if let Some(controls) = self.get_controls(&mut event) {
                        if let Some(control) = controls.iter_mut().find(|b| b.is_some() && b.as_ref().unwrap().get_event() == event) {
                            control.as_mut().unwrap().call_action();
                        }
                    }
                }
            }
        
            fn get_controls(&mut self, event: &mut Event) -> Option<&mut Vec<Option<Box<dyn Control + 'a>>>> {
                match *event {
                    Event::Quit { .. } => {
                        *event = Event::Quit{ timestamp: 0 };
                        Some(&mut self.quit_controls)
                    },
                    Event::Window { win_event, .. } => {
                        *event = Event::Window { timestamp: 0, window_id: 0, win_event };
                        Some(&mut self.window_controls)
                    },
                    Event::KeyDown { scancode, .. } => {
                        *event = Event::KeyDown { timestamp: 0, window_id: 0, keycode: None, scancode, keymod: Mod::NOMOD, repeat: false };
                        Some(&mut self.keydown_controls)
                    },
                    Event::KeyUp { scancode, .. } => {
                        *event = Event::KeyUp { timestamp: 0, window_id: 0, keycode: None, scancode, keymod: Mod::NOMOD, repeat: false };
                        Some(&mut self.keyup_controls)
                    },
                    Event::MouseButtonDown { mouse_btn, clicks, x, y, .. } => {
                        // store x and y for later use
                        *event = Event::MouseButtonDown { timestamp: 0, window_id: 0, which: 0, mouse_btn, clicks, x: 0, y: 0 };
                        Some(&mut self.mousedown_controls)
                    },
                    Event::MouseButtonUp { mouse_btn, clicks, x, y, .. } => {
                        *event = Event::MouseButtonUp { timestamp: 0, window_id: 0, which: 0, mouse_btn, clicks, x: 0, y: 0 };
                        Some(&mut self.mouseup_controls)
                    },
                    Event::MouseWheel { x, y, direction, .. } => {
                        *event = Event::MouseWheel { timestamp: 0, window_id: 0, which: 0, x: 0, y: 0, direction };
                        Some(&mut self.mousewheel_controls)
                    },
                    _ => { None }
                }
            }
        
            pub fn add_control(&mut self, mut event: Event, action: Box<dyn FnMut() + 'a>) -> Result<usize, &'static str> {
                let controls = self.get_controls(&mut event);
                
                if controls.is_none() {
                    return Err("event unknown");
                }
        
                let controls = controls.unwrap();
                let mut free_spot : Option<usize> = None;
        
                for (idx, control) in controls.iter().enumerate() {
                    if let Some(control) = control {
                        if event == control.get_event() {
                            return Err("a control is already bound to this event");
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
        
            pub fn replace_control(&mut self, mut event: Event, idx: usize, action: Box<dyn FnMut() + 'a>) {
                let controls = self.get_controls(&mut event).unwrap();
                let binding = Box::new(ControlBinding { event: event.clone(), action });
                let old_control = controls[idx].replace(binding);
                assert!(old_control.is_some(), "missing control");
                assert_eq!(old_control.unwrap().get_event(), event, "events do not match");
            }
        
            pub fn remove_control(&mut self, mut event: Event, idx: usize) {
                let controls = self.get_controls(&mut event).unwrap();
                assert!(controls[idx].take().is_some(), "missing control to remove");
            }
        }
    }

    use self::control_manager::*;

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
        pub fn replace(&mut self, action: Box<dyn FnMut() + 'a>) {
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

        #[must_use]
        pub fn add_control(&mut self, ctrl_manager_type: ControlManagerType, event: Event, action: Box<dyn FnMut() + 'a>) -> Result<ControlHandle<'a>, &'static str> {
            let manager = &mut self.control_managers[ctrl_manager_type as usize];
            let idx = manager.borrow_mut().add_control(event.clone(), action)?;
            Ok(ControlHandle { manager: Rc::downgrade(manager), event, idx })
        }
    }
}
