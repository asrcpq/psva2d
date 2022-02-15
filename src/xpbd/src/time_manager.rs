use std::time::SystemTime;

pub enum TimeModel {
	VideoRender,
	RtFrameLock,
	// RtFrameUnlock,
}

pub struct TimeManager {
	pft: f32,
	model: TimeModel,
	pause_start: Option<SystemTime>,
	start_time: SystemTime,
	total_pause: u128,
}

impl Default for TimeManager {
	fn default() -> Self {
		let now = SystemTime::now();
		Self {
			pft: 0.005,
			model: TimeModel::RtFrameLock,
			pause_start: Some(now),
			start_time: now,
			total_pause: 0,
		}
	}
}

impl TimeManager {
	pub fn set(&mut self, on: bool) {
		if on != self.pause_start.is_none() {
			return;
		}
		if on {
			let pause_time = SystemTime::now()
				.duration_since(self.pause_start.take().unwrap())
				.unwrap()
				.as_micros();
			self.total_pause += pause_time;
		} else {
			self.pause_start = Some(SystemTime::now());
		}
	}

	pub fn take_time(&mut self) -> f32 {
		let now = SystemTime::now();
		let passed = now.duration_since(self.start_time).unwrap().as_micros();
		self.start_time = now;
		let total_pause = self.total_pause;
		self.total_pause = 0;
		let dt = passed - total_pause;
		match self.model {
			TimeModel::VideoRender => self.pft,
			TimeModel::RtFrameLock => {
				if dt < (self.pft * 1e6) as u128 {
					std::thread::sleep(std::time::Duration::from_micros(
						(self.pft * 1e6) as u64 - dt as u64,
					));
				}
				self.pft
			}
		}
	}
}
