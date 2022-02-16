use std::collections::HashMap;

use crate::particle::{PRef, Particle};
use crate::{C2, V2};

pub struct ParticleGroup {
	id_alloc: usize,
	csize: V2,
	offset: V2,
	data: HashMap<C2, Vec<PRef>>,
}

impl Default for ParticleGroup {
	fn default() -> Self {
		Self {
			id_alloc: 0,
			csize: V2::new(20., 20.),
			offset: V2::new(0., 0.),
			data: HashMap::new(),
		}
	}
}

impl ParticleGroup {
	pub fn update(&mut self, dt: f32) {
		let old_data = std::mem::take(&mut self.data);
		for p in old_data.into_iter().map(|(_, p)| p).flatten() {
			p.lock().unwrap().update(dt);
			self.add_pref(p);
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

	pub fn add_particle(&mut self, mass: f32, pos: V2, accel: V2) -> PRef {
		let p = Particle::new_ref(self.id_alloc, mass, pos, accel);
		let cpos = self.get_cpos(pos);
		let e = self.data.entry(cpos).or_insert_with(Vec::new);
		(*e).push(p.clone());
		self.id_alloc += 1;
		p
	}

	fn add_pref(&mut self, pref: PRef) {
		let pos = pref.lock().unwrap().get_pos();
		let cpos = self.get_cpos(pos);
		let e = self.data.entry(cpos).or_insert_with(Vec::new);
		(*e).push(pref);
	}
}
