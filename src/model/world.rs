use crate::model::creature::{Creature, Observation};
use crate::model::{Action, MovementAction, TurningAction};
use crate::util::gridlookup::GridLookup;
use ndarray_rand::rand::Rng;
use rayon::prelude::*;
use std::cmp::min;
use std::collections::HashSet;

pub struct World {
    creatures: Option<Vec<Creature>>,
    gridlookup: GridLookup<usize>,

    grass_values: Vec<u32>,
    grass_tile_x: usize,
    grass_tile_y: usize,
    grass_max: u32,
    grass_recharge: u32,

    creature_id: usize,
    min_pop: usize,
    min_fams: usize,
}

const GRASS_MAX: u32 = 512;
const BITE_DIST: f64 = 1.0;
const BITE_DIST_2: f64 = BITE_DIST * BITE_DIST;
const BITE_DAMAGE: u32 = Creature::STARTING_ENERGY;
const GRASS_EAT_FRAC: f64 = 0.5;
const TURN_SPEED: f64 = 0.01;
const WALK_SPEED: f64 = 0.02;

const CREATURE_ENERGY_COST: u32 = 1;
const CREATURE_WALK_ENERGY_COST: u32 = 3;

const MAX_AGE: u32 = 60000;

impl World {
    pub fn new(x: usize, y: usize, min_pop: usize) -> Self {
        let grass_max = GRASS_MAX;
        let xstep = Observation::MAX_DIST as f64;
        let ystep = xstep;
        World {
            creatures: Some(vec![]),
            gridlookup: GridLookup::new(x as f64, y as f64, xstep, ystep),
            grass_values: vec![grass_max; x * y],
            grass_tile_x: x,
            grass_tile_y: y,
            grass_max,
            grass_recharge: 1,
            creature_id: 0,
            min_pop,
            min_fams: 5,
        }
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.grass_tile_x, self.grass_tile_y)
    }

    pub fn get_inc_creature_id(&mut self) -> usize {
        let t = self.creature_id;
        self.creature_id += 1;
        t
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

    pub fn get_grass_index(&self, x: usize, y: usize) -> usize {
        y * self.grass_tile_x + x
    }

    pub fn get_grass_loc(&self, i: usize) -> (usize, usize) {
        let x = i % self.grass_tile_x;
        let y = i / self.grass_tile_y;
        (x, y)
    }

    pub fn get_grass_max(&self) -> u32 {
        self.grass_max
    }

    pub fn update<R: Rng>(&mut self, mut rng: R) {
        let world_x = self.grass_tile_x as f64;
        let world_y = self.grass_tile_y as f64;

        // Update grass
        let grass_recharge = self.grass_recharge;
        let grass_max = self.grass_max;
        self.grass_values
            .par_iter_mut()
            .for_each(|g| *g = min(*g + grass_recharge, grass_max));

        // Update creatures
        let mut cs = self.creatures.take().unwrap();

        cs = cs
            .into_par_iter()
            .map(|mut c| {
                c.remove_energy(CREATURE_ENERGY_COST);
                c.inc_age();
                c
            })
            .filter(|c| (c.get_energy() > 0) && (c.get_age() < MAX_AGE))
            .collect();

        cs.iter().enumerate().for_each(|(indx, c)| {
            let (x, y, _) = c.get_pos();
            self.gridlookup.put((x, y), indx);
        });

        let observations = cs
            .par_iter()
            .map(|c| self.observe(c, &cs, &self.gridlookup))
            .collect::<Vec<_>>();
        let actions: Vec<(TurningAction, MovementAction, Action)> = cs
            .par_iter_mut()
            .zip(observations.into_par_iter())
            .map(|(c, o)| c.get_preferred_action(o))
            .collect();

        cs.par_iter_mut()
            .zip(actions.par_iter())
            .for_each(|(c, (turn_act, move_act, _))| {
                let (_, _, t) = c.get_pos();
                let (ts, tc) = t.sin_cos();
                match move_act {
                    MovementAction::WAIT => {}
                    MovementAction::FORWARD => {
                        let (x, y) = c.get_pos_mut();
                        *x += WALK_SPEED * tc;
                        *y += WALK_SPEED * ts;
                        if *x < 0. {
                            *x = world_x - 0.001;
                        }
                        if *x > world_x {
                            *x = 0.;
                        }
                        if *y < 0. {
                            *y = world_y - 0.001;
                        }
                        if *y > world_y {
                            *y = 0.;
                        }
                        c.remove_energy(CREATURE_WALK_ENERGY_COST);
                    }
                };

                match turn_act {
                    TurningAction::WAIT => {}
                    TurningAction::LEFT => c.set_theta(t + TURN_SPEED),
                    TurningAction::RIGHT => c.set_theta(t - TURN_SPEED),
                };
            });

        let mut creatures_to_add = vec![];
        (0..cs.len())
            .zip(actions.into_iter())
            .for_each(|(ic, (_, _, action))| {
                let c = &cs[ic];
                let (x, y, theta) = c.get_pos();
                match action {
                    Action::WAIT => {}
                    Action::EAT => {
                        let grass_x = x.floor() as usize % self.grass_tile_x;
                        let grass_y = y.floor() as usize % self.grass_tile_y;
                        let indx = self.get_grass_index(grass_x, grass_y);
                        let grass = self.grass_values[indx];
                        let to_eat = (grass as f64 * GRASS_EAT_FRAC).round() as u32;
                        self.grass_values[indx] -= to_eat;

                        let digested = (to_eat as f64 * cs[ic].get_veg_eff()).round() as u32;
                        cs[ic].add_energy(digested);
                    }
                    Action::REPLICATE => {
                        if c.get_energy() > Creature::STARTING_ENERGY * 4 {
                            cs[ic].remove_energy(3 * Creature::STARTING_ENERGY);

                            let mut newc = cs[ic].clone_mutate(self.get_inc_creature_id());

                            let rev_t = (theta + std::f64::consts::PI) % std::f64::consts::TAU;
                            let (dy, dx) = rev_t.sin_cos();
                            let dx = dx * 10. * WALK_SPEED;
                            let dy = dy * 10. * WALK_SPEED;
                            let (cx, cy) = newc.get_pos_mut();
                            *cx += dx;
                            *cy += dy;
                            newc.set_theta(rev_t);
                            creatures_to_add.push(newc)
                        }
                    }
                    Action::BITE => {
                        const VISION_RANGE_2: f64 = Observation::VISION_RANGE / 2.;

                        self.gridlookup
                            .get_within_step((x, y), &mut cs, |cs, (cx, cy, t)| {
                                let t = *t;
                                if cs[t].get_id() != cs[ic].get_id() {
                                    let d2 = (x - cx).powi(2) + (y - cy).powi(2);
                                    if d2 <= BITE_DIST_2 {
                                        let abs_dtheta = (cy - y).atan2(cx - x);
                                        let dtheta = (abs_dtheta - (theta - VISION_RANGE_2)
                                            + 2. * std::f64::consts::TAU)
                                            % std::f64::consts::TAU;
                                        if dtheta < Observation::VISION_RANGE {
                                            let meat_eff = 1. - cs[ic].get_veg_eff();
                                            let dam =
                                                (meat_eff * BITE_DAMAGE as f64).round() as u32;
                                            let removed = cs[t].remove_energy(dam);
                                            let digested =
                                                (removed as f64 * meat_eff).round() as u32;
                                            cs[ic].add_energy(digested);
                                        }
                                    }
                                }
                                cs
                            });
                    }
                }
            });
        cs.extend(creatures_to_add.into_iter());

        while cs.len() < self.min_pop {
            let id = self.get_inc_creature_id();
            let x = rng.gen_range(0. ..self.grass_tile_x as f64);
            let y = rng.gen_range(0. ..self.grass_tile_y as f64);
            let t = rng.gen_range(0. ..std::f64::consts::TAU);

            let c = Creature::new(id, id, x, y, t, rng.gen_range(0. ..1.));
            cs.push(c);
        }

        if self.min_fams > 0 {
            let mut set = HashSet::new();
            set.extend(cs.iter().map(|c| c.get_fam()));
            while set.len() < self.min_fams {
                let id = self.get_inc_creature_id();
                let x = rng.gen_range(0. ..self.grass_tile_x as f64);
                let y = rng.gen_range(0. ..self.grass_tile_y as f64);
                let t = rng.gen_range(0. ..std::f64::consts::TAU);

                let c = Creature::new(id, id, x, y, t, rng.gen_range(0. ..1.));
                set.insert(c.get_fam());
                cs.push(c);
            }
        }

        self.gridlookup.clear();
        self.creatures = Some(cs);
    }

    pub fn observe(&self, c: &Creature, cs: &[Creature], grid: &GridLookup<usize>) -> Observation {
        const MAX_D2: f64 = Observation::MAX_DIST * Observation::MAX_DIST;
        const VISION_RANGE_2: f64 = Observation::VISION_RANGE / 2.;

        let (x, y, theta) = c.get_pos();
        let mut observation = Observation::new_empty();

        observation.energy = c.get_energy() as f64 / (Creature::STARTING_ENERGY as f64);

        let mut observation =
            grid.get_within_step((x, y), observation, |mut observation, (cx, cy, t)| {
                let d2 = (x - cx).powi(2) + (y - cy).powi(2);
                if d2 <= MAX_D2 {
                    let oc = &cs[*t];
                    if oc.get_id() != c.get_id() {
                        let abs_dtheta = (cy - y).atan2(cx - x);
                        let dtheta = (abs_dtheta - (theta - VISION_RANGE_2)
                            + 2. * std::f64::consts::TAU)
                            % std::f64::consts::TAU;

                        if dtheta < Observation::VISION_RANGE {
                            let d_from_left = dtheta / Observation::VISION_RANGE;
                            let soft_bin = Observation::NUM_SITES as f64 * d_from_left;
                            let bin = soft_bin.floor() as usize;

                            let d2 = (x - cx).powi(2) + (y - cy).powi(2);
                            if d2 < observation.dists[bin] {
                                observation.dists[bin] = d2;
                                observation.colors[bin] = oc.get_color();
                            }
                        };
                    }
                }
                observation
            });

        observation
            .dists
            .iter_mut()
            .zip(observation.colors.iter_mut())
            .filter(|(d, _)| d.is_infinite())
            .for_each(|(d, c)| {
                *d = MAX_D2;
                *c = 0.0;
            });

        // Change to square roots.
        observation.dists.iter_mut().for_each(|d| *d = d.sqrt());

        // Neighboring grass
        let gx = x.floor() as usize;
        let gy = y.floor() as usize;
        (0..Observation::GRASS_NEIGHBORS).for_each(|ix| {
            (0..Observation::GRASS_NEIGHBORS).for_each(|iy| {
                let mid = Observation::GRASS_NEIGHBORS / 2;
                // Avoid underflow.
                let sel_grass_x = (self.grass_tile_x + gx + ix - mid) % self.grass_tile_x;
                let sel_grass_y = (self.grass_tile_y + gy + iy - mid) % self.grass_tile_y;
                let grass_indx = self.get_grass_index(sel_grass_x, sel_grass_y);
                observation.neighboring_grass[Observation::neighbor_index(ix, iy)] =
                    self.grass_values[grass_indx] as f64 / GRASS_MAX as f64;
            })
        });

        observation
    }

    pub fn num_creatures(&self) -> usize {
        self.creatures.as_ref().unwrap().len()
    }
}
