use std::sync::mpsc::channel;
use vkrender::renderer::Renderer;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

use protocol::pr_model::PrModel;

fn main() {
	let event_loop = EventLoop::with_user_event();
	let mut renderer = Renderer::new(&event_loop);
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
	let mut last_model = PrModel::default();
	event_loop.run(move |event, _, control_flow| match event {
		Event::WindowEvent {
			event: WindowEvent::CloseRequested,
			..
		} => {
			*control_flow = ControlFlow::Exit;
		}
		Event::WindowEvent {
			event: WindowEvent::Resized(_),
			..
		} => {
			renderer.recreate_swapchain = true;
		}
		Event::RedrawEventsCleared => {
			renderer.render(std::mem::take(&mut last_model));
		}
		Event::UserEvent(pr_model) => {
			last_model = pr_model;
		}
		_ => {}
	});
}
