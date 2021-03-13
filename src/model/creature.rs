// Contains information about the world as seen at a snapshot.

const NUM_SITES: usize = 5;
const MAX_DIST: f64 = 20.;
#[derive(Clone, Debug)]
pub struct Observation {
    // Colors and dists (0-1) for 0-MAX_DIST
    pub colors: [f64; NUM_SITES],
    pub dists: [f64; NUM_SITES],
}

impl Observation {
    pub const NUM_SITES: usize = NUM_SITES;
    pub const MAX_DIST: f64 = MAX_DIST;
    pub const VISION_RANGE: f64 = std::f64::consts::FRAC_PI_4;

    pub fn new_empty() -> Self {
        Self {
            colors: [0.; Self::NUM_SITES],
            dists: [std::f64::INFINITY; Self::NUM_SITES],
        }
    }
}

pub enum Action {
    WAIT,
    EAT,
    BITE,
    REPLICATE,
    FORWARD,
    LEFT,
    RIGHT,
}

pub struct Creature {
    id: usize,
    x: f64,
    y: f64,
    theta: f64,
    color: f64,
    last_obs: Option<Observation>,
}

pub enum DietEnum {
    VEGETARIAN,
    CARNIVORE,
}

impl Creature {
    pub fn new(id: usize, x: f64, y: f64, theta: f64) -> Self {
        Self {
            id,
            x,
            y,
            theta,
            color: 0.0,
            last_obs: None,
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
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

    pub fn get_diet(&self) -> DietEnum {
        DietEnum::VEGETARIAN
    }

    pub fn get_color(&self) -> f64 {
        self.color
    }

    pub fn get_preferred_actions(&mut self, o: Observation) -> Vec<Action> {
        self.last_obs = Some(o);
        // TODO get preferred actions.
        vec![Action::RIGHT]
    }
}
