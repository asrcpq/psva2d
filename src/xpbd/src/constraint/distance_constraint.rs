use crate::particle::PRef;

pub struct DistanceConstraint {
	p1: PRef,
	p2: PRef,
}

impl DistanceConstraint {
	pub fn new(p1: PRef, p2: PRef) -> Self {
		Self {
			p1,
			p2,
		}
	}

	pub fn step(&mut self, dt: f32) { }
}
