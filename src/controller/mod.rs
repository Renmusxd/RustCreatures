use crate::model::World;
use crate::view::View;
use ndarray_rand::rand::rngs::ThreadRng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct Controller {
    world: World,
    view: View,
    draw_hertz: u32,
    hertz: u32,
}

impl Controller {
    pub fn new(w: World, v: View) -> Self {
        Self {
            world: w,
            view: v,
            draw_hertz: 60,
            hertz: 6000,
        }
    }

    pub fn run_loop(&mut self) -> bool {
        self.view.present(&self.world).unwrap();

        let mut move_up = false;
        let mut move_down = false;
        let mut move_left = false;
        let mut move_right = false;

        let mut rng: ThreadRng = ThreadRng::default();
        let mut last_draw = UNIX_EPOCH;
        let mut last_update = UNIX_EPOCH;
        'running: loop {
            let now_time = SystemTime::now();
            let since_last_draw = now_time
                .duration_since(last_draw)
                .unwrap_or(Duration::new(0, 0));
            let since_last_update = now_time
                .duration_since(last_update)
                .unwrap_or(Duration::new(0, 0));

            if since_last_draw > Duration::new(0, 1_000_000_000u32 / self.draw_hertz) {
                last_draw = now_time;
                self.view.present(&self.world).unwrap();
            }
            if since_last_update > Duration::new(0, 1_000_000_000u32 / self.hertz) {
                last_update = now_time;
                self.world.update(&mut rng);
            }

            let mut toggle_v = false;

            let mut diff_scale = 1.0;
            for event in self.view.get_events() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running true,
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        repeat: false,
                        ..
                    } => {
                        move_up = true;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        repeat: false,
                        ..
                    } => {
                        move_down = true;
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Up),
                        repeat: false,
                        ..
                    } => {
                        move_up = false;
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Down),
                        repeat: false,
                        ..
                    } => {
                        move_down = false;
                    }

                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        repeat: false,
                        ..
                    } => {
                        move_left = true;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        repeat: false,
                        ..
                    } => {
                        move_right = true;
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Left),
                        repeat: false,
                        ..
                    } => {
                        move_left = false;
                    }
                    Event::KeyUp {
                        keycode: Some(Keycode::Right),
                        repeat: false,
                        ..
                    } => {
                        move_right = false;
                    }
                    Event::MouseWheel { y: -1, .. } => {
                        diff_scale *= 1.1;
                    }
                    Event::MouseWheel { y: 1, .. } => {
                        diff_scale /= 1.1;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::V),
                        repeat: false,
                        ..
                    } => toggle_v = true,
                    _ => {}
                }
            }
            let mut diff_x = 0.0;
            let mut diff_y = 0.0;
            let scaling = self.view.get_scaling();
            if move_up {
                diff_y += 10. / scaling;
            }
            if move_down {
                diff_y -= 10. / scaling;
            }
            if move_left {
                diff_x += 10. / scaling;
            }
            if move_right {
                diff_x -= 10. / scaling;
            }

            self.view.diff_off(diff_x, diff_y);
            self.view.diff_scale(diff_scale);

            if toggle_v {
                self.view.toggle_vision();
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / self.hertz));
        }
    }
}
