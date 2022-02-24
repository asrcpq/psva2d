use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

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
	texture_map: HashMap<i32, FaceInfo>,
}

pub type TextureIndexerRef = Rc<RefCell<TextureIndexer>>;

impl TextureIndexer {
	pub fn into_ref(self) -> TextureIndexerRef {
		Rc::new(RefCell::new(self))
	}

	pub fn compile_model(&self, pr_model: &PrModel) -> RenderModel {
		let mut result = RenderModel::default();
		for (id, particle) in &pr_model.particles {
			result.vs.insert(*id, particle.pos);
		}
		for constraint in pr_model.constraints.iter() {
			if constraint.particles.len() == 3 {
				let texind = if let Some(ind) = self
					.texture_map
					.get(&constraint.id)
					.cloned()
				{
					ind
				} else {
					eprintln!("Indexer constraint {} not found", constraint.id);
					FaceInfo::default()
				};
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

	pub fn add_faces(&mut self, cids: Vec<i32>, faces: HashMap<usize, FaceInfo>) {
		for (idx, face_info) in faces.into_iter() {
			let ret = self.texture_map.insert(cids[idx], face_info);
			assert!(ret.is_none());
		}
	}
}
