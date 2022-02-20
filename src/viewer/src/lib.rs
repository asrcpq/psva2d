use std::sync::mpsc::channel;
use winit::event::{Event, KeyboardInput, VirtualKeyCode as Vkc, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

use material::face::TextureData;
use material::texture_indexer::TextureIndexer;
use protocol::pr_model::PrModel;
use protocol::view::View;
use vkrender::camera::Camera;
use vkrender::vkrender::VkRender;
use xpbd::controller_message::ControllerMessage;
use xpbd::pworld::PWorld;

pub fn run(
	mut pworld: PWorld,
	indexer: TextureIndexer,
	textures: Vec<TextureData>,
) {
	let window_size = [1600u32, 1000];
	let event_loop = EventLoop::with_user_event();
	let mut vkr = VkRender::new(&event_loop, textures, window_size);
	let mut view = View::default();
	let elp: EventLoopProxy<PrModel> = event_loop.create_proxy();
	let mut update_flag = true;
	let (tx2, rx2) = channel();
	let _ = std::thread::spawn(move || {
		let (tx, rx) = channel();
		let _ = std::thread::spawn(move || {
			pworld.run_thread(tx, rx2);
		});
		while let Ok(pr_model) = rx.recv() {
			elp.send_event(pr_model).unwrap();
		}
	});
	let mut last_model: Option<PrModel> = None;
	event_loop.run(move |event, _, control_flow| match event {
		Event::WindowEvent { event: e, .. } => match e {
			WindowEvent::CloseRequested => {
				*control_flow = ControlFlow::Exit;
			}
			WindowEvent::Resized(_) => {
				vkr.recreate_swapchain = true;
			}
			WindowEvent::KeyboardInput {
				input:
					KeyboardInput {
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
					Vkc::R => vkr.toggle_render_mode(),
					Vkc::Space => {
						tx2.send(ControllerMessage::TogglePause).unwrap()
					}
					Vkc::S => {
						tx2.send(ControllerMessage::FrameForward).unwrap()
					}
					_ => {}
				}
				update_flag = true;
			}
			_ => {}
		},
		Event::RedrawEventsCleared => {
			if update_flag {
				if let Some(pr_model) = &last_model {
					update_flag = false;
					let render_model = indexer.compile_model(pr_model);
					vkr.render(render_model, Camera::from_view(&view));
				}
			}
		}
		Event::UserEvent(pr_model) => {
			last_model = Some(pr_model);
			update_flag = true;
		}
		Event::MainEventsCleared => {
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
		_ => {}
	});
}
