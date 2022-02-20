use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use protocol::pr_model::PrModel;
use protocol::view::View;

pub struct Renderer {
	canvas: Canvas<Window>,
	vp: View,
}

impl Renderer {
	pub fn new(mut canvas: Canvas<Window>) -> Self {
		canvas.set_draw_color(Color::RGB(0, 0, 0));
		canvas.clear();
		canvas.present();
		Self {
			canvas,
			vp: View::default()
				.with_screen_center([800., 500.])
				.with_scaler0([100., 100.]),
		}
	}
}

impl Renderer {
	fn map_pos(&self, pos: [f32; 2]) -> [i32; 2] {
		let cast = self.vp.w2s(pos);
		[cast[0] as i32, cast[1] as i32]
	}

	pub fn draw_points(&mut self, pr_model: PrModel) {
		self.canvas.set_draw_color(Color::RGB(0, 0, 0));
		self.canvas.clear();
		for pr_constraint in pr_model.constraints.into_iter() {
			if pr_constraint.particles.len() == 2 {
				let prp1 = pr_model
					.particles
					.get(&pr_constraint.particles[0])
					.unwrap();
				let prp2 = pr_model
					.particles
					.get(&pr_constraint.particles[1])
					.unwrap();
				let [x1, y1] = self.map_pos(prp1.pos);
				let [x2, y2] = self.map_pos(prp2.pos);
				self.canvas
					.aa_line(
						x1 as i16,
						y1 as i16,
						x2 as i16,
						y2 as i16,
						Color::RGB(0, 255, 255),
					)
					.unwrap();
			}
		}
		self.canvas.set_draw_color(Color::RGB(255, 0, 255));
		for pr_particle in pr_model.particles.values() {
			let pos = pr_particle.pos;
			let [x, y] = self.map_pos(pos);
			// overflow is okay
			self.canvas
				.draw_points(&*vec![
					(x, y).into(),
					(x - 1, y).into(),
					(x + 1, y).into(),
					(x, y - 1).into(),
					(x, y + 1).into(),
				])
				.unwrap();
		}
		self.canvas.present();
	}
}
