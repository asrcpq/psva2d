use protocol::pr_model::PrModel;

pub struct View {
	world_center: [f32; 2],
	screen_center: [f32; 2],
	scaler: [f32; 2], // consider flip
}

impl Default for View {
	fn default() -> Self {
		Self {
			world_center: [0., 0.],
			screen_center: [0., 0.],
			scaler: [0.1, -0.16],
		}
	}
}

impl View {
	pub fn move_view(&mut self, ds: [f32; 2]) {
		self.world_center[0] += ds[0];
		self.world_center[1] += ds[1];
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
