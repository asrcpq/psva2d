pub struct View {
	world_center: [f32; 2],
	screen_size: [u32; 2],
	zoom: f32,
	move_k: f32,
}

impl Default for View {
	fn default() -> Self {
		Self {
			world_center: [0., 0.],
			screen_size: [640, 480],
			zoom: 200.0,
			move_k: 0.1,
		}
	}
}

impl View {
	pub fn move_view(&mut self, direction: u8) {
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
		self.screen_size = new_size;
	}

	pub fn scale_view(&mut self, zoom_in: bool) {
		if zoom_in {
			self.zoom *= 1.5;
		} else {
			self.zoom /= 1.5;
		}
	}

	pub fn get_c(&self) -> [f32; 2] {
		self.world_center
	}

	pub fn get_r(&self) -> [f32; 2] {
		[
			self.zoom / self.screen_size[0] as f32,
			self.zoom / self.screen_size[1] as f32,
		]
	}
}
