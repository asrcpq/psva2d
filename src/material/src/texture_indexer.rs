use std::collections::HashMap;

use crate::face::{Face, FaceGroup};
use crate::render_model::RenderModel;
use protocol::pr_model::PrModel;

#[derive(Clone)]
pub struct FaceInfo {
	pub texture_id: i32,
	pub uvid: [usize; 3],
}

impl Default for FaceInfo {
	fn default() -> Self {
		Self {
			texture_id: -1,
			uvid: [0; 3],
		}
	}
}

#[derive(Default)]
pub struct TextureIndexer {
	id_alloc: usize, // constraint id
	texture_map: HashMap<usize, FaceInfo>,
}

impl TextureIndexer {
	pub fn compile_model(&self, pr_model: &PrModel) -> RenderModel {
		let mut result = RenderModel::default();
		for (id, particle) in &pr_model.particles {
			result.vs.insert(*id, particle.pos);
		}
		for constraint in pr_model.constraints.iter() {
			if constraint.particles.len() == 3 {
				let texind = self
					.texture_map
					.get(&constraint.id)
					.cloned()
					.unwrap_or_default();
				let face = Face {
					vid: constraint.particles.clone().try_into().unwrap(),
					uvid: texind.uvid,
				};
				let e = result
					.face_groups
					.entry(texind.texture_id)
					.or_insert_with(FaceGroup::default);
				e.faces.push(face);
			}
		}
		result
	}

	pub fn alloc_id(&mut self, info: FaceInfo) -> usize {
		self.texture_map.insert(self.id_alloc, info);
		self.id_alloc += 1;
		self.id_alloc - 1
	}
}
