use protocol::pr_model::PrModel;

pub struct View {
	world_center: [f32; 2],
	screen_center: [f32; 2],
	scaler: [f32; 2], // consider flip
	sscaler: f32,
	scaler0: [f32; 2],

	move_distance: f32,
}

impl Default for View {
	fn default() -> Self {
		let scaler0 = [0.1, -0.16];
		Self {
			world_center: [0., 0.],
			screen_center: [0., 0.],
			scaler: scaler0.clone(),
			sscaler: 1.0,
			scaler0,

			move_distance: 0.1,
		}
	}
}

impl View {
	pub fn move_view(&mut self, direction: u8) { // lurd
		match direction {
			0 => self.world_center[0] -= self.move_distance,
			1 => self.world_center[1] += self.move_distance,
			2 => self.world_center[0] += self.move_distance,
			3 => self.world_center[1] -= self.move_distance,
			_ => { eprintln!("ERROR: wrong direction {}", direction) }
		}
	}

	pub fn scale_view(&mut self, zoom_in: bool) {
		if zoom_in {
			self.sscaler *= 1.5;
		} else {
			self.sscaler /= 1.5;
		}
		self.scaler[0] = self.scaler0[0] * self.sscaler;
		self.scaler[1] = self.scaler0[1] * self.sscaler;
	}

	pub fn w2s(&self, p: [f32; 2]) -> [f32; 2] {
		[
			(p[0] - self.world_center[0]) * self.scaler[0] + self.screen_center[0],
			(p[1] - self.world_center[1]) * self.scaler[1] + self.screen_center[1],
		]
	}

	pub fn s2w(&self, p: [f32; 2]) -> [f32; 2] {
		[
			(p[0] - self.screen_center[0]) / self.scaler[0] + self.world_center[0],
			(p[1] - self.screen_center[1]) / self.scaler[1] + self.world_center[1],
		]
	}

	pub fn transform_model(&self, pr_model: &mut PrModel) {
		for particle in pr_model.particles.values_mut() {
			particle.pos = self.w2s(particle.pos);
		}
	}
}
