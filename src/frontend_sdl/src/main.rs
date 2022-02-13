use sdl2::gfx::primitives::DrawRenderer;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::time::Duration;
use std::thread::sleep;
use std::os::unix::net::UnixStream;
use std::io::Read;

use protocol::Message::{self, *};

pub fn main() {
	let mut stream = loop {
		match UnixStream::connect("psva2d.socket") {
			Ok(s) => break s,
			Err(_) => {
				eprintln!("Waiting connection");
				sleep(Duration::from_secs(1));
			},
		}
	};
	eprintln!("Connection ok");
	let mut buf = vec![0u8; 10_000_000];
	stream.set_nonblocking(true).unwrap();
	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem.window("psva2d", 800, 600)
		.position_centered()
		.build()
		.unwrap();
	let mut canvas = window.into_canvas().build().unwrap();
	canvas.set_draw_color(Color::RGB(0, 0, 0));
	canvas.clear();
	canvas.present();
	let mut event_pump = sdl_context.event_pump().unwrap();
	let mut i = 0;
	'running: loop {
		i = (i + 1) % 255;
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
					break 'running
				},
				_ => {}
			}
		}
		canvas.present();
		let buflen = loop {
			if let Ok(buflen) = stream.read(&mut buf) {
				break buflen
			}
			sleep(Duration::from_millis(10));
		};
		let msg = Message::from_bytes(&buf[..buflen]);
		match msg {
			WorldUpdate(pvec) => {
				eprintln!("Update");
				canvas.set_draw_color(Color::RGB(0, 0, 0));
				canvas.clear();
				for [x, y] in pvec.into_iter() {
					// overflow is okay
					canvas.filled_circle(
						x as i16,
						y as i16,
						5,
						Color::RGB(0, 255, 0),
					).unwrap();
				}
			},
			_ => {},
		}
	}
}
