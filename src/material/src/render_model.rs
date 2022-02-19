use crate::face::{Face, FaceGroup};
use protocol::pr_model::PrModel;

use std::collections::HashMap;

pub struct RenderModel {
	pub vs: HashMap<usize, [f32; 2]>,
	pub face_groups: Vec<FaceGroup>,
}
