use std::sync::mpsc::channel;
use vkrender::renderer::Renderer;
use winit::event::{Event, KeyboardInput, VirtualKeyCode as Vkc , WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

use protocol::pr_model::PrModel;
use vkrender::view::View;

fn main() {
	let window_size = [1600u32, 1000];
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
				WindowEvent::KeyboardInput {
					input: KeyboardInput {
							state: winit::event::ElementState::Pressed,
							virtual_keycode: Some(keycode),
							..
						},
					..
				} => {
					match keycode {
						Vkc::H => view.move_view(0),
						Vkc::K => view.move_view(1),
						Vkc::L => view.move_view(2),
						Vkc::J => view.move_view(3),
						Vkc::I => view.scale_view(true),
						Vkc::O => view.scale_view(false),
						_ => {}
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
