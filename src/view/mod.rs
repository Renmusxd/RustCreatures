use crate::model::{Observation, World};
use sdl2::event::EventPollIterator;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::WindowCanvas;
use sdl2::EventPump;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub struct View {
    canvas: WindowCanvas,
    event_pump: EventPump,
    scaling: f64,
    xoff: f64,
    yoff: f64,

    draw_vision: bool,
}

impl View {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window_size = 800;
        let window = video_subsystem
            .window("Rust Creatures", window_size, window_size)
            .position_centered()
            .resizable()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        Self {
            canvas,
            event_pump,
            scaling: 10.0,
            xoff: 0.0,
            yoff: 0.0,
            draw_vision: false,
        }
    }

    pub fn toggle_vision(&mut self) {
        self.draw_vision = !self.draw_vision;
    }

    pub fn get_scaling(&self) -> f64 {
        self.scaling
    }

    pub fn diff_off(&mut self, rel_x: f64, rel_y: f64) {
        self.xoff += rel_x;
        self.yoff += rel_y;
    }

    pub fn diff_scale(&mut self, rel: f64) {
        // Get old center
        let (cx, cy) = self.canvas.output_size().unwrap();

        let midx = cx as f64 / (self.scaling * 2.) - self.xoff;
        let midy = cy as f64 / (self.scaling * 2.) - self.yoff;

        self.scaling *= rel;

        // Get new xoff and yoff
        self.xoff = -midx + cx as f64 / (self.scaling * 2.);
        self.yoff = -midy + cy as f64 / (self.scaling * 2.);
    }

    pub fn present(&mut self, w: &World) -> Result<(), String> {
        let (world_x, world_y) = w.get_size();
        let world_x = world_x as f64;
        let world_y = world_y as f64;

        let (window_x, window_y) = self.canvas.output_size().unwrap();

        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Draw grass
        let scaling = self.scaling;
        let grass_max = w.get_grass_max() as f64;
        w.get_grass_slice()
            .iter()
            .enumerate()
            .try_for_each(|(i, v)| -> Result<(), String> {
                let (x, y) = w.get_grass_loc(i);
                let rg = *v as f64 / grass_max;
                let r = ((1. - rg) * 148.) as u8;

                // Only draw squares in bounds.
                let (canvas_x, canvas_y) = self.map_to_screen(x as f64, y as f64);
                let x_left = canvas_x + (scaling as i32) >= 0;
                let x_right = canvas_x <= window_x as i32;
                let y_top = canvas_y + (scaling as i32) >= 0;
                let y_bot = canvas_y <= window_y as i32;

                if (x_left || x_right) || (y_top || y_bot) {
                    self.canvas.set_draw_color(Color::RGB(r, 94, 0));
                    let r = Rect::new(canvas_x, canvas_y, scaling as u32, scaling as u32);
                    self.canvas.fill_rect(r)?;
                    self.canvas.set_draw_color(Color::RGB(0, 127, 0));
                    self.canvas.draw_rect(r)
                } else {
                    Ok(())
                }
            })?;

        let mut fam_color_hash = HashMap::<usize, (u8, u8, u8)>::new();

        // Draw Creatures
        w.get_creature_slice()
            .iter()
            .try_for_each(|c| -> Result<(), String> {
                let (x, y, theta) = c.get_pos();

                let (canvas_x, canvas_y) = self.map_to_screen(x, y);

                let rad = (self.scaling / 2.) as i32;
                let is_on_screen =
                    self.canvas_on_screen(canvas_x - rad, canvas_y - rad, 2 * rad, 2 * rad);

                if is_on_screen {
                    let fam = c.get_fam();
                    let ent = fam_color_hash.entry(fam);
                    let (r, g, b) = *ent.or_insert_with(|| {
                        let mut s = DefaultHasher::new();
                        fam.hash(&mut s);
                        let hashed_fam = s.finish();
                        let r = hashed_fam & (255);
                        let g = (hashed_fam & (255 << 8)) >> 8;
                        let b = (hashed_fam & (255 << 16)) >> 16;
                        (r as u8, g as u8, b as u8)
                    });

                    let col = Color::RGB(r as u8, g as u8, b as u8);
                    let rad = rad as i16;

                    self.canvas
                        .filled_circle(canvas_x as i16, canvas_y as i16, rad, col)?;

                    let veg_eff = c.get_veg_eff();
                    let g = 255. * veg_eff;
                    let r = 255. * (1. - veg_eff);
                    let diet_col = Color::RGB(r.round() as u8, g.round() as u8, 0);
                    let rad = (scaling / 3.) as i16;
                    self.canvas
                        .filled_circle(canvas_x as i16, canvas_y as i16, rad, diet_col)?;

                    let (theta_s, theta_c) = theta.sin_cos();

                    let (canvas_x_for, canvas_y_for) = self.map_to_screen(x + theta_c, y + theta_s);
                    let start = Point::new(canvas_x, canvas_y);
                    let end = Point::new(canvas_x_for, canvas_y_for);
                    self.canvas.set_draw_color(col);
                    self.canvas.draw_line(start, end)?;
                }

                if self.draw_vision {
                    if let Some(o) = c.get_last_observation() {
                        o.dists
                            .iter()
                            .cloned()
                            .zip(o.colors.iter().cloned())
                            .enumerate()
                            .try_for_each(|(i, (d, c))| {
                                let dangle =
                                    Observation::VISION_RANGE / Observation::NUM_SITES as f64;
                                let angle =
                                    i as f64 * dangle - Observation::VISION_RANGE / 2.0 + theta;
                                let (a_s, a_c) = angle.sin_cos();
                                let (a_ds, a_dc) = (angle + dangle).sin_cos();

                                [-world_x, 0., world_x]
                                    .iter()
                                    .cloned()
                                    .try_for_each(|xoff| {
                                        [-world_y, 0., world_y].iter().cloned().try_for_each(
                                            |yoff| {
                                                let start_x_world = x + xoff + d * a_c;
                                                let start_y_world = y + yoff + d * a_s;
                                                let end_x_world = x + xoff + d * a_dc;
                                                let end_y_world = y + yoff + d * a_ds;

                                                let start_in_x_world =
                                                    start_x_world >= 0. && start_x_world <= world_x;
                                                let start_in_y_world =
                                                    start_y_world >= 0. && start_y_world <= world_y;
                                                let start_in_world =
                                                    start_in_x_world && start_in_y_world;
                                                let end_in_x_world =
                                                    end_x_world >= 0. && end_x_world <= world_x;
                                                let end_in_y_world =
                                                    end_y_world >= 0. && end_y_world <= world_y;
                                                let end_in_world = end_in_x_world && end_in_y_world;
                                                if start_in_world || end_in_world {
                                                    let (startx, starty) = self.map_to_screen(
                                                        start_x_world,
                                                        start_y_world,
                                                    );

                                                    let (endx, endy) = self
                                                        .map_to_screen(end_x_world, end_y_world);

                                                    let start_on_screen =
                                                        self.canvas_on_screen(startx, starty, 0, 0);
                                                    let end_on_screen =
                                                        self.canvas_on_screen(startx, starty, 0, 0);

                                                    if start_on_screen || end_on_screen {
                                                        let start = Point::new(startx, starty);
                                                        let end = Point::new(endx, endy);

                                                        let v = (255. * c).floor() as u8;
                                                        let col = Color::RGB(v, v, v);
                                                        self.canvas.set_draw_color(col);
                                                        self.canvas.draw_line(start, end)
                                                    } else {
                                                        Ok(())
                                                    }
                                                } else {
                                                    Ok(())
                                                }
                                            },
                                        )
                                    })
                            })?;
                    }
                }

                Ok(())
            })?;

        self.canvas.present();

        Ok(())
    }

    #[inline]
    fn map_to_screen(&self, x: f64, y: f64) -> (i32, i32) {
        Self::apply_offsets(x, y, self.xoff, self.yoff, self.scaling)
    }

    #[inline]
    fn apply_offsets(x: f64, y: f64, xoff: f64, yoff: f64, scaling: f64) -> (i32, i32) {
        let canvas_x = ((x + xoff) * scaling) as i32;
        let canvas_y = ((y + yoff) * scaling) as i32;
        (canvas_x, canvas_y)
    }

    #[inline]
    fn phys_on_screen(&self, x: f64, y: f64, w: f64, h: f64) -> bool {
        let (canvas_x, canvas_y) = self.map_to_screen(x, y);
        let (canvas_dx, canvas_dy) = self.map_to_screen(x + w, y + h);
        self.canvas_on_screen(
            canvas_x,
            canvas_y,
            canvas_dx - canvas_x,
            canvas_dy - canvas_y,
        )
    }

    #[inline]
    fn canvas_on_screen(&self, x: i32, y: i32, w: i32, h: i32) -> bool {
        let (window_x, window_y) = self.canvas.output_size().unwrap();
        let x_left = x + w >= 0;
        let x_right = x <= window_x as i32;
        let y_top = y + h >= 0;
        let y_bot = h <= window_y as i32;
        (x_left || x_right) && (y_top || y_bot)
    }

    pub fn get_events(&mut self) -> EventPollIterator {
        self.event_pump.poll_iter()
    }
}
