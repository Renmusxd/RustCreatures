use crate::model::creature::DietEnum;
use crate::model::{Observation, World};
use sdl2::event::{Event, EventPollIterator};
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use sdl2::EventPump;

pub struct View {
    canvas: WindowCanvas,
    event_pump: EventPump,
}

impl View {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window_size = 800;
        let window = video_subsystem
            .window("Conformal Mappings", window_size, window_size)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        Self { canvas, event_pump }
    }

    pub fn present(&mut self, w: &World) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Draw grass
        let scaling = 10;
        let grass_max = w.get_grass_max() as f64;
        w.get_grass_slice()
            .iter()
            .enumerate()
            .try_for_each(|(i, v)| -> Result<(), String> {
                let (x, y) = w.get_grass_loc(i);
                let g = ((*v as f64 / grass_max) * 255.).floor() as u8;
                self.canvas.set_draw_color(Color::RGB(0, g, 0));
                let r = Rect::new(
                    (x * scaling) as i32 - (scaling / 2) as i32,
                    (y * scaling) as i32 - (scaling / 2) as i32,
                    scaling as u32,
                    scaling as u32,
                );
                self.canvas.fill_rect(r)?;
                self.canvas.set_draw_color(Color::RGB(127, 127, 127));
                self.canvas.draw_rect(r)?;
                Ok(())
            })?;

        // Draw Creatures
        let scaling = scaling as f64;
        w.get_creature_slice()
            .iter()
            .try_for_each(|c| -> Result<(), String> {
                let (x, y, theta) = c.get_pos();
                let col = c.get_color();
                let bw = col.atan();
                let v = (255. * bw).floor() as u8;

                let col_bord = Color::RGB(v, v, v);
                self.canvas.filled_circle(
                    (x * scaling) as i16,
                    (y * scaling) as i16,
                    (scaling / 2.) as i16,
                    col_bord,
                )?;
                let col = match c.get_diet() {
                    DietEnum::VEGETARIAN => Color::RGB(0, 255, 0),
                    DietEnum::CARNIVORE => Color::RGB(255, 0, 0),
                };
                self.canvas.filled_circle(
                    (x * scaling) as i16,
                    (y * scaling) as i16,
                    (scaling / 2.) as i16 - 2,
                    col,
                )?;

                let (theta_s, theta_c) = theta.sin_cos();

                let start = Point::new((x * scaling) as i32, (y * scaling) as i32);
                let end = Point::new(
                    ((x + theta_c) * scaling) as i32,
                    ((y + theta_s) * scaling) as i32,
                );
                self.canvas.set_draw_color(col_bord);
                self.canvas.draw_line(start, end)?;

                if let Some(o) = c.get_last_observation() {
                    o.dists
                        .iter()
                        .cloned()
                        .zip(o.colors.iter().cloned())
                        .enumerate()
                        .try_for_each(|(i, (d, c))| {
                            let dangle = Observation::VISION_RANGE / Observation::NUM_SITES as f64;
                            let angle = i as f64 * dangle - Observation::VISION_RANGE / 2.0 + theta;
                            let (a_s, a_c) = angle.sin_cos();
                            let (a_ds, a_dc) = (angle + dangle).sin_cos();

                            let start = Point::new(
                                ((x + d * a_c) * scaling) as i32,
                                ((y + d * a_s) * scaling) as i32,
                            );
                            let end = Point::new(
                                ((x + d * a_dc) * scaling) as i32,
                                ((y + d * a_ds) * scaling) as i32,
                            );
                            let v = (255. * c).floor() as u8;
                            let col = Color::RGB(v, v, v);
                            self.canvas.set_draw_color(col);
                            self.canvas.draw_line(start, end)
                        })?;
                }

                Ok(())
            })?;

        self.canvas.present();

        Ok(())
    }

    pub fn get_events(&mut self) -> EventPollIterator {
        self.event_pump.poll_iter()
    }
}
