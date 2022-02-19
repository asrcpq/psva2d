use std::sync::mpsc::channel;
use vkrender::renderer::Renderer;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

use protocol::pr_model::PrModel;
use vkrender::view::View;

fn main() {
	let window_size = [1600u32, 1000];
	let mouse_scale = window_size[1] as f32;
	let event_loop = EventLoop::with_user_event();
	let mut renderer = Renderer::new(&event_loop, window_size.clone());
	let mut view = View::default();
	let elp: EventLoopProxy<PrModel> = event_loop.create_proxy();
	let _ = std::thread::spawn(move || {
		let (tx, rx) = channel();
		let _ = std::thread::spawn(move || {
			let mut world = xpbd::world::World::default();
			world.init_test();
			world.run_thread(tx);
		});
		while let Ok(pr_model) = rx.recv() {
			elp.send_event(pr_model).unwrap();
		}
	});
	let mut last_model: Option<PrModel> = None;
	let mut last_cursor: [f32; 2] = [0.; 2];
	let mut middle_button = false;
	event_loop.run(move |event, _, control_flow| match event {
		Event::WindowEvent {
			event: e,
			..
		} => {
			match e {
				WindowEvent::CloseRequested => {
					*control_flow = ControlFlow::Exit;
				},
				WindowEvent::Resized(_) => {
					renderer.recreate_swapchain = true;
				},
				WindowEvent::CursorMoved {
					position: p,
					..
				} => {
					let px = p.x as f32;
					let py = p.y as f32;
					if middle_button {
						view.move_view([
							(px - last_cursor[0]) / mouse_scale * -5.0,
							(py - last_cursor[1]) / mouse_scale * -5.0,
						]);
					}
					last_cursor = view.s2w([px, py]);
				},
				WindowEvent::MouseInput {
					state: s,
					button: winit::event::MouseButton::Middle,
					..
				} => {
					if s == winit::event::ElementState::Pressed {
						middle_button = true;
					} else {
						middle_button = false;
					}
				},
				_ => {},
			}
		}
		Event::RedrawEventsCleared => {
			if let Some(mut pr_model) = last_model.take() {
				view.transform_model(&mut pr_model);
				renderer.render(pr_model);
			}
		}
		Event::UserEvent(pr_model) => {
			last_model = Some(pr_model);
		}
		_ => {}
	});
}
