pub mod distance;
pub mod volume;

pub trait Constraint: Send {
	fn pre_iteration(&mut self);
	fn step(&mut self, dt: f32);
}
