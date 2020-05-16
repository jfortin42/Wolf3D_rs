use timer;

use math_2d::{
    Vec2,
    Mat2,
};

use level::{
    Map,
    GeometrySet,
};

mod input {

    pub use controls::controls::*;
    
    enum KeyHandle<'a> {
        Hold {
            _press: ControlHandle<'a>,
            _release: ControlHandle<'a>,
        },
        _OneTouch(ControlHandle<'a>),
    }

    pub struct Key<'a> {
        pub active: Rc<Cell<bool>>,
        _handle: KeyHandle<'a>,
    }

    impl<'a> Key<'a> {
        pub fn new(control_handler: &mut ControlHandler<'a>, scancode: Scancode) -> Result<Self, Box<dyn Error>> {
            let active = Rc::new(Cell::new(false));

            let mut set_action = |event, value| -> std::result::Result<ControlHandle<'a>, Box<dyn Error>> {
                let clone_active = Rc::downgrade(&active);
                let action = Box::new(move |_: Event| { clone_active.upgrade().unwrap().set(value); });
                let handle = control_handler.add_control(ControlManagerType::Game, event, action)?;
                Ok(handle)
            };
            
            let event = Event::KeyDown {
                timestamp: 0, window_id: 0, keycode: None, scancode: Some(scancode), keymod: Mod::NOMOD, repeat: false
            };
            let _press = set_action(event, true)?;
            
            let event = Event::KeyUp {
                timestamp: 0, window_id: 0, keycode: None, scancode: Some(scancode), keymod: Mod::NOMOD, repeat: false
            };
            let _release = set_action(event, false)?;
            Ok(Self { active, _handle: KeyHandle::Hold{ _press, _release } })
        }
    }
    
    pub struct MouseMotion<'a> {
        pub xrel: Rc<Cell<i32>>,
        pub yrel: Rc<Cell<i32>>,
        _handle: ControlHandle<'a>,
    }

    impl<'a> MouseMotion<'a> {
        pub fn new(control_handler: &mut ControlHandler<'a>) -> Result<Self, Box<dyn Error>> {
            let xrel = Rc::new(Cell::new(0));
            let yrel = Rc::new(Cell::new(0));

            let xrel_clone = Rc::downgrade(&xrel);
            let yrel_clone = Rc::downgrade(&yrel);

            let event = Event::MouseMotion {
                timestamp: 0, window_id: 0, which: 0, mousestate: MouseState::from_sdl_state(MouseButton::Unknown as u32), x: 0, y: 0, xrel: 0, yrel: 0
            };
            let action = Box::new(move |motion_event: Event| {
                match motion_event {
                    Event::MouseMotion{ xrel, yrel, .. } => {
                        xrel_clone.upgrade().unwrap().set(xrel);
                        yrel_clone.upgrade().unwrap().set(yrel);
                    },
                    _ => {
                        panic!("mouse motion action: expected MouseMotion event, found {:#?}", motion_event);
                    }
                }
            });
            let _handle = control_handler.add_control(ControlManagerType::Game, event, action)?;

            Ok(Self { xrel, yrel, _handle })
        }
    }
}

use self::input::*;

pub struct PlayerActions<'a> {
    forward: Key<'a>,
    left: Key<'a>,
    backward: Key<'a>,
    right: Key<'a>,
    turn: MouseMotion<'a>,
}

impl<'a> PlayerActions<'a> {
    pub fn new(control_handler: &mut ControlHandler<'a>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            forward: Key::new(control_handler, Scancode::W)?,
            left: Key::new(control_handler, Scancode::A)?,
            backward: Key::new(control_handler, Scancode::S)?,
            right: Key::new(control_handler, Scancode::D)?,
            turn: MouseMotion::new(control_handler)?,
        })
    }
}

pub struct Player<'a> {
    pub position: Vec2,
    pub direction: Vec2,
    move_speed: f32,
    turn_speed: f32,
    actions: PlayerActions<'a>,
}

