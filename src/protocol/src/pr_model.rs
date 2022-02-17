// pr_model: Physical model for rendering

use std::collections::HashMap;

pub struct PrParticle {
	pub pos: (f32, f32),
}

pub struct PrConstraint {
	pub id: usize,
	pub particles: Vec<usize>,
}

pub struct PrModel {
	pub particles: HashMap<usize, PrParticle>,
	pub constraints: Vec<PrConstraint>,
}

impl std::fmt::Debug for PrModel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(
			f,
			"PrModel with plen: {}, clen: {}",
			self.particles.len(),
			self.constraints.len(),
		)
	}
}
