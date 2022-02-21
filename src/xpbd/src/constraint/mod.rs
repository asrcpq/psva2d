pub mod constraint_template;
pub mod distance;
pub mod volume;

mod particle_list;

use crate::particle::PRef;
use protocol::pr_model::PrConstraint;

pub trait Constraint: dyn_clone::DynClone + Send {
	fn pre_iteration(&mut self);
	fn step(&mut self, dt: f32);
	fn render(&self) -> PrConstraint;
}

dyn_clone::clone_trait_object!(Constraint);
