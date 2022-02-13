use crate::particle::PRef;
use crate::constraint::Constraint;
use crate::V2;

pub struct DistanceConstraint {
	p1: PRef,
	p2: PRef,
	l0: f32,
}

impl DistanceConstraint {
	pub fn new_constraint(p1: PRef, p2: PRef) -> Box<dyn Constraint> {
		let pos1 = p1.borrow().get_pos();
		let pos2 = p2.borrow().get_pos();
		let result = Self {
			p1,
			p2,
			l0: (pos1 - pos2).magnitude(),
		};
		Box::new(result)
	}
}

impl Constraint for DistanceConstraint {
	fn step(&mut self, _dt: f32) {
		let mut p1_mut = self.p1.borrow_mut();
		let mut p2_mut = self.p2.borrow_mut();
		let imass1 = p1_mut.get_imass();
		let imass2 = p2_mut.get_imass();
		let imass = imass1 + imass2;
		if imass == 0.0 { return }
		let mut dp = p1_mut.get_pos() - p2_mut.get_pos();
		let l = dp.magnitude();
		if l.abs() < f32::EPSILON {
			eprintln!("Bad value");
			dp = V2::new(0.0, 1.0);
		}
		let dl = l - self.l0;
		let correct = -0.1 * dp.normalize() * dl / imass;

		p1_mut.add_pos(correct * imass1);
		p2_mut.add_pos(-correct * imass2);
	}
}
