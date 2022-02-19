use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime};

use crate::constraint::Constraint;
use crate::particle_group::ParticleGroup;
use crate::physical_model::PhysicalModel;
use crate::V2;
use protocol::pr_model::PrConstraint;
use protocol::pr_model::PrModel;

pub struct PWorld {
	pub dt: f32,
	pub ppr: usize,
	iteration: usize,

	pg: ParticleGroup,
	constraints: Vec<Box<dyn Constraint>>,
	tmp_constraints: Vec<Box<dyn Constraint>>,
}

impl Default for PWorld {
	fn default() -> Self {
		let pg = ParticleGroup::default();
		Self {
			dt: 0.005,
			ppr: 4,
			iteration: 20,

			pg,
			constraints: Vec::new(),
			tmp_constraints: Vec::new(),
		}
	}
}

impl PWorld {
	pub fn init_test(&mut self) {
		self.pg = Default::default();
		self.constraints = Default::default();
		for m in 0..3 {
			for n in 0..3 {
				let x = -5.0 + 2.0 * m as f32;
				let y = 0.5 + 1.0 * n as f32 + 0.5 * (m % 2) as f32;
				let pmodel = PhysicalModel::new_block(
					25,
					3,
					self.pg.csize(),
					1e-5 * (0.1f32).powf(m as f32),
					1e-8 * (0.1f32).powf(n as f32),
				);
				self.add_model(pmodel, V2::new(x, y));
			}
		}
	}

	pub fn add_model(&mut self, physical_model: PhysicalModel, offset: V2) {
		for p in physical_model.particles.into_iter() {
			p.try_lock().unwrap().offset_pos(offset);
			self.pg.add_pref(p);
		}
		for c in physical_model.constraints.into_iter() {
			self.constraints.push(c);
		}
	}

	pub fn pr_model(&self) -> PrModel {
		let ps = self.pg.pr_particles();
		let cs: Vec<PrConstraint> =
			self.constraints.iter().map(|x| x.render()).collect();
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

	pub fn run(&mut self) {
		for _ in 0..self.ppr {
			self.update_frame(self.dt, self.iteration);
		}
	}

	pub fn run_thread(&mut self, tx: Sender<PrModel>) {
		let mut start_time = SystemTime::now();
		let rtime: u64 = (self.dt * 1e6 * self.ppr as f32) as u64;
		loop {
			self.run();
			let model = self.pr_model();
			tx.send(model).unwrap();

			let next_time = SystemTime::now();
			let dt = next_time.duration_since(start_time).unwrap().as_micros()
				as u64;
			if dt < rtime {
				std::thread::sleep(Duration::from_micros(rtime - dt));
			}
			start_time = next_time;
		}
	}
}
