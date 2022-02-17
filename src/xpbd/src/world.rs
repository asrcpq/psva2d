use protocol::pr_model::PrModel;
use protocol::pr_model::PrConstraint;

use crate::constraint::distance::DistanceConstraint;
use crate::constraint::volume::VolumeConstraint;
use crate::constraint::Constraint;
use crate::particle_group::ParticleGroup;
use crate::V2;

pub struct World {
	pg: ParticleGroup,
	constraints: Vec<Box<dyn Constraint>>,
	tmp_constraints: Vec<Box<dyn Constraint>>,
}

impl Default for World {
	fn default() -> Self {
		let pg = ParticleGroup::default();
		Self {
			pg,
			constraints: Vec::new(),
			tmp_constraints: Vec::new(),
		}
	}
}

impl World {
	pub fn init_test(&mut self) {
		self.pg = Default::default();
		self.constraints = Default::default();
		for m in 0..5 {
			for n in 0..3 {
				let x = -5.0 + 2.0 * m as f32;
				let y = 0.5 + 1.0 * n as f32 + 0.5 * (m % 2) as f32;
				self.add_test_block(
					x,
					y,
					25,
					3,
					self.pg.csize(),
					1e-4 * (0.3f32).powf(m as f32),
					1e-7 * (0.1f32).powf(n as f32),
				);
			}
		}
	}

	#[allow(clippy::all)]
	fn add_test_block(
		&mut self,
		x0: f32,
		y0: f32,
		x: usize,
		y: usize,
		size: f32,
		compl_d: f32,
		compl_v: f32,
	) {
		let mut ps = vec![];
		for idx in 0..x {
			let mut pline = vec![];
			for idy in 0..y {
				let w = if idx == 0 { f32::INFINITY } else { 1.0 };
				let p = self.pg.add_particle(
					w,
					V2::new(x0 + size * idx as f32, y0 + size * idy as f32),
					V2::new(0., -9.8),
				);
				pline.push(p);
			}
			ps.push(pline);
		}
		for idx in 1..x {
			for idy in 0..y {
				let dc = DistanceConstraint::new(
					ps[idx][idy].clone(),
					ps[idx - 1][idy].clone(),
				)
				.attractive_only()
				.with_compliance(compl_d)
				.build();
				self.constraints.push(dc);
			}
		}
		for idx in 0..x {
			for idy in 1..y {
				let dc = DistanceConstraint::new(
					ps[idx][idy].clone(),
					ps[idx][idy - 1].clone(),
				)
				.attractive_only()
				.with_compliance(compl_d)
				.build();
				self.constraints.push(dc);
			}
		}
		for idx in 1..x {
			for idy in 1..y {
				let dc = DistanceConstraint::new(
					ps[idx - 1][idy].clone(),
					ps[idx][idy - 1].clone(),
				)
				.attractive_only()
				.with_compliance(compl_d)
				.build();
				self.constraints.push(dc);
				let dc = DistanceConstraint::new(
					ps[idx - 1][idy - 1].clone(),
					ps[idx][idy].clone(),
				)
				.attractive_only()
				.with_compliance(compl_d)
				.build();
				self.constraints.push(dc);
				let vc = VolumeConstraint::new(vec![
					ps[idx][idy].clone(),
					ps[idx][idy - 1].clone(),
					ps[idx - 1][idy - 1].clone(),
				])
				.with_compliance(compl_v)
				.build();
				self.constraints.push(vc);
				let vc = VolumeConstraint::new(vec![
					ps[idx][idy].clone(),
					ps[idx - 1][idy].clone(),
					ps[idx - 1][idy - 1].clone(),
				])
				.with_compliance(compl_v)
				.build();
				self.constraints.push(vc);
			}
		}
	}

	pub fn pr_model(&self) -> PrModel {
		let ps = self.pg.pr_particles();
		let cs: Vec<PrConstraint> = self.constraints
			.iter()
			.map(|x| x.render())
			.collect();
		PrModel {
			particles: ps,
			constraints: cs,
		}
	}

	#[cfg(not(debug_assertions))]
	fn solve_constraints(&mut self, dt: f32) {
		use rayon::prelude::*;
		self.constraints
			.par_iter_mut()
			.chain(self.tmp_constraints.par_iter_mut())
			.for_each(|constraint| constraint.step(dt));
	} 

	#[cfg(debug_assertions)]
	fn solve_constraints(&mut self, dt: f32) {
		self.constraints
			.iter_mut()
			.chain(self.tmp_constraints.iter_mut())
			.for_each(|constraint| constraint.step(dt));
	} 

	fn update_frame(&mut self, dt: f32, iteration: usize) {
		if dt == 0f32 {
			return;
		}
		self.pg.update(dt);
		self.tmp_constraints = self.pg.collision_constraints();
		for constraint in self.constraints.iter_mut() {
			constraint.pre_iteration();
		}
		for _ in 0..iteration {
			self.solve_constraints(dt);
		}
	}

	pub fn run(&mut self, dt: f32, frame: usize, iteration: usize) {
		for _ in 0..frame {
			self.update_frame(dt, iteration);
		}
	}
}
