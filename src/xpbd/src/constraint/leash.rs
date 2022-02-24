use crate::constraint::Constraint;
use crate::particle::PRef;
use crate::V2;
use protocol::pr_model::PrConstraint;

#[derive(Clone)]
pub struct LeashConstraint {
	p: PRef,
	pos: V2,
	lambda: f32,
	compliance: f32,
}

impl LeashConstraint {
	pub fn new(p: PRef) -> Self {
		let pos = p.try_lock().unwrap().get_pos();
		Self::new_with_pos(p, pos)
	}

	pub fn new_with_pos(p: PRef, pos: V2) -> Self {
		Self {
			p,
			pos,
			lambda: 0.,
			compliance: 1e-8,
		}
	}

	pub fn with_compliance(mut self, c: f32) -> Self {
		self.compliance = c;
		self
	}

	pub fn build(self) -> Box<dyn Constraint> {
		Box::new(self)
	}
}

impl Constraint for LeashConstraint {
	fn render(&self, id: i32) -> PrConstraint {
		PrConstraint {
			id,
			particles: vec![self.p.try_lock().unwrap().get_id()],
		}
	}

	fn pre_iteration(&mut self) {
		self.lambda = 0f32;
	}

	fn step(&mut self, dt: f32) {
		let mut p = self.p.lock().unwrap();
		let imass = p.get_imass();
		if imass == 0.0 {
			return;
		}
		let dp = p.get_pos() - self.pos;
		let dl = dp.magnitude();
		let compliance_t = self.compliance / dt.powi(2);
		let dlambda =
			(-dl - compliance_t * self.lambda) / (imass + compliance_t);
		let correct = dlambda * dp / dl;
		self.lambda += dlambda;
		p.add_pos(correct * imass);
	}
}
