use crate::constraint::constraint_template::ConstraintTemplate;
use crate::particle::ParticleTemplate;

#[derive(Clone, Default)]
pub struct PhysicalModel {
	pub particles: Vec<ParticleTemplate>,
	pub constraints: Vec<ConstraintTemplate>,
	// for each vec: first object depends on existence of all others
	// usize for constraints idx
	pub dependencies: Vec<[usize; 2]>,
}

impl std::fmt::Debug for PhysicalModel {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(
			f,
			"PhysicalModel with plen: {}, clen: {}",
			self.particles.len(),
			self.constraints.len(),
		)
	}
}
