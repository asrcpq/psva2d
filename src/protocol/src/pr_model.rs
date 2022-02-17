// pr_model: Physical model for rendering

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PrParticle {
	pub id: usize,
	pub pos: (f32, f32),
}

#[derive(Serialize, Deserialize)]
pub struct PrConstraint {
	pub id: usize,
	pub particles: Vec<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct PrModel {
	pub particles: Vec<PrParticle>,
	pub constraints: Vec<PrConstraint>,
}
