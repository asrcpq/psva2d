use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, SystemTime};

use crate::constraint::constraint_template::ConstraintTemplate;
use crate::constraint::distance::DistanceConstraint;
use crate::constraint::volume::VolumeConstraint;
use crate::constraint::Constraint;
use crate::controller_message::ControllerMessage;
use crate::particle::Particle;
use crate::particle_group::ParticleGroup;
use crate::physical_model::PhysicalModel;
use crate::V2;
use protocol::pr_model::PrConstraint;
use protocol::pr_model::PrModel;
use protocol::user_event::UserEvent;

pub struct PWorld {
	pub dt: f32,
	pub ppr: usize,
	pub time_scale: f32,
	iteration: usize,

	// -1: always play
	// 0: pause
	// n: play n frames
	forward_frames: i32,

	pg: ParticleGroup,
	constraints: Vec<Box<dyn Constraint>>,
	tmp_constraints: Vec<Box<dyn Constraint>>,
}

impl Default for PWorld {
	fn default() -> Self {
		let pg = ParticleGroup::default();
		Self {
			dt: 0.005,
			ppr: 5,
			time_scale: 1.0,
			iteration: 6,
			forward_frames: -1,

			pg,
			constraints: Vec::new(),
			tmp_constraints: Vec::new(),
		}
	}
}

impl PWorld {
	pub fn with_time_scale(mut self, time_scale: f32) -> Self {
		self.time_scale = time_scale;
		self
	}

	pub fn with_dt(mut self, dt: f32) -> Self {
		self.dt = dt;
		self
	}

	pub fn with_paused(mut self) -> Self {
		self.forward_frames = 1; // provide first frame
		self
	}

	pub fn with_slow_down(mut self, k: f32) -> Self {
		self.dt /= k;
		self.time_scale *= k;
		self
	}

	pub fn with_ppr(mut self, ppr: usize) -> Self {
		self.ppr = ppr;
		self
	}

	pub fn add_model(&mut self, physical_model: PhysicalModel, offset: V2) {
		eprintln!("INFO: add model: {:?}", physical_model);
		let mut id_map = vec![];
		for p in physical_model.particles.into_iter() {
			let p =
				Particle::new_ref(0, p.imass, p.pos + offset, V2::new(0., 9.8));
			self.pg.add_pref(p.clone());
			id_map.push(p);
		}
		for c in physical_model.constraints.into_iter() {
			use ConstraintTemplate::*;
			let con = match c {
				Distance(ct) => {
					let p1 = id_map[ct.ps[0]].clone();
					let p2 = id_map[ct.ps[1]].clone();
					DistanceConstraint::new_with_l0(p1, p2, ct.l0)
						.with_compliance(ct.compliance)
						.with_ty(ct.ty)
						.with_id(ct.id)
						.build()
				}
				Volume(ct) => {
					let ps = (0..3).map(|i| id_map[ct.ps[i]].clone()).collect();
					VolumeConstraint::new(ps)
						.with_compliance(ct.compliance)
						.with_id(ct.id)
						.build()
				}
			};
			self.constraints.push(con);
		}
	}

	pub fn pr_model(&self) -> PrModel {
		let ps = self.pg.pr_particles();
		let cs: Vec<PrConstraint> = self
			.constraints
			.iter()
			.chain(self.tmp_constraints.iter())
			.map(|x| x.render())
			.collect();
		PrModel {
			particles: ps,
			constraints: cs,
		}
	}

	#[cfg(not(debug_assertions))]
	fn solve_constraints(&mut self, dt: f32) {
		use rayon::prelude::*;
		self.constraints
			.par_iter_mut()
			.chain(self.tmp_constraints.par_iter_mut())
			.for_each(|constraint| constraint.step(dt));
	}

	#[cfg(debug_assertions)]
	fn solve_constraints(&mut self, dt: f32) {
		self.constraints
			.iter_mut()
			.chain(self.tmp_constraints.iter_mut())
			.for_each(|constraint| constraint.step(dt));
	}

	fn update_frame(&mut self, dt: f32, iteration: usize) {
		if dt == 0f32 {
			return;
		}
		self.pg.update(dt);
		self.tmp_constraints = self.pg.collision_constraints();
		for constraint in self.constraints.iter_mut() {
			constraint.pre_iteration();
		}
		for _ in 0..iteration {
			self.solve_constraints(dt);
		}
	}

	pub fn run(&mut self) {
		for _ in 0..self.ppr {
			self.update_frame(self.dt, self.iteration);
		}
	}

	pub fn run_thread(
		&mut self,
		tx: Sender<UserEvent>,
		rx: Receiver<ControllerMessage>,
	) {
		let mut start_time = SystemTime::now();
		let rtime: u64 =
			(self.dt * 1e6 * self.ppr as f32 * self.time_scale) as u64;
		let mut first_frame = true;
		loop {
			if self.forward_frames != 0 {
				self.forward_frames -= 1;
				if !first_frame {
					self.run();
				} else {
					first_frame = false;
				}
				let model = self.pr_model();
				let next_time = SystemTime::now();
				let dt =
					next_time.duration_since(start_time).unwrap().as_micros()
						as f32 / rtime as f32;
				let event = UserEvent::Update(model, dt);
				tx.send(event).unwrap();
			}

			let next_time = SystemTime::now();
			let dt = next_time.duration_since(start_time).unwrap().as_micros()
				as u64;
			while let Ok(msg) = rx.try_recv() {
				match msg {
					ControllerMessage::TogglePause => {
						if self.forward_frames == 0 {
							self.forward_frames = -1;
						} else {
							self.forward_frames = 0;
						}
					}
					ControllerMessage::FrameForward => {
						if self.forward_frames == 0 {
							self.forward_frames += 1;
						}
					}
				}
			}
			if dt < rtime {
				std::thread::sleep(Duration::from_micros(rtime - dt));
			}
			start_time = next_time;
		}
	}
}
