use crate::constraint::constraint_template::ConstraintTemplate;
use crate::particle::ParticleTemplate;

#[derive(Clone, Default)]
pub struct PhysicalModel {
	pub particles: Vec<ParticleTemplate>,
	pub constraints: Vec<ConstraintTemplate>,
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

impl PhysicalModel {
	#[allow(clippy::needless_range_loop)]
	pub fn new_block(
		_mass: f32,
		_x: usize,
		_y: usize,
		_size: f32,
		_compl_d: f32,
		_compl_v: f32,
	) -> Self {
		unimplemented!()
	}
}
