use crate::model::World;
use crate::view::View;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

pub struct Controller {
    world: World,
    view: View,
    hertz: u32,
}

impl Controller {
    pub fn new(w: World, v: View) -> Self {
        Self {
            world: w,
            view: v,
            hertz: 60,
        }
    }

    pub fn run_loop(&mut self) -> bool {
        self.view.present(&self.world).unwrap();
        'running: loop {
            self.world.update();
            self.view.present(&self.world).unwrap();
            for event in self.view.get_events() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running true,
                    _ => {}
                }
            }
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / self.hertz));
        }
    }
}
