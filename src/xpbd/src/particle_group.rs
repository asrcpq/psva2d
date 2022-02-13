use crate::particle::{Particle, PRef};
use crate::{V2, C2};

pub struct ParticleGroup {
	size: C2,
	csize: V2,
	offset: V2,
	data: Vec<Vec<Vec<PRef>>>,
}

impl Default for ParticleGroup {
	fn default() -> Self {
		Self {
			size: C2::new(40, 30),
			csize: V2::new(20., 20.),
			offset: V2::new(0., 0.),
			data: vec![vec![Vec::new(); 40]; 30],
		}
	}
}

impl ParticleGroup {
	pub fn init_test(&mut self) {
		let p = Particle::new_ref(1., V2::new(10., 10.), V2::new(0., 1.));
		self.data[0][0].push(p);
	}

	pub fn update(&mut self, dt: f32) {
		for p in self.data.iter().flatten().flatten() {
			p.borrow_mut().update(dt);
		}
	}

	pub fn get_particles(&self) -> Vec<PRef> {
		self.data.iter().flatten().flatten().cloned().collect()
	}
}
