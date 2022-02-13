use std::os::unix::net::{UnixListener, UnixStream};
use std::time::SystemTime;
use std::io::Write;

use crate::particle_group::ParticleGroup;

pub struct World {
	listener: UnixListener,
	stream: Option<UnixStream>,
	pg: ParticleGroup,
}

impl Default for World {
	fn default() -> Self {
		let _ = std::fs::remove_file("psva2d.socket");
		let listener = UnixListener::bind("psva2d.socket").unwrap();
		let mut pg = ParticleGroup::default();
		pg.init_test();
		Self {
			listener,
			stream: None,
			pg,
		}
	}
}

impl World {
	fn listen(&mut self) {
		let stream = self.listener.incoming().next().unwrap().unwrap();
		self.stream = Some(stream);
	}

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

	fn send_msg(&mut self) {
		let msg = self.update_msg().to_bytes();
		loop {
			if let Some(stream) = self.stream.as_mut() {
				if let Ok(_) = stream.write_all(&msg) {
					return
				}
			}
			eprintln!("Wait for connection");
			self.listen();
			eprintln!("Connected");
		}
	}

	pub fn run(&mut self) {
		let mut dt = 0f32;
		self.send_msg();
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
			self.send_msg();
			// recompute after sleep
			let duration = SystemTime::now()
				.duration_since(start_time)
				.unwrap()
				.as_micros();
			dt = duration as f32 / 1e6f32;
		}
	}
}
