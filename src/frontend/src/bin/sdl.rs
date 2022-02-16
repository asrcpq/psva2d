use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use frontend::renderer::Renderer;
use protocol::sock::SockClient;
use protocol::Message;

pub fn main() {
	let mut sock = SockClient::default();
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem
		.window("psva2d", 1600, 1000)
		.position_centered()
		.build()
		.unwrap();
	let canvas = window.into_canvas().build().unwrap();
	let mut renderer = Renderer::new(canvas);
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
		loop {
			let msg = sock.read_msg();
			match msg {
				// todo: update last only
				Message::WorldUpdate(pvec) => {
					renderer.draw_points(pvec);
				}
				Message::Nop => break,
			}
		}
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
}
