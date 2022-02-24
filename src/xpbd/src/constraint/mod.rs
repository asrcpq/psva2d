pub mod constraint_template;
pub mod distance;
pub mod leash;
pub mod volume;

mod particle_list;

use crate::particle::PRef;
use crate::V2;
use protocol::pr_model::PrConstraint;

pub type CRef = Box<dyn Constraint>;

pub trait Constraint: dyn_clone::DynClone + Send {
	fn pre_iteration(&mut self);
	fn step(&mut self, dt: f32);
	fn render(&self, id: i32) -> PrConstraint;
}

dyn_clone::clone_trait_object!(Constraint);

pub fn rp() -> V2 {
	use rand::prelude::*;
	let dx = rand::thread_rng().gen::<f32>() / 1e5;
	let dy = rand::thread_rng().gen::<f32>() / 1e5;
	V2::new(dx, dy)
}
