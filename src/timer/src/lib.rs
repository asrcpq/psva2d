use std::time::SystemTime;

pub struct Timer {
	prev_time: SystemTime,
	pub laps: Vec<f32>,
	pub laps_accumulate: Vec<f32>,
}

impl Default for Timer {
	fn default() -> Self {
		Self {
			prev_time: SystemTime::now(),
			laps: Vec::new(),
			laps_accumulate: Vec::new(),
		}
	}
}

impl Timer {
	pub fn lap(&mut self) -> (f32, f32) {
		let dt = SystemTime::now()
			.duration_since(self.prev_time)
			.unwrap()
			.as_micros();
		let dt = dt as f32 / 1e6;
		self.laps.push(dt);
		let dt_accumulate = dt + self.laps_accumulate.last().unwrap_or(&0.);
		self.laps_accumulate.push(dt_accumulate);
		self.prev_time = SystemTime::now();
		(dt, dt_accumulate)
	}

	pub fn sleep(&self, t: f32) {
		std::thread::sleep(std::time::Duration::from_micros((t * 1e6) as u64));
	}
}
