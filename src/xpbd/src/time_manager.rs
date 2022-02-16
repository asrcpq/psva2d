use std::time::SystemTime;

pub enum TimeModel {
	VideoRender,
	RtFrameLock,
}

pub struct TimeManager {
	pft: f32,
	model: TimeModel,
	start_time: SystemTime,
}

impl Default for TimeManager {
	fn default() -> Self {
		let now = SystemTime::now();
		Self {
			pft: 0.005,
			model: TimeModel::RtFrameLock,
			start_time: now,
		}
	}
}

impl TimeManager {
	pub fn video_render(mut self) -> Self {
		self.model = TimeModel::VideoRender;
		self
	}

	pub fn take_time(&mut self) -> f32 {
		let now = SystemTime::now();
		let dt = now.duration_since(self.start_time).unwrap().as_micros();
		self.start_time = now;
		if matches!(self.model, TimeModel::RtFrameLock)
			&& dt < (self.pft * 1e6) as u128
		{
			std::thread::sleep(std::time::Duration::from_micros(
				(self.pft * 1e6) as u64 - dt as u64,
			));
		}
		self.pft
	}
}
