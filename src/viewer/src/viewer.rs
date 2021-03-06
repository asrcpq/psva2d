use protocol::user_event::UserEvent;
use protocol::V2;

use std::sync::mpsc::{channel, Sender};
use winit::event::{
	ElementState, Event, KeyboardInput, ModifiersState, MouseButton,
	WindowEvent,
};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

use crate::keycode::key2byte;
use material::face::TextureData;
use material::texture_indexer::TextureIndexerRef;
use protocol::pr_model::PrModel;
use protocol::view::View;
use vkrender::camera::Camera;
use vkrender::render_mode::RenderMode;
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
	render_mode: RenderMode,
	update_flag: bool,
	particle_id: Option<usize>,
	last_model: Option<PrModel>,
	input_buffer: Vec<u8>,
	controller: Option<Sender<ControllerMessage>>,
}

impl Viewer {
	pub fn new(
		mut pworld: PWorld,
		indexer: TextureIndexerRef,
		textures: Vec<TextureData>,
	) -> Self {
		let window_size = [800u32, 600];
		let xmin = -15.0;
		let xmax = 15.0;
		let ymin = -30.0;
		let ymax = 0.;
		let posbox = Posbox {
			xmin,
			xmax,
			ymin,
			ymax,
		};
		pworld = pworld.with_posbox(posbox);
		let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event();
		let mut vkr =
			VkRender::new(&event_loop, window_size, textures, indexer);
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
			render_mode: RenderMode::default(),
			update_flag: true,
			particle_id: None,
			last_model: None,
			input_buffer: Vec::new(),
			controller: None,
		}
	}

	fn select_particle(&mut self, c: V2) {
		let mut min_dist = f32::INFINITY;
		let mut min_id = 0;
		let c = self.view.s2w(c);
		let pr_model = match self.last_model.as_ref() {
			None => {
				self.particle_id = None;
				return;
			}
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

	fn send(&mut self, msg: ControllerMessage) {
		self.controller.as_mut().unwrap().send(msg).unwrap()
	}

	pub fn run(mut self) {
		let (tx2, rx2) = channel();
		self.controller = Some(tx2);
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
		let mut load_smoother = 0.0;
		let mut button_state = [false; 4];
		let mut last_cursor = V2::new(0.0f32, 0.0f32);
		let mut modstate = ModifiersState::default();
		event_loop.run(move |event, _, control_flow| match event {
			Event::WindowEvent { event: e, .. } => match e {
				WindowEvent::CloseRequested => {
					*control_flow = ControlFlow::Exit;
				}
				WindowEvent::Resized(new_size) => {
					self.view.resize([new_size.width, new_size.height]);
					self.vkr.flush_swapchain();
					self.update_flag = true;
				}
				WindowEvent::ModifiersChanged(modstate2) => {
					modstate = modstate2;
				}
				WindowEvent::CursorMoved { position: p, .. } => {
					let c = V2::new(p.x as f32, p.y as f32);
					if button_state[1] {
						if modstate.ctrl() {
							let mut k = (c - last_cursor).y;
							k = (k / -100.).exp();
							self.view.zoom(k);
						} else {
							self.view.move_view(c - last_cursor);
						}
						self.update_flag = true;
					} else if button_state[0] {
						if let Some(id) = self.particle_id {
							let c = self.view.s2w(c);
							self.send(ControllerMessage::ControlParticle(
								id,
								c.into(),
							));
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
						} else if let Some(id) = self.particle_id.take() {
							self.send(ControllerMessage::UncontrolParticle(id));
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
					if let Some(byte) = key2byte(keycode) {
						self.input_buffer.push(byte);
						self.parse_input_buffer();
						let input_text = if self.input_buffer.is_empty() {
							Vec::new()
						} else {
							let mut input_text =
								"key: ".bytes().collect::<Vec<u8>>();
							input_text.extend(self.input_buffer.clone());
							input_text
						};
						self.vkr.set_text("input", input_text, false);
						self.update_flag = true;
					}
				}
				_ => {}
			},
			Event::RedrawEventsCleared => {
				if self.update_flag {
					if let Some(pr_model) = &self.last_model {
						self.update_flag = false;
						self.vkr
							.render(pr_model, Camera::from_view(&self.view));
					}
				}
			}
			Event::UserEvent(user_event) => match user_event {
				UserEvent::Update(pr_model, info) => {
					load_smoother *= 0.8;
					load_smoother += info.load * 0.2;
					let load = load_smoother;
					let fps_text =
						format!("Load: {:.2}%", load * 100.).bytes().collect();
					self.vkr.set_text("load", fps_text, load > 1.0);
					let p = info.particle_len;
					let par_text = format!("psize: {}", p).bytes().collect();
					self.vkr.set_text("par", par_text, false);
					let c0 = info.constraint_len[0];
					let c1 = info.constraint_len[1];
					let c2 = info.constraint_len[2];
					let con_text = format!("csize: {} {} {}", c0, c1, c2)
						.bytes()
						.collect();
					self.vkr.set_text("con", con_text, false);
					self.last_model = Some(pr_model);
					self.update_flag = true;
				}
			},
			Event::MainEventsCleared => {
				std::thread::sleep(std::time::Duration::from_millis(10));
			}
			_ => {}
		});
	}

	fn parse_input_buffer(&mut self) {
		if self.input_buffer.is_empty() {
			return;
		}
		match self.input_buffer[0] {
			b'h' => self.view.move_view_key(0),
			b'k' => self.view.move_view_key(1),
			b'l' => self.view.move_view_key(2),
			b'j' => self.view.move_view_key(3),
			b'i' => self.view.scale_view(true),
			b'o' => self.view.scale_view(false),
			b'r' => {
				let flag = match self.input_buffer.get(1) {
					Some(b'c') => {
						self.render_mode.constraint =
							!self.render_mode.constraint;
						true
					}
					Some(b'b') => {
						self.render_mode.world_box =
							!self.render_mode.world_box;
						true
					}
					Some(_) => false,
					None => return,
				};
				if flag {
					self.vkr.set_render_mode(self.render_mode);
				}
			}
			b' ' => self.send(ControllerMessage::TogglePause),
			b's' => self.send(ControllerMessage::FrameForward),
			_ => {}
		}
		self.input_buffer = Vec::new();
	}
}
