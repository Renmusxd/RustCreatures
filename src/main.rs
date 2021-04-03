extern crate blas_src;

mod controller;
mod model;
mod util;
mod view;

fn main() {
    let w = model::World::new(100, 100, 20);
    let v = view::View::new();
    let mut c = controller::Controller::new(w, v);
    c.run_loop();
}
