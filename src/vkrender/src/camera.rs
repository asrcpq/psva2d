use protocol::view::View;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Camera {
	pub c: [f32; 2],
	pub r: [f32; 2],
}

impl Default for Camera {
	fn default() -> Self {
		Camera {
			c: [0.0, 0.0],
			r: [0.1, -0.16],
		}
	}
}

impl Camera {
	pub fn new_view() -> View {
		View::default()
	}

	pub fn from_view(view: &View) -> Self {
		Self {
			c: view.get_c(),
			r: view.get_r(),
		}
	}
}
