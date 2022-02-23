use crate::constraint::particle_list::ParticleList;
use crate::constraint::Constraint;
use crate::particle::PRef;
use crate::V2;
use protocol::pr_model::PrConstraint;

#[derive(Clone)]
pub struct VolumeConstraintTemplate {
	pub id: isize,
	pub ps: Vec<usize>,
	pub compliance: f32,
}

// fn area_f(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> f32 {
// 	x1 * y2 + x2 * y3 + x3 * y1 - x3 * y2 - x1 * y3 - x2 * y1
// }

fn area_p(p1: V2, p2: V2, p3: V2) -> f32 {
	p1[0] * p2[1] + p2[0] * p3[1] + p3[0] * p1[1]
		- p3[0] * p2[1]
		- p1[0] * p3[1]
		- p2[0] * p1[1]
}

#[derive(Clone)]
pub struct VolumeConstraint {
	id: isize, // for triangle render, 0 for no render
	ps: ParticleList,
	ps_sort: ParticleList,
	s0: f32,
	lambda: f32,
	compliance: f32,
}

impl VolumeConstraint {
	pub fn new(ps: Vec<PRef>) -> Self {
		let ps_sort = ParticleList::new(ps.clone(), true);
		let ps = ParticleList::new(ps, false);
		let p0 = ps_sort[0].try_lock().unwrap().get_pos();
		let p1 = ps_sort[1].try_lock().unwrap().get_pos();
		let p2 = ps_sort[2].try_lock().unwrap().get_pos();
		let s0 = area_p(p0, p1, p2);
		Self {
			id: -1,
			ps,
			ps_sort,
			s0,
			lambda: 0f32,
			compliance: 1e-9,
		}
	}

	pub fn with_id(mut self, id: isize) -> Self {
		self.id = id;
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

impl Constraint for VolumeConstraint {
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
		let mut p0_mut = self.ps_sort[0].lock().unwrap();
		let mut p1_mut = self.ps_sort[1].lock().unwrap();
		let mut p2_mut = self.ps_sort[2].lock().unwrap();

		let imass0 = p0_mut.get_imass();
		let imass1 = p1_mut.get_imass();
		let imass2 = p2_mut.get_imass();
		let imass = imass0 + imass1 + imass2;
		if imass == 0.0 {
			return;
		}

		let pos0 = p0_mut.get_pos();
		let pos1 = p1_mut.get_pos();
		let pos2 = p2_mut.get_pos();
		let s = area_p(pos0, pos1, pos2);
		let ds = s - self.s0;
		let x0 = pos0[0];
		let x1 = pos1[0];
		let x2 = pos2[0];
		let y0 = pos0[1];
		let y1 = pos1[1];
		let y2 = pos2[1];
		let grad0 = V2::new(y1 - y2, x2 - x1);
		let grad1 = V2::new(y2 - y0, x0 - x2);
		let grad2 = V2::new(y0 - y1, x1 - x0);

		let beta = imass0 * grad0.magnitude_squared()
			+ imass1 * grad1.magnitude_squared()
			+ imass2 * grad2.magnitude_squared();
		let compliance_t = self.compliance / dt.powi(2);
		let dlambda =
			(-ds - compliance_t * self.lambda) / (beta + compliance_t);
		self.lambda += dlambda;
		let correct0 = dlambda * imass0 * grad0;
		let correct1 = dlambda * imass1 * grad1;
		let correct2 = dlambda * imass2 * grad2;

		p0_mut.add_pos(correct0);
		p1_mut.add_pos(correct1);
		p2_mut.add_pos(correct2);
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_area_p() {
		let p0 = V2::new(0., 0.);
		let p1 = V2::new(1., 0.);
		let p2 = V2::new(0., 2.);
		let a0 = area_p(p0, p1, p2);
		let a1 = area_p(p0, p2, p1);
		eprintln!("{} {}", a0, a1);
		assert!((a0.abs() - 2.).abs() < 1e-6);
		assert!((a0 + a1).abs() < 1e-6);
	}
}
