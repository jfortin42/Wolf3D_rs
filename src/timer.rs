static mut TIMER: Option<sdl2::TimerSubsystem> = None; 
static mut LAST_TIME: Option<u32> = None;
static mut DELTA_TIME: f32 = 0.0;

pub fn init_timer(timer_subsystem: sdl2::TimerSubsystem) {
    unsafe {
        TIMER = Some(timer_subsystem);
    }
}

pub fn update() {
    unsafe {
        match LAST_TIME {
            None => {
                LAST_TIME = Some(TIMER.as_mut().unwrap().ticks());
            },
            Some(last_time) => {
                let current = TIMER.as_mut().unwrap().ticks();
                DELTA_TIME = (current - last_time) as f32 / 1000.0;
                LAST_TIME = Some(current);
            }
        }
    }
}

/// from_update() gives the elapsed time since the last call to update.
pub fn from_update() -> f32 {
    unsafe {
        let current = TIMER.as_mut().unwrap().ticks();
        (current - LAST_TIME.unwrap()) as f32 / 1000.0
    }
}

/// get_delta_time provides the time between the current and previous frame.
pub fn get_delta_time() -> f32 {
    unsafe {
        DELTA_TIME
    }
}