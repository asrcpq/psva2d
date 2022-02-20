use std::collections::HashMap;

use crate::face::{Face, FaceGroup};
use crate::render_model::RenderModel;
use protocol::pr_model::PrModel;

#[derive(Clone, Default)]
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
		let mut result = RenderModel::default();
		for (id, particle) in &pr_model.particles {
			result.vs.insert(*id, particle.pos);
		}
		for constraint in pr_model.constraints.iter() {
			if constraint.particles.len() == 3 {
				let texind = self.texture_map
					.get(&constraint.id)
					.cloned()
					.unwrap_or(TextureIndex::default());
				let face = Face {
					vid: constraint.particles.clone().try_into().unwrap(),
					uvid: texind.uvid,
				};
				let e = result.face_groups
					.entry(texind.texture_id)
					.or_insert_with(FaceGroup::default);
				e.faces.push(face);
			}
		}
		result
	}
}
