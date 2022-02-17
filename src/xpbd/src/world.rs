use protocol::sock::SockServer;
use protocol::pr_model::PrModel;

use crate::constraint::distance::DistanceConstraint;
use crate::constraint::volume::VolumeConstraint;
use crate::constraint::Constraint;
use crate::particle_group::ParticleGroup;
use crate::time_manager::TimeManager;
use crate::V2;

pub struct World {
	sock: SockServer,
	pg: ParticleGroup,
	constraints: Vec<Box<dyn Constraint>>,
	tmp_constraints: Vec<Box<dyn Constraint>>,
	timeman: TimeManager,
	bench_mode: bool,
	cutoff_frame: usize,

	// pframe per rframe
	ppr: usize,
}

impl Default for World {
	fn default() -> Self {
		let pg = ParticleGroup::default();
		Self {
			sock: SockServer::default(),
			pg,
			constraints: Vec::new(),
			tmp_constraints: Vec::new(),
			timeman: TimeManager::default(),
			bench_mode: false,
			cutoff_frame: 0,
			ppr: 4,
		}
	}
}

impl World {
	pub fn bench_mode(mut self, frame_count: usize) -> Self {
		self.timeman = self.timeman.video_render();
		self.cutoff_frame = frame_count;
		self.bench_mode = true;
		self
	}

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
					ps[idx][idy].clone(),
					ps[idx - 1][idy - 1].clone(),
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

	fn update_msg(&self) -> protocol::Message {
		let ps = self.pg.pr_particles();
		protocol::Message::WorldUpdate(PrModel {
			particles: ps,
			constraints: Vec::new()
		})
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

	fn send_msg(&mut self) {
		if self.bench_mode {
			return;
		}
		let msg = self.update_msg().to_bytes();
		self.sock.send_msg(&msg);
	}

	pub fn run(&mut self) {
		self.init_test();
		let mut frame_id = 0;
		self.send_msg();
		loop {
			if self.cutoff_frame == 0 && frame_id > 1000 {
				frame_id = 0;
				self.init_test();
			} else {
				frame_id += 1;
			}
			let dt = self.timeman.take_time();
			self.update_frame(dt, 20);
			if frame_id % self.ppr == 0 {
				self.send_msg();
				if self.cutoff_frame > 0
					&& frame_id / self.ppr == self.cutoff_frame
				{
					return;
				}
			}
		}
	}
}
