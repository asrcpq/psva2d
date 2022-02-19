use crate::face::{Face, FaceGroup};
use protocol::pr_model::PrModel;

use std::collections::HashMap;

pub struct RenderModel {
	pub vs: HashMap<usize, [f32; 2]>,
	pub face_groups: Vec<FaceGroup>,
}

impl RenderModel {
	pub fn simple_from_pr_model(pr_model: &PrModel) -> Self {
		let mut vs = HashMap::new();
		for (id, particle) in &pr_model.particles {
			vs.insert(*id, particle.pos);
		}
		let mut faces = Vec::new();
		for constraint in pr_model.constraints.iter() {
			if constraint.particles.len() == 3 {
				let face = Face {
					vid: constraint.particles.clone().try_into().unwrap(),
				};
				faces.push(face);
			}
		}
		Self {
			vs,
			face_groups: vec![FaceGroup { faces }],
		}
	}
}
