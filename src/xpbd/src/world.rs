use rayon::prelude::*;

use protocol::sock::SockServer;

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
		for m in 0..10 {
			for n in 0..6 {
				let x = -2.5 + 0.5 * m as f32;
				let y = 0.5 + 0.5 * n as f32;
				self.add_test_block(
					x,
					y,
					15,
					3,
					0.02,
					1e-4 * (0.5f32).powf(m as f32),
					1e-6 * (0.1f32).powf(n as f32),
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
		let mut result = Vec::new();
		for p in self.pg.get_particles().into_iter() {
			let pos = p.lock().unwrap().get_pos();
			result.push(pos.try_into().unwrap())
		}
		protocol::Message::WorldUpdate(result)
	}

	fn update_frame(&mut self, dt: f32, iteration: usize) {
		if dt == 0f32 {
			return;
		}
		self.pg.update(dt);
		for constraint in self.constraints.iter_mut() {
			constraint.pre_iteration();
		}
		for _ in 0..iteration {
			self.constraints
				.par_iter_mut()
				.for_each(|constraint| constraint.step(dt));
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
