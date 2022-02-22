use protocol::user_event::UserEvent;

use std::sync::mpsc::channel;
use winit::event::{Event, KeyboardInput, VirtualKeyCode as Vkc, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

use material::face::TextureData;
use material::texture_indexer::TextureIndexerRef;
use protocol::pr_model::PrModel;
use protocol::view::View;
use vkrender::camera::Camera;
use vkrender::vkrender::VkRender;
use xpbd::controller_message::ControllerMessage;
use xpbd::pworld::PWorld;

pub fn run(
	mut pworld: PWorld,
	indexer: TextureIndexerRef,
	textures: Vec<TextureData>,
) {
	let window_size = [1600u32, 1000];
	let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event();
	let mut vkr = VkRender::new(&event_loop, window_size, textures, indexer);
	let mut view = View::default();
	let elp: EventLoopProxy<UserEvent> = event_loop.create_proxy();
	let mut update_flag = true;
	let (tx2, rx2) = channel();
	let _ = std::thread::spawn(move || {
		let (tx, rx) = channel();
		let _ = std::thread::spawn(move || {
			pworld.run_thread(tx, rx2);
		});
		while let Ok(user_event) = rx.recv() {
			elp.send_event(user_event).unwrap();
		}
	});
	let mut last_model: Option<PrModel> = None;
	event_loop.run(move |event, _, control_flow| match event {
		Event::WindowEvent { event: e, .. } => match e {
			WindowEvent::CloseRequested => {
				*control_flow = ControlFlow::Exit;
			}
			WindowEvent::Resized(new_size) => {
				view.resize([new_size.width, new_size.height]);
				vkr.recreate_swapchain = true;
				update_flag = true;
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
					vkr.render(pr_model, Camera::from_view(&view));
				}
			}
		}
		Event::UserEvent(user_event) => match user_event {
			UserEvent::Update(pr_model, load) => {
				let fps_text =
					format!("Load: {:.2}%", load * 100.).bytes().collect();
				vkr.set_text(fps_text, load > 1.0);
				last_model = Some(pr_model);
				update_flag = true;
			}
		},
		Event::MainEventsCleared => {
			std::thread::sleep(std::time::Duration::from_millis(10));
		}
		_ => {}
	});
}
