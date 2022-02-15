use crate::constraint::Constraint;
use crate::constraint::ParticleList;
use crate::particle::PRef;
use crate::V2;

pub struct DistanceConstraint {
	ps: ParticleList,
	l0: f32,
	lambda: f32,
	compliance: f32,
}

impl DistanceConstraint {
	pub fn new(p1: PRef, p2: PRef) -> Self {
		let ps = ParticleList::new(vec![p1, p2]);
		let pos1 = ps[0].lock().unwrap().get_pos();
		let pos2 = ps[1].lock().unwrap().get_pos();
		Self {
			ps,
			l0: (pos1 - pos2).magnitude(),
			lambda: 0f32,
			compliance: 1e-7,
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

impl Constraint for DistanceConstraint {
	fn pre_iteration(&mut self) {
		self.lambda = 0f32;
	}

	fn step(&mut self, dt: f32) {
		let mut p1_mut = self.ps[0].lock().unwrap();
		let mut p2_mut = self.ps[1].lock().unwrap();
		let imass1 = p1_mut.get_imass();
		let imass2 = p2_mut.get_imass();
		let imass = imass1 + imass2;
		if imass == 0.0 {
			return;
		}
		let mut dp = p1_mut.get_pos() - p2_mut.get_pos();
		let l = dp.magnitude();
		if l.abs() < f32::EPSILON {
			eprintln!("Bad value");
			dp = V2::new(0.0, 1.0);
		}
		let dl = l - self.l0;
		// note: efficiency can be improved
		let compliance_t = self.compliance / dt.powi(2);
		let dlambda =
			(-dl - compliance_t * self.lambda) / (imass + compliance_t);
		let correct = dlambda * dp.normalize();
		self.lambda += dlambda;

		p1_mut.add_pos(correct * imass1);
		p2_mut.add_pos(-correct * imass2);
	}
}
