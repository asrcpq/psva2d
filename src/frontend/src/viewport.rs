use crate::V2;

pub struct Viewport {
	world_center: V2,
	screen_center: V2,
	scaler: V2, // consider flip
}

impl Default for Viewport {
	fn default() -> Self {
		Self {
			world_center: V2::new(0.0, 2.0),
			screen_center: V2::new(800., 500.),
			scaler: V2::new(200., -200.),
		}
	}
}

impl Viewport {
	pub fn w2s(&self, p: V2) -> V2 {
		(p - self.world_center).component_mul(&self.scaler) + self.screen_center
	}
}
