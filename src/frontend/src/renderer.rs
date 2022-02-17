use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use protocol::pr_model::PrModel;
use crate::viewport::Viewport;
use crate::V2;

pub struct Renderer {
	canvas: Canvas<Window>,
	vp: Viewport,
}

impl Renderer {
	pub fn new(mut canvas: Canvas<Window>) -> Self {
		canvas.set_draw_color(Color::RGB(0, 0, 0));
		canvas.clear();
		canvas.present();
		Self {
			canvas,
			vp: Viewport::default(),
		}
	}
}

impl Renderer {
	pub fn draw_points(&mut self, pr_model: PrModel) {
		self.canvas.set_draw_color(Color::RGB(0, 0, 0));
		self.canvas.clear();
		self.canvas.set_draw_color(Color::RGB(0, 255, 127));
		for pr_particle in pr_model.particles.into_iter() {
			let pos = pr_particle.pos;
			let p = V2::new(pos.0, pos.1);
			let [x, y]: [f32; 2] = self.vp.w2s(p).try_into().unwrap();
			let x = x as i32;
			let y = y as i32;
			// overflow is okay
			self.canvas.draw_points(&*vec![
				(x, y).into(),
				(x - 1, y).into(),
				(x + 1, y).into(),
				(x, y - 1).into(),
				(x, y + 1).into(),
			]).unwrap();
		}
		self.canvas.present();
	}
}