impl<'a> Player<'a> {
    pub fn new(spawn_pos: Vec2, direction: Vec2, control_handler: &mut ControlHandler<'a>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            position: spawn_pos,
            direction,
            move_speed: 1.5, // tiles per second
            turn_speed: 0.524, // radians per second
            actions: PlayerActions::new(control_handler)?,
        })
    }

    pub fn update(&mut self, map: &Map, geo_set: &GeometrySet) {
        self.update_position(map, geo_set);
        self.update_direction();
    }

    fn update_direction(&mut self) {
        let angle = self.actions.turn.xrel.get() as f32 * self.turn_speed * timer::get_delta_time();
        let rot_matrix = Mat2::rotation_matrix(angle);
        
        // rotation matrix:
        // [ cos(a) -sin(a) ] [dir.x]
        // [ sin(a)  cos(a) ] [dir.y]
        self.direction = rot_matrix * self.direction;
        self.direction.normalize();
        self.actions.turn.xrel.set(0);
        self.actions.turn.yrel.set(0);
    }

    fn update_position(&mut self, map: &Map, geo_set: &GeometrySet) {
        let mut move_direction = Vec2::default();
        
        if self.actions.forward.active.get() {
            move_direction += self.direction;
        }
        if self.actions.left.active.get() {
            move_direction += self.direction.orthogonal(false);
        }
        if self.actions.backward.active.get() {
            move_direction += self.direction * -1.0;
        }
        if self.actions.right.active.get() {
            move_direction += self.direction.orthogonal(true);
        }
        move_direction.normalize();

        let old_x = self.position.x as isize;
        let old_y = self.position.y as isize;
        self.position += move_direction * self.move_speed * timer::get_delta_time();
        let new_x = self.position.x as isize;
        let new_y = self.position.y as isize;
        
        // temporary
        // collision solution with tiles (not with geometry primitives)
        let delta = 0.001f32;
        if move_direction.x < 0.0 {
            if new_x < 0 || (new_x == old_x && old_x == 0) {
                if self.position.x < delta {
                    self.position.x = delta;
                }
            } else if (new_x != old_x
                    && geo_set.geometries[map.tiles[old_y as usize][new_x as usize].geo_idx].is_some())
                || (new_x == old_x
                    && geo_set.geometries[map.tiles[old_y as usize][(old_x - 1) as usize].geo_idx].is_some() 
                    && self.position.x < old_x as f32 + delta) {
                self.position.x = old_x as f32 + delta;
            }
        } else {
            if new_x >= map.width as isize || (new_x == old_x && old_x == (map.width - 1) as isize) {
                if self.position.x > map.width as f32 - delta {
                    self.position.x = map.width as f32 - delta;
                }
            } else if (new_x != old_x
                    && geo_set.geometries[map.tiles[old_y as usize][new_x as usize].geo_idx].is_some())
                || (new_x == old_x
                    && geo_set.geometries[map.tiles[old_y as usize][(old_x + 1) as usize].geo_idx].is_some() 
                    && self.position.x > old_x as f32 + 1.0 - delta) {
                self.position.x = old_x as f32 + 1.0 - delta;
            }
        }
        if move_direction.y < 0.0 {
            if new_y < 0 || (new_y == old_y && old_y == 0) {
                if self.position.y < delta {
                    self.position.y = delta;
                }
            } else if (new_y != old_y
                    && geo_set.geometries[map.tiles[new_y as usize][old_x as usize].geo_idx].is_some())
                || (new_y == old_y
                    && geo_set.geometries[map.tiles[(old_y - 1) as usize][old_x as usize].geo_idx].is_some() 
                    && self.position.y < old_y as f32 + delta) {
                self.position.y = old_y as f32 + delta;
            }
        } else {
            if new_y >= map.height as isize || (new_y == old_y && old_y == (map.height - 1) as isize) {
                if self.position.y > map.height as f32 - delta {
                    self.position.y = map.height as f32 - delta;
                }
            } else if (new_y != old_y
                    && geo_set.geometries[map.tiles[new_y as usize][old_x as usize].geo_idx].is_some())
                || (new_y == old_y
                    && geo_set.geometries[map.tiles[(old_y + 1) as usize][old_x as usize].geo_idx].is_some() 
                    && self.position.y > old_y as f32 + 1.0 - delta) {
                self.position.y = old_y as f32 + 1.0 - delta;
            }
        }
    }
}