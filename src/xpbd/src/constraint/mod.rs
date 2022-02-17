pub mod distance;
pub mod volume;

use crate::particle::PRef;
use protocol::pr_model::PrConstraint;

pub trait Constraint: Send {
	fn pre_iteration(&mut self);
	fn step(&mut self, dt: f32);
	fn render(&self) -> PrConstraint;
}

pub struct ParticleList {
	particles: Vec<PRef>,
}

impl ParticleList {
	pub fn new(particles: Vec<PRef>) -> Self {
		let mut zipped: Vec<_> = particles
			.into_iter()
			.map(|x| {
				(
					{
						let id = x.try_lock().unwrap().get_id();
						id
					},
					x,
				)
			})
			.collect();
		zipped.sort_by_key(|x| x.0);
		Self {
			particles: zipped.into_iter().map(|(_, p)| p).collect(),
		}
	}

	pub fn ids(&self) -> Vec<usize> {
		self.particles
			.iter()
			.map(|x| x.try_lock().unwrap().get_id())
			.collect()
	}
}

impl std::ops::Index<usize> for ParticleList {
	type Output = PRef;
	fn index(&self, idx: usize) -> &Self::Output {
		&self.particles[idx]
	}
}
