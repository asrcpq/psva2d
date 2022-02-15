use rayon::prelude::*;

use std::time::SystemTime;

use protocol::sock::SockServer;

use crate::constraint::distance::DistanceConstraint;
use crate::constraint::volume::VolumeConstraint;
use crate::constraint::Constraint;
use crate::particle_group::ParticleGroup;
use crate::V2;

pub struct World {
	sock: SockServer,
	pg: ParticleGroup,
	constraints: Vec<Box<dyn Constraint>>,

	// pframe per rframe
	ppr: usize,

	// physical frametime
	pft: f32,

	// no sleep mode means no frame lock, always use all cpu
	sleep: bool,
}

impl Default for World {
	fn default() -> Self {
		let pg = ParticleGroup::default();
		Self {
			sock: SockServer::default(),
			pg,
			constraints: Vec::new(),
			ppr: 4,
			pft: 0.005,
			sleep: true,
		}
	}
}

impl World {
	pub fn with_pft(mut self, pft: f32) -> Self {
		self.pft = pft;
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
					6,
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
				let vc = VolumeConstraint::new([
					ps[idx][idy].clone(),
					ps[idx][idy - 1].clone(),
					ps[idx - 1][idy - 1].clone(),
				])
				.with_compliance(compl_v)
				.build();
				self.constraints.push(vc);
				let vc = VolumeConstraint::new([
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
			self.constraints.par_iter_mut()
				.for_each(|constraint| constraint.step(dt));
		}
	}

	fn send_msg(&mut self) {
		let msg = self.update_msg().to_bytes();
		self.sock.send_msg(&msg);
	}

	pub fn run(&mut self) {
		self.init_test();
		let mut frame_id = 0;
		let mut dt = 0f32;
		self.send_msg();
		loop {
			let start_time = SystemTime::now();
			if frame_id > 1000 {
				frame_id = 0;
				self.init_test();
			} else {
				frame_id += 1;
			}
			self.update_frame(dt, 20);
			let duration = SystemTime::now()
				.duration_since(start_time)
				.unwrap()
				.as_micros() as f32 / 1e6;
			dt = if duration < self.pft {
				if self.sleep {
					std::thread::sleep(std::time::Duration::from_micros(
						((self.pft - duration) * 1e6) as u64
					));
					self.pft
				} else {
					duration
				}
			} else {
				// laggy
				self.pft
			};
			if frame_id % self.ppr == 0 {
				self.send_msg();
			}
			if frame_id % 100 == 0 {
				eprint!("[2K{:.2}ms\r", (duration - self.pft) * 1e3);
			}
		}
	}
}
