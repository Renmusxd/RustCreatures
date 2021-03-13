use crate::model::creature::Creature;

mod controller;
mod model;
mod view;

fn main() {
    let mut w = model::World::new(100, 100);
    w.add_creature(Creature::new(0, 25., 25., 0.));
    w.add_creature(Creature::new(1, 35., 25., -std::f64::consts::PI - 0.1));
    let v = view::View::new();
    let mut c = controller::Controller::new(w, v);
    c.run_loop();
}
