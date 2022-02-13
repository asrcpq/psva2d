use crate::particle::PRef;
use crate::constraint::Constraint;

pub struct DistanceConstraint {
	p1: PRef,
	p2: PRef,
}

impl DistanceConstraint {
	pub fn new_constraint(p1: PRef, p2: PRef) -> Box<dyn Constraint> {
		let result = Self {
			p1,
			p2,
		};
		Box::new(result)
	}
}

impl Constraint for DistanceConstraint {
	fn step(&mut self, dt: f32) {
		eprintln!("step");
	}
}
