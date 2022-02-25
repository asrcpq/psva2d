use std::sync::{Arc, RwLock};

use crate::V2;
use protocol::pr_model::PrParticle;

pub type PRef = Arc<RwLock<Particle>>;

#[derive(Clone)]
pub struct ParticleTemplate {
	pub imass: f32,
	pub pos: V2,
}

#[derive(Clone)]
pub struct Particle {
	pub id: usize, // prevent dead lock
	pub imass: f32,
	pub pos: V2,
	pub ppos: V2,
	pub accel: V2,
}

impl Particle {
	pub fn new_ref(id: usize, imass: f32, pos: V2, accel: V2) -> PRef {
		let result = Self {
			id,
			imass, // inf is handled
			pos,
			ppos: pos,
			accel,
		};
		Arc::new(RwLock::new(result))
	}

	pub fn get_id(&self) -> usize {
		self.id
	}

	pub fn set_id(&mut self, id: usize) {
		self.id = id;
	}

	pub fn get_pos(&self) -> V2 {
		self.pos
	}

	pub fn add_pos(&mut self, dp: V2) {
		self.pos += dp
	}

	pub fn offset_pos(&mut self, dp: V2) {
		self.pos += dp;
		self.ppos += dp;
	}

	pub fn reset_pos(&mut self, p: V2) {
		self.pos = p;
		self.ppos = p;
	}

	pub fn get_imass(&self) -> f32 {
		self.imass
	}

	pub fn update(&mut self, t: f32, max_dp: f32) {
		if self.imass == 0f32 {
			return;
		}
		let ppos = self.pos;
		let dv = self.accel * t;
		// NOTE: this does not work with dynamic t, should record last t and scale
		let mut dp = self.pos - self.ppos + dv * t;
		if dp.magnitude() > max_dp {
			dp = dp.normalize() * max_dp;
		}
		self.pos += dp;
		self.ppos = ppos;
	}

	pub fn render(&self) -> PrParticle {
		PrParticle {
			pos: [self.pos[0], self.pos[1]],
		}
	}
}
