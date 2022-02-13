use std::os::unix::net::UnixListener;
use std::time::SystemTime;
use std::io::Write;

// use crate::V2;

#[derive(Default)]
pub struct World {
//	std::collections::HashMap<CellPos>
}

impl World {
	fn update_msg(&self) -> protocol::Message {
		protocol::Message::WorldUpdate(vec![[10f32, 10f32]])
	}

	fn update_frame(&mut self, dt: f32) {
		if dt == 0f32 { return }
	}

	pub fn run(&mut self) {
		let _ = std::fs::remove_file("psva2d.socket");
		let listener = UnixListener::bind("psva2d.socket").unwrap();
		let mut stream = listener.incoming().next().unwrap().unwrap();
		let mut dt = 0f32;
		loop {
			let start_time = SystemTime::now();
			self.update_frame(dt);
			let msg = self.update_msg().to_bytes();
			let duration = SystemTime::now()
				.duration_since(start_time)
				.unwrap()
				.as_micros();
			if duration < 20000 {
				std::thread::sleep(std::time::Duration::from_micros(
					20000 - duration as u64
				));
			}
			dt = duration as f32 / 1e6f32;
			stream.write_all(&msg).unwrap();
		}
	}
}
