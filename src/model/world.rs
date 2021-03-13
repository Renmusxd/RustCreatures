use crate::model::creature::{Creature, Observation};
use crate::model::Action;
use std::cmp::min;

pub struct World {
    creatures: Option<Vec<Creature>>,

    grass_values: Vec<u32>,
    grass_tile_x: usize,
    grass_tile_y: usize,
    grass_max: u32,
    grass_recharge: u32,
}

const GRASS_MAX: u32 = 127;

impl World {
    pub fn new(x: usize, y: usize) -> Self {
        let grass_max = GRASS_MAX;
        World {
            creatures: Some(vec![]),
            grass_values: vec![grass_max; x * y],
            grass_tile_x: x,
            grass_tile_y: y,
            grass_max,
            grass_recharge: 1,
        }
    }

    pub fn add_creature(&mut self, c: Creature) {
        self.creatures.as_mut().unwrap().push(c)
    }

    pub fn get_creature_slice(&self) -> &[Creature] {
        self.creatures.as_ref().unwrap()
    }

    pub fn get_grass_slice(&self) -> &[u32] {
        &self.grass_values
    }

    pub fn get_grass_loc(&self, i: usize) -> (usize, usize) {
        let x = i % self.grass_tile_x;
        let y = i / self.grass_tile_y;
        (x, y)
    }

    pub fn get_grass_max(&self) -> u32 {
        self.grass_max
    }

    pub fn update(&mut self) {
        // Update grass
        let grass_recharge = self.grass_recharge;
        let grass_max = self.grass_max;
        self.grass_values
            .iter_mut()
            .for_each(|g| *g = min(*g + grass_recharge, grass_max));

        // Update creatures
        let mut cs = self.creatures.take().unwrap();
        let observations = cs.iter().map(|c| self.observe(c, &cs)).collect::<Vec<_>>();

        cs.iter_mut()
            .zip(observations.into_iter())
            .for_each(|(c, o)| {
                let mut actions = c.get_preferred_actions(o);
                let (_, _, t) = c.get_pos();
                let (ts, tc) = t.sin_cos();
                while let Some(action) = actions.pop() {
                    match action {
                        Action::WAIT => break,
                        Action::EAT => unimplemented!(),
                        Action::BITE => unimplemented!(),
                        Action::REPLICATE => unimplemented!(),
                        Action::FORWARD => {
                            let (x, y) = c.get_pos_mut();
                            *x = *x + tc;
                            *y = *y + ts;
                        }
                        Action::LEFT => {
                            c.set_theta(t + 0.01);
                        }
                        Action::RIGHT => {
                            c.set_theta(t - 0.01);
                        }
                    }
                }
            });

        self.creatures = Some(cs);
    }

    pub fn observe(&self, c: &Creature, cs: &[Creature]) -> Observation {
        let (x, y, theta) = c.get_pos();
        let mut observation = Observation::new_empty();
        let max_d2 = Observation::MAX_DIST.powi(2);
        cs.iter()
            .filter(|oc| oc.get_id() != c.get_id())
            .for_each(|c| {
                let (cx, cy, _) = c.get_pos();
                let dist2 = (x - cx).powi(2) + (y - cy).powi(2);
                if dist2 < max_d2 {
                    let abs_dtheta = (cy - y).atan2(cx - x);
                    let dtheta =
                        (abs_dtheta - theta + std::f64::consts::TAU) % std::f64::consts::TAU;

                    if dtheta < Observation::VISION_RANGE {
                        let d_from_left = (dtheta / Observation::VISION_RANGE) + 0.5;
                        let soft_bin = Observation::NUM_SITES as f64 * d_from_left;
                        let bin = soft_bin.floor() as usize;

                        let d2 = (x - cx).powi(2) + (y - cy).powi(2);
                        if d2 < observation.dists[bin] {
                            observation.dists[bin] = d2;
                            observation.colors[bin] = c.get_color();
                        }
                    }
                }
            });

        // Any which are still infinity should be changed to grass colors.
        observation
            .dists
            .iter_mut()
            .zip(observation.colors.iter_mut())
            .enumerate()
            .filter(|(_, (d, _))| d.is_infinite())
            .for_each(|(i, (d, c))| {
                // TODO observe grass.
                *d = max_d2;
                *c = 0.0;
            });

        // Change to square roots.
        observation.dists.iter_mut().for_each(|d| *d = d.sqrt());

        observation
    }

    pub fn num_creatures(&self) -> usize {
        self.creatures.as_ref().unwrap().len()
    }
}
