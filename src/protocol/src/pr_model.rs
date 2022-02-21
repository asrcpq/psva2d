// pr_model: Physical model for rendering

use std::collections::HashMap;

pub struct PrParticle {
	pub pos: [f32; 2],
}

pub struct PrConstraint {
	pub id: isize,
	pub particles: Vec<usize>,
}

#[derive(Default)]
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
