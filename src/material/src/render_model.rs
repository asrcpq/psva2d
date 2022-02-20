use crate::face::{Face, FaceGroup};

use std::collections::HashMap;

#[derive(Default)]
pub struct RenderModel {
	pub vs: HashMap<usize, [f32; 2]>,
	pub face_groups: HashMap<usize, FaceGroup>,
}
