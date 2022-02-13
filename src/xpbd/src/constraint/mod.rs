pub mod distance_constraint;

pub trait Constraint {
	fn step(&mut self, dt: f32);
}
