use std::collections::HashMap;

use crate::face::{Face, FaceGroup};
use crate::render_model::RenderModel;
use protocol::pr_model::PrModel;

pub struct TextureIndex {
	pub texture_id: usize,
	pub uvid: [usize; 3],
}

#[derive(Default)]
pub struct TextureIndexer {
	id_alloc: usize, // constraint id
	texture_map: HashMap<usize, TextureIndex>,
}

impl TextureIndexer {
	pub fn compile_model(&self, pr_model: &PrModel) -> RenderModel {
		let mut vs = HashMap::new();
		for (id, particle) in &pr_model.particles {
			vs.insert(*id, particle.pos);
		}
		let mut faces = Vec::new();
		for constraint in pr_model.constraints.iter() {
			if constraint.id > 0 {
				let texind = self.texture_map.get(&constraint.id).unwrap();
				let face = Face {
					vid: constraint.particles.clone().try_into().unwrap(),
					uvid: [0; 3], // FIXME
				};
				faces.push(face);
			} else if constraint.particles.len() == 3 { // TODO: remove
				let face = Face {
					vid: constraint.particles.clone().try_into().unwrap(),
					uvid: [0; 3],
				};
				faces.push(face);
			}
		}
		RenderModel {
			vs,
			face_groups: vec![FaceGroup { faces }],
		}
	}
}
