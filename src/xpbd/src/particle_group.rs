use std::collections::HashMap;

use crate::particle::{Particle, PRef};
use crate::{V2, C2};

pub struct ParticleGroup {
	csize: V2,
	offset: V2,
	data: HashMap<C2, Vec<PRef>>,
}

impl Default for ParticleGroup {
	fn default() -> Self {
		Self {
			csize: V2::new(20., 20.),
			offset: V2::new(0., 0.),
			data: HashMap::new(),
		}
	}
}

impl ParticleGroup {
	pub fn update(&mut self, dt: f32) {
		for p in self.data.values().flatten() {
			p.borrow_mut().update(dt);
		}
	}

	pub fn get_particles(&self) -> Vec<PRef> {
		self.data.values().flatten().cloned().collect()
	}

	fn get_cpos(&self, p: V2) -> C2 {
		let dp = p - self.offset;
		C2::new(
			(dp[0] / self.csize[0]).floor() as isize,
			(dp[1] / self.csize[1]).floor() as isize,
		)
	}

	pub fn add_particle(&mut self, p: PRef) {
		let pos = p.borrow().get_pos();
		let cpos = self.get_cpos(pos);
		let e = self.data.entry(cpos).or_insert_with(Vec::new);
		(*e).push(p);
	}
}
