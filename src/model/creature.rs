use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

// Contains information about the world as seen at a snapshot.

use crate::model::brain::*;
use ndarray_rand::rand;
use rand_distr::{Distribution, Normal, NormalError};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const NUM_SITES: usize = 5;
const MAX_DIST: f64 = 20.;
const GRASS_NEIGHBORS: usize = 3;

#[derive(Clone, Debug)]
pub struct Observation {
    // Colors and dists (0-1) for 0-MAX_DIST
    pub colors: [f64; NUM_SITES],
    pub dists: [f64; NUM_SITES],
    pub neighboring_grass: [f64; GRASS_NEIGHBORS * GRASS_NEIGHBORS],
    pub energy: f64,
}

impl Observation {
    pub const NUM_SITES: usize = NUM_SITES;
    pub const MAX_DIST: f64 = MAX_DIST;
    pub const VISION_RANGE: f64 = std::f64::consts::FRAC_PI_2;
    pub const GRASS_NEIGHBORS: usize = GRASS_NEIGHBORS;
    pub const NUM_NEIGHBORS: usize = Self::GRASS_NEIGHBORS * Self::GRASS_NEIGHBORS;

    pub const NUM_INPUTS: usize = 2 * Self::NUM_SITES + Self::NUM_NEIGHBORS + 1;

    pub fn new_empty() -> Self {
        Self {
            colors: [0.; Self::NUM_SITES],
            dists: [std::f64::INFINITY; Self::NUM_SITES],
            neighboring_grass: [0.; Self::NUM_NEIGHBORS],
            energy: 0.0,
        }
    }

    pub fn neighbor_index(ix: usize, iy: usize) -> usize {
        iy * Self::GRASS_NEIGHBORS + ix
    }

    pub fn inputs(&self) -> [f64; Self::NUM_INPUTS] {
        let mut inputs = [0.; Self::NUM_INPUTS];
        (0..Self::NUM_SITES).for_each(|i| {
            inputs[i] = self.colors[i];
        });
        (0..Self::NUM_SITES).for_each(|i| {
            inputs[Self::NUM_SITES + i] = self.dists[i];
        });
        (0..Self::GRASS_NEIGHBORS * Self::GRASS_NEIGHBORS).for_each(|i| {
            inputs[2 * Self::NUM_SITES + i] = self.neighboring_grass[i];
        });
        inputs[2 * Self::NUM_SITES + Self::NUM_NEIGHBORS] = self.energy;

        inputs
    }
}

#[derive(FromPrimitive, Debug)]
pub enum TurningAction {
    WAIT,
    LEFT,
    RIGHT,
}

#[derive(FromPrimitive, Debug)]
pub enum MovementAction {
    WAIT,
    FORWARD,
}

#[derive(FromPrimitive, Debug)]
pub enum Action {
    WAIT,
    EAT,
    BITE,
    REPLICATE,
}

impl TurningAction {
    pub const NUM_ACTIONS: usize = 3;
}
impl MovementAction {
    pub const NUM_ACTIONS: usize = 2;
}
impl Action {
    pub const NUM_ACTIONS: usize = 4;
}

const TOTAL_ACTIONS: usize =
    { TurningAction::NUM_ACTIONS + MovementAction::NUM_ACTIONS + Action::NUM_ACTIONS };

pub struct Creature {
    id: usize,
    fam: usize,
    x: f64,
    y: f64,
    theta: f64,
    color: f64,
    last_obs: Option<Observation>,
    energy: u32,
    // Vegetable efficiency
    veg_eff: f64,
    brain: NeuralBrain<{ Observation::NUM_INPUTS }, { TOTAL_ACTIONS }>,
    age: u32,
}

impl Creature {
    pub const STARTING_ENERGY: u32 = 4096;
    pub const MUT_RATE: f64 = 0.05;

