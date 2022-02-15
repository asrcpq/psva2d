pub mod distance;
pub mod volume;

pub trait Constraint {
	fn step(&mut self, dt: f32);

	fn reset_lambda(&mut self);
}
