use crate::constraint::particle_list::ParticleList;
use crate::constraint::Constraint;
use crate::particle::PRef;
use crate::V2;
use protocol::pr_model::PrConstraint;

#[derive(Clone)]
pub struct DistanceConstraintTemplate {
	pub id: isize,
	pub ps: Vec<usize>,
	pub l0: f32,
	pub compliance: f32,
	pub ty: DCTy,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DistanceConstraintType {
	Normal,
	Repulsive, // collision
	Attractive,
}
type DCTy = DistanceConstraintType;

#[derive(Clone)]
pub struct DistanceConstraint {
	id: isize,
	ps: ParticleList,
	ps_sort: ParticleList,
	l0: f32,
	lambda: f32,
	compliance: f32,
	ty: DCTy,
}

impl DistanceConstraint {
	pub fn new(p1: PRef, p2: PRef) -> Self {
		let pos1 = p1.lock().unwrap().get_pos();
		let pos2 = p2.lock().unwrap().get_pos();
		let l0 = (pos1 - pos2).magnitude();
		Self::new_with_l0(p1, p2, l0)
	}

	pub fn with_id(mut self, id: isize) -> Self {
		self.id = id;
		self
	}

	pub fn new_with_l0(p1: PRef, p2: PRef, l0: f32) -> Self {
		let ps = vec![p1, p2];
		let ps_sort = ParticleList::new(ps.clone(), true);
		let ps = ParticleList::new(ps, false);
		Self {
			id: -1,
			ps,
			ps_sort,
			l0,
			lambda: 0f32,
			compliance: 1e-7,
			ty: DCTy::Normal,
		}
	}

	pub fn repulsive_only(mut self) -> Self {
		self.ty = DCTy::Repulsive;
		self
	}

	pub fn attractive_only(mut self) -> Self {
		self.ty = DCTy::Attractive;
		self
	}

	pub fn with_ty(mut self, ty: DCTy) -> Self {
		self.ty = ty;
		self
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
	fn render(&self) -> PrConstraint {
		PrConstraint {
			id: self.id,
			particles: self.ps.ids(),
		}
	}

	fn pre_iteration(&mut self) {
		self.lambda = 0f32;
	}

	fn step(&mut self, dt: f32) {
		let mut p1_mut = self.ps_sort[0].lock().unwrap();
		let mut p2_mut = self.ps_sort[1].lock().unwrap();
		let imass1 = p1_mut.get_imass();
		let imass2 = p2_mut.get_imass();
		let imass = imass1 + imass2;
		if imass == 0.0 {
			return;
		}
		let mut dp = p1_mut.get_pos() - p2_mut.get_pos();
		let l = dp.magnitude();
		if l.abs() < f32::EPSILON {
			eprintln!("Dup point detected in distance constraint!");
			dp = V2::new(0.0, 1.0);
		}
		let dl = l - self.l0;
		if self.ty == DCTy::Repulsive && dl >= 0.
			|| self.ty == DCTy::Attractive && dl <= 0.
		{
			return;
		}
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