    pub fn new(id: usize, fam: usize, x: f64, y: f64, theta: f64, veg: f64) -> Self {
        let mut s = DefaultHasher::new();
        fam.hash(&mut s);
        let hash = s.finish();
        let c = hash as f64 / std::u64::MAX as f64;

        Self {
            id,
            fam,
            x,
            y,
            theta,
            color: c,
            last_obs: None,
            energy: Self::STARTING_ENERGY,
            veg_eff: veg,
            brain: NeuralBrain::default(),
            age: 0,
        }
    }

    pub fn clone_mutate(&self, new_id: usize) -> Self {
        let newbrain = self.brain.clone_mutate(Self::MUT_RATE);

        // Tweak veg mut between 0 and 1
        let veg_logit = ((1. / self.veg_eff) - 1.).ln();

        let mut rng = rand::thread_rng();
        let normal = Normal::new(0., Self::MUT_RATE).unwrap();
        let v = normal.sample(&mut rng);
        let veg_logit = veg_logit + v;
        let new_veg_eff = 1. / (1. + veg_logit.exp());

        Self {
            id: new_id,
            fam: self.fam,
            x: self.x,
            y: self.y,
            theta: self.theta,
            color: self.color,
            last_obs: None,
            energy: Self::STARTING_ENERGY,
            veg_eff: new_veg_eff,
            brain: newbrain,
            age: 0,
        }
    }

    pub fn get_age(&self) -> u32 {
        self.age
    }

    pub fn inc_age(&mut self) -> u32 {
        self.age += 1;
        self.age
    }

    pub fn get_veg_eff(&self) -> f64 {
        self.veg_eff
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
    pub fn get_fam(&self) -> usize {
        self.fam
    }

    pub fn get_last_observation(&self) -> Option<&Observation> {
        self.last_obs.as_ref()
    }

    pub fn get_pos(&self) -> (f64, f64, f64) {
        (self.x, self.y, self.theta)
    }

    pub fn get_pos_mut(&mut self) -> (&mut f64, &mut f64) {
        (&mut self.x, &mut self.y)
    }

    pub fn set_theta(&mut self, theta: f64) {
        self.theta = theta % std::f64::consts::TAU;
        while self.theta < 0. {
            self.theta += std::f64::consts::TAU;
        }
    }

    pub fn get_color(&self) -> f64 {
        self.color
    }

    pub fn get_preferred_action(
        &mut self,
        o: Observation,
    ) -> (TurningAction, MovementAction, Action) {
        // Get all inputs starting at 0 (up to 1 for most, above for others like energy).
        let inputs = o.inputs();
        let mut actions = [0.; TOTAL_ACTIONS];

        self.brain.feed(&inputs, &mut actions);

        let i = TurningAction::NUM_ACTIONS;
        let (turning_index, _) = max_index(std::f64::MIN, actions[0..i].iter().cloned());
        let j = TurningAction::NUM_ACTIONS + MovementAction::NUM_ACTIONS;
        let (moving_index, _) = max_index(std::f64::MIN, actions[i..j].iter().cloned());
        let k = TOTAL_ACTIONS;
        let (action_index, _) = max_index(std::f64::MIN, actions[j..k].iter().cloned());

        let turn_action = TurningAction::from_usize(turning_index).unwrap();
        let move_action = MovementAction::from_usize(moving_index).unwrap();
        let action = Action::from_usize(action_index).unwrap();

        self.last_obs = Some(o);
        (turn_action, move_action, action)
    }

    pub fn get_energy(&self) -> u32 {
        self.energy
    }

    pub fn add_energy(&mut self, energy: u32) {
        self.energy += energy;
    }

    pub fn remove_energy(&mut self, energy: u32) -> u32 {
        if self.energy > energy {
            self.energy -= energy;
            energy
        } else {
            let tmp = self.energy;
            self.energy = 0;
            tmp
        }
    }
}

fn max_index<It: IntoIterator<Item = T>, T: PartialOrd + Copy>(init: T, it: It) -> (usize, T) {
    it.into_iter()
        .enumerate()
        .fold((0, init), |(maxindx, maxv), (indx, v)| {
            if v > maxv {
                (indx, v)
            } else {
                (maxindx, maxv)
            }
        })
}
