use crate::constraint::PRef;

#[derive(Clone)]
pub struct ParticleList {
	particles: Vec<PRef>,
}

impl ParticleList {
	pub fn new(particles: Vec<PRef>, sort: bool) -> Self {
		if sort {
			let mut zipped: Vec<_> = particles
				.into_iter()
				.map(|x| {
					(
						{
							let id = x.try_read().unwrap().get_id();
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
		} else {
			Self { particles }
		}
	}

	pub fn ids(&self) -> Vec<usize> {
		self.particles
			.iter()
			.map(|x| x.try_read().unwrap().get_id())
			.collect()
	}
}

impl std::ops::Index<usize> for ParticleList {
	type Output = PRef;
	fn index(&self, idx: usize) -> &Self::Output {
		&self.particles[idx]
	}
}
