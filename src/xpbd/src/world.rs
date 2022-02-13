use std::time::SystemTime;
use std::io::Write;

use crate::particle_group::ParticleGroup;
use protocol::sock::SockServer;

pub struct World {
	sock: SockServer,
	pg: ParticleGroup,
}

impl Default for World {
	fn default() -> Self {
		let mut pg = ParticleGroup::default();
		pg.init_test();
		Self {
			sock: SockServer::default(),
			pg,
		}
	}
}

impl World {
	fn update_msg(&self) -> protocol::Message {
		let mut result = Vec::new();
		for p in self.pg.get_particles().into_iter() {
			let pos = p.borrow().get_pos();
			result.push(pos.try_into().unwrap())
		}
		protocol::Message::WorldUpdate(result)
	}

	fn update_frame(&mut self, dt: f32) {
		if dt == 0f32 { return }
		self.pg.update(dt);
	}

	pub fn run(&mut self) {
		let mut dt = 0f32;
		loop {
			let start_time = SystemTime::now();
			self.update_frame(dt);
			let duration = SystemTime::now()
				.duration_since(start_time)
				.unwrap()
				.as_micros();
			if duration < 20000 {
				std::thread::sleep(std::time::Duration::from_micros(
					20000 - duration as u64
				));
			}
			let msg = self.update_msg().to_bytes();
			self.sock.send_msg(&msg);
			// recompute after sleep
			let duration = SystemTime::now()
				.duration_since(start_time)
				.unwrap()
				.as_micros();
			dt = duration as f32 / 1e6f32;
		}
	}
}
