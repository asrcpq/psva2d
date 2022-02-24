use crate::V2;

pub struct View {
	world_center: V2,
	screen_r: V2,
	zoom: f32,
	move_k: f32,
}

impl Default for View {
	fn default() -> Self {
		Self {
			world_center: V2::new(0., -5.),
			screen_r: V2::new(640., 480.), // half size
			zoom: 100.0,
			move_k: 0.2,
		}
	}
}

impl View {
	pub fn move_view(&mut self, ds: V2) {
		self.world_center -= ds / self.zoom;
	}

	pub fn get_zoom(&self) -> f32 {
		self.zoom
	}

	pub fn s2w(&self, pos: V2) -> V2 {
		let result = (pos - self.screen_r) / self.zoom;
		result + self.world_center
	}

	pub fn move_view_key(&mut self, direction: u8) {
		// lurd
		match direction {
			0 => self.world_center[0] -= self.move_k,
			1 => self.world_center[1] -= self.move_k,
			2 => self.world_center[0] += self.move_k,
			3 => self.world_center[1] += self.move_k,
			_ => {
				eprintln!("ERROR: wrong direction {}", direction)
			}
		}
	}

	pub fn resize(&mut self, new_size: [u32; 2]) {
		self.screen_r[0] = new_size[0] as f32 / 2.;
		self.screen_r[1] = new_size[1] as f32 / 2.;
	}

	pub fn zoom(&mut self, k: f32) {
		self.zoom *= k;
	}

	pub fn scale_view(&mut self, zoom_in: bool) {
		if zoom_in {
			self.zoom *= 1.5;
		} else {
			self.zoom /= 1.5;
		}
	}

	pub fn get_c(&self) -> [f32; 2] {
		self.world_center.into()
	}

	pub fn get_r(&self) -> [f32; 2] {
		[
			self.zoom / self.screen_r[0] as f32,
			self.zoom / self.screen_r[1] as f32,
		]
	}
}
