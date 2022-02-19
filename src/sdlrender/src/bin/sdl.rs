use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::{Duration, SystemTime};

use frontend::renderer::Renderer;
use xpbd::pworld::PWorld;

pub fn main() {
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
	let mut join_handle: Option<std::thread::JoinHandle<PWorld>> = None;
	let mut start_time = SystemTime::now();
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
		let mut world = match join_handle {
			None => {
				let mut world = PWorld::default();
				world.init_test();
				world
			}
			Some(handle) => handle.join().unwrap(),
		};
		let pr_model = world.pr_model();
		let next_time = SystemTime::now();
		let dt = next_time.duration_since(start_time).unwrap().as_micros();
		if dt < 20_000 {
			std::thread::sleep(Duration::from_micros(20_000 - dt as u64));
		}
		start_time = next_time;
		join_handle = Some(std::thread::spawn(move || {
			world.run();
			world
		}));
		renderer.draw_points(pr_model);
		std::thread::sleep(std::time::Duration::from_millis(10));
	}
}
