use std::sync::mpsc::{Receiver, Sender};

use crate::constraint::constraint_template::ConstraintTemplate;
use crate::constraint::distance::DistanceConstraint;
use crate::constraint::leash::LeashConstraint;
use crate::constraint::volume::VolumeConstraint;
use crate::constraint_group::ConstraintGroup;
use crate::controller_message::ControllerMessage;
use crate::particle::Particle;
use crate::particle_group::ParticleGroup;
use crate::physical_model::PhysicalModel;
use crate::posbox::Posbox;
use crate::V2;
use protocol::pr_model::PrModel;
use protocol::user_event::UpdateInfo;
use protocol::user_event::UserEvent;
use stpw::Timer;

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
	cg: ConstraintGroup,

	print_perf: bool,
}

impl Default for PWorld {
	fn default() -> Self {
		Self {
			dt: 0.005,
			ppr: 5,
			time_scale: 1.0,
			iteration: 6,
			forward_frames: -1,

			pg: Default::default(),
			cg: Default::default(),

			print_perf: false,
		}
	}
}

impl PWorld {
	pub fn with_posbox(mut self, posbox: Posbox) -> Self {
		self.pg = self.pg.with_posbox(posbox);
		self
	}

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

	pub fn add_model(
		&mut self,
		physical_model: PhysicalModel,
		offset: V2,
	) -> Vec<i32> {
		let mut cids = Vec::new();
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
						.build()
				}
				Volume(ct) => {
					let ps = (0..3).map(|i| id_map[ct.ps[i]].clone()).collect();
					VolumeConstraint::new(ps)
						.with_compliance(ct.compliance)
						.build()
				}
			};
			let cid = self.cg.add_constraint(con);
			cids.push(cid);
		}
		for v in physical_model.dependencies.iter() {
			let key = cids[v[0]];
			let value = v.iter().skip(1).map(|&x| cids[x]).collect();
			self.cg.add_dependency(key, value);
		}
		cids
	}

	pub fn pr_model(&self) -> PrModel {
		let ps = self.pg.pr_particles();
		let cs = self.cg.pr_constraints();
		PrModel {
			particles: ps,
			constraints: cs,
		}
	}

	fn update_frame(&mut self, dt: f32, iteration: usize) {
		let mut timer = Timer::default();
		if dt == 0f32 {
			return;
		}
		self.cg.pre_iteration();
		timer.lap();
		self.pg.update(dt);
		timer.lap();
		self.cg.set_tmp_constraints(self.pg.collision_constraints());
		timer.lap();
		for _ in 0..iteration {
			self.cg.solve_constraints(dt);
		}
		timer.lap();
		if self.print_perf {
			eprintln!("{:?}", timer.get_laps());
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
		let rtime = self.dt * self.ppr as f32 * self.time_scale;
		let mut first_frame = true;
		loop {
			let mut timer = Timer::default();
			if self.forward_frames != 0 {
				self.forward_frames -= 1;
				if !first_frame {
					self.run();
				} else {
					first_frame = false;
				}
				let model = self.pr_model();
				let (dt, _) = timer.lap();
				let event = UserEvent::Update(
					model,
					UpdateInfo {
						load: dt / rtime,
						particle_len: self.pg.len(),
						constraint_len: self.cg.len(),
					},
				);
				tx.send(event).unwrap();
			}

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
					ControllerMessage::ControlParticle(id, pos) => {
						if let Some(pref) = self.pg.get_pref(id) {
							let con =
								LeashConstraint::new_with_pos(pref, pos.into());
							self.cg.control_particle(id, con);
						} else {
							eprintln!(
								"ERROR: control particle id {} is bad",
								id
							);
						}
					}
					ControllerMessage::UncontrolParticle(id) => {
						self.cg.uncontrol_particle(id);
					}
				}
			}
			let (_, dt_a) = timer.lap();
			if dt_a < rtime {
				timer.sleep(rtime - dt_a);
			}
		}
	}
}
