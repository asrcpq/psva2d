use std::time::SystemTime;

use protocol::sock::SockServer;

use crate::constraint::distance::DistanceConstraint;
use crate::constraint::volume::VolumeConstraint;
use crate::constraint::Constraint;
use crate::particle::Particle;
use crate::particle_group::ParticleGroup;
use crate::V2;

pub struct World {
	sock: SockServer,
	pg: ParticleGroup,
	constraints: Vec<Box<dyn Constraint>>,
}

impl Default for World {
	fn default() -> Self {
		let pg = ParticleGroup::default();
		Self {
			sock: SockServer::default(),
			pg,
			constraints: Vec::new(),
		}
	}
}

impl World {
	pub fn init_test(&mut self) {
		let x0 = 0.;
		let y0 = 1.;
		let dx = 0.02;
		let dy = 0.02;
		let p0 =
			Particle::new_ref(f32::INFINITY, V2::new(x0, y0), V2::new(0., 0.));
		let p1 = Particle::new_ref(
			f32::INFINITY,
			V2::new(x0, y0 + dy),
			V2::new(0., 0.),
		);
		self.pg.add_particle(p0.clone());
		self.pg.add_particle(p1.clone());
		let mut last_p0 = p0;
		let mut last_p1 = p1;
		for i in 1..=15 {
			let p0 = Particle::new_ref(
				1.,
				V2::new(x0 + i as f32 * dx, y0),
				V2::new(0., -9.8),
			);
			let p1 = Particle::new_ref(
				1.,
				V2::new(x0 + i as f32 * dx, y0 + dy),
				V2::new(0., -9.8),
			);
			self.pg.add_particle(p0.clone());
			self.pg.add_particle(p1.clone());
			let dc0 =
				DistanceConstraint::new(last_p0.clone(), p0.clone()).build();
			let dc1 =
				DistanceConstraint::new(last_p1.clone(), p1.clone()).build();
			let dc2 = DistanceConstraint::new(p0.clone(), p1.clone()).build();
			self.constraints.push(dc0);
			self.constraints.push(dc1);
			self.constraints.push(dc2);
			let vc0 =
				VolumeConstraint::new([last_p0, last_p1.clone(), p0.clone()])
					.build();
			let vc1 = VolumeConstraint::new([last_p1, p0.clone(), p1.clone()])
				.build();
			self.constraints.push(vc0);
			self.constraints.push(vc1);
			last_p0 = p0;
			last_p1 = p1;
		}
	}

	fn update_msg(&self) -> protocol::Message {
		let mut result = Vec::new();
		for p in self.pg.get_particles().into_iter() {
			let pos = p.borrow().get_pos();
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
			constraint.reset_lambda();
		}
		for _ in 0..iteration {
			for constraint in self.constraints.iter_mut() {
				constraint.step(dt);
			}
		}
	}

	fn send_msg(&mut self) {
		let msg = self.update_msg().to_bytes();
		self.sock.send_msg(&msg);
	}

	pub fn run(&mut self) {
		let mut dt = 0f32;
		self.send_msg();
		loop {
			let start_time = SystemTime::now();
			self.update_frame(dt, 10);
			let duration = SystemTime::now()
				.duration_since(start_time)
				.unwrap()
				.as_micros();
			if duration < 20000 {
				std::thread::sleep(std::time::Duration::from_micros(
					20000 - duration as u64,
				));
			}
			// recompute after sleep
			let duration = SystemTime::now()
				.duration_since(start_time)
				.unwrap()
				.as_micros();
			dt = duration as f32 / 1e6f32;
			self.send_msg();
		}
	}
}
