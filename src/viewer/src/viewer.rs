use protocol::V2;
use protocol::user_event::UserEvent;

use std::sync::mpsc::channel;
use winit::event::{
	ElementState,
	Event,
	KeyboardInput,
	MouseButton,
	VirtualKeyCode as Vkc,
	WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

use material::face::TextureData;
use material::texture_indexer::TextureIndexerRef;
use protocol::pr_model::PrModel;
use protocol::view::View;
use vkrender::camera::Camera;
use vkrender::vertex::VertexWf;
use vkrender::vk::vkrender::VkRender;
use xpbd::controller_message::ControllerMessage;
use xpbd::posbox::Posbox;
use xpbd::pworld::PWorld;

pub struct Viewer {
	view: View,
	pworld: Option<PWorld>,
	event_loop: Option<EventLoop<UserEvent>>,
	vkr: VkRender,
	particle_id: Option<usize>,
	last_model: Option<PrModel>,
}

impl Viewer {
	pub fn new(
		mut pworld: PWorld,
		indexer: TextureIndexerRef,
		textures: Vec<TextureData>,
	) -> Self {
		let window_size = [800u32, 600];
		let xmin = -5.0;
		let xmax = 5.0;
		let ymin = -10.0;
		let ymax = 0.;
		let posbox = Posbox {
			xmin,
			xmax,
			ymin,
			ymax,
		};
		pworld = pworld.with_posbox(posbox);
		let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event();
		let mut vkr = VkRender::new(&event_loop, window_size, textures, indexer);
		vkr.set_primitives(
			vec![
				[xmin, ymin],
				[xmin, ymax],
				[xmin, ymax],
				[xmax, ymax],
				[xmax, ymax],
				[xmax, ymin],
				[xmax, ymin],
				[xmin, ymin],
			]
			.into_iter()
			.map(|pos| VertexWf {
				color: [1.0, 1.0, 0.0, 1.0],
				pos,
			})
			.collect(),
		);
	
		Self {
			view: View::default(),
			pworld: Some(pworld),
			event_loop: Some(event_loop),
			vkr,
			particle_id: None,
			last_model: None,
		}

	}

	fn select_particle(&mut self, c: V2) {
		let mut min_dist = f32::INFINITY;
		let mut min_id = 0;
		let c = self.view.s2w(c);
		let pr_model = match self.last_model.as_ref() {
			None => {
				self.particle_id = None;
				return
			},
			Some(m) => m,
		};
		for (id, particle) in &pr_model.particles {
			let pos: V2 = particle.pos.into();
			let dist = (c - pos).magnitude();
			if dist < min_dist {
				min_id = *id;
				min_dist = dist;
			}
		}
		self.particle_id = if min_dist < 0.005 * self.view.get_zoom() {
			Some(min_id)
		} else {
			None
		};
	}

	pub fn run(mut self) {
		let (tx2, rx2) = channel();
		let mut pworld = self.pworld.take().unwrap();
		let event_loop = self.event_loop.take().unwrap();
		let elp: EventLoopProxy<UserEvent> = event_loop.create_proxy();
		let _ = std::thread::spawn(move || {
			let (tx, rx) = channel();
			let _ = std::thread::spawn(move || {
				pworld.run_thread(tx, rx2);
			});
			while let Ok(user_event) = rx.recv() {
				elp.send_event(user_event).unwrap();
			}
		});
		let mut update_flag = true;
		let mut load_smoother = 0.0;
		let mut button_state = [false; 4];
		let mut last_cursor = V2::new(0.0f32, 0.0f32);
		event_loop.run(move |event, _, control_flow| match event {
			Event::WindowEvent { event: e, .. } => match e {
				WindowEvent::CloseRequested => {
					*control_flow = ControlFlow::Exit;
				}
				WindowEvent::Resized(new_size) => {
					self.view.resize([new_size.width, new_size.height]);
					self.vkr.flush_swapchain();
					update_flag = true;
				}
				WindowEvent::CursorMoved {
					position: p,
					..
				} => {
					let c = V2::new(p.x as f32, p.y as f32);
					if button_state[1] {
						self.view.move_view(c - last_cursor);
						update_flag = true;
					} else if button_state[0] {
						if let Some(id) = self.particle_id {
							let c = self.view.s2w(c);
							tx2.send(ControllerMessage::ControlParticle(id, c.into())).unwrap();
						}
					}
					last_cursor = c;
				} 
				WindowEvent::MouseInput {
					button: b,
					state: s,
					..
				} => {
					let pressed = s == ElementState::Pressed;
					let idx = match b {
						MouseButton::Left => 0,
						MouseButton::Middle => 1,
						MouseButton::Right => 2,
						_ => 3,
					};
					button_state[idx] = pressed;
					if idx == 0 {
						if pressed {
							self.select_particle(last_cursor);
						} else {
							self.particle_id = None;
						}
					}
				}
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state: ElementState::Pressed,
							virtual_keycode: Some(keycode),
							..
						},
					..
				} => {
					match keycode {
						Vkc::H => self.view.move_view_key(0),
						Vkc::K => self.view.move_view_key(1),
						Vkc::L => self.view.move_view_key(2),
						Vkc::J => self.view.move_view_key(3),
						Vkc::I => self.view.scale_view(true),
						Vkc::O => self.view.scale_view(false),
						Vkc::R => self.vkr.toggle_render_mode(),
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
					if let Some(pr_model) = &self.last_model {
						update_flag = false;
						self.vkr.render(pr_model, Camera::from_view(&self.view));
					}
				}
			}
			Event::UserEvent(user_event) => match user_event {
				UserEvent::Update(pr_model, load) => {
					load_smoother *= 0.8;
					load_smoother += load * 0.2;
					let load = load_smoother;
					let fps_text =
						format!("Load: {:.2}%", load * 100.).bytes().collect();
					self.vkr.set_text(fps_text, load > 1.0);
					self.last_model = Some(pr_model);
					update_flag = true;
				}
			},
			Event::MainEventsCleared => {
				std::thread::sleep(std::time::Duration::from_millis(10));
			}
			_ => {}
		});
	}
}
