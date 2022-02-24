use crate::constraint::particle_list::ParticleList;
use crate::constraint::{rp, Constraint};
use crate::particle::PRef;
use protocol::pr_model::PrConstraint;

#[derive(Clone)]
pub struct DistanceConstraintTemplate {
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
	ps: ParticleList,
	ps_sort: ParticleList,
	l0: f32,
	lambda: f32,
	compliance: f32,
	plas_thresh: f32,
	plas_cutoff: f32,
	break_range: [f32; 2],
	ty: DCTy,
}

impl DistanceConstraint {
	pub fn new(p1: PRef, p2: PRef) -> Self {
		let pos1 = p1.lock().unwrap().get_pos();
		let pos2 = p2.lock().unwrap().get_pos();
		let l0 = (pos1 - pos2).magnitude();
		Self::new_with_l0(p1, p2, l0)
	}

	pub fn new_with_l0(p1: PRef, p2: PRef, l0: f32) -> Self {
		let ps = vec![p1, p2];
		let ps_sort = ParticleList::new(ps.clone(), true);
		let ps = ParticleList::new(ps, false);
		Self {
			ps,
			ps_sort,
			l0,
			lambda: 0f32,
			compliance: 1e-7,
			plas_thresh: 0.0,
			plas_cutoff: 1.0,
			break_range: [0.0, f32::INFINITY],
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

	pub fn with_plasticity(mut self, thresh: f32, cutoff: f32) -> Self {
		self.plas_thresh = thresh;
		self.plas_cutoff = cutoff * self.l0;
		self
	}

	pub fn with_break_range(mut self, range: [f32; 2]) -> Self {
		self.break_range[0] = range[0] * self.l0;
		self.break_range[1] = range[1] * self.l0;
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
	fn render(&self, id: i32) -> PrConstraint {
		PrConstraint {
			id,
			particles: self.ps.ids(),
		}
	}

	fn pre_iteration(&mut self) -> bool {
		self.lambda = 0f32;
		let p1 = self.ps[0].try_lock().unwrap();
		let p2 = self.ps[1].try_lock().unwrap();

		let dp = p1.get_pos() - p2.get_pos();
		let l = dp.magnitude();
		if self.break_range[0] > l || self.break_range[1] < l {
			return false
		}
		if self.plas_thresh == 0.0 {
			return true;
		}
		let dl = l - self.l0;
		if dl.abs() > self.plas_cutoff {
			self.l0 += dl * self.plas_thresh;
		}
		true
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
		let dp = p1_mut.get_pos() - p2_mut.get_pos();
		let l = dp.magnitude();
		if !l.is_normal() {
			eprintln!("WARN: bad distance {}", l);
			p1_mut.add_pos(rp());
			p2_mut.add_pos(rp());
			return;
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
		let correct = dlambda * dp / l;
		self.lambda += dlambda;

		p1_mut.add_pos(correct * imass1);
		p2_mut.add_pos(-correct * imass2);
	}
}
