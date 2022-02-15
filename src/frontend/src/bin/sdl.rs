use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use frontend::V2;
use frontend::viewport::Viewport;
use protocol::sock::SockClient;
use protocol::Message;

pub fn main() {
	let mut sock = SockClient::default();
	eprintln!("Connection ok");
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem
		.window("psva2d", 800, 600)
		.position_centered()
		.build()
		.unwrap();
	let vp = Viewport::default();
	let mut canvas = window.into_canvas().build().unwrap();
	canvas.set_draw_color(Color::RGB(0, 0, 0));
	canvas.clear();
	canvas.present();
	let mut event_pump = sdl_context.event_pump().unwrap();
	'running: loop {
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. }
				| Event::KeyDown {
					keycode: Some(Keycode::Q),
					..
				} => break 'running,
				_ => {}
			}
		}
		canvas.present();
		loop {
			let msg = sock.read_msg();
			match msg {
				// todo: update last only
				Message::WorldUpdate(pvec) => {
					canvas.set_draw_color(Color::RGB(0, 0, 0));
					canvas.clear();
					for p_array in pvec.into_iter() {
						let p: V2 = p_array.try_into().unwrap();
						let [x, y]: [f32; 2] = vp.w2s(p).try_into().unwrap();
						// overflow is okay
						canvas
							.filled_circle(
								x as i16,
								y as i16,
								2,
								Color::RGB(0, 255, 0),
							)
							.unwrap();
					}
				}
				Message::Nop => break,
			}
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
}
