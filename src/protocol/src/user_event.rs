use crate::pr_model::PrModel;

#[derive(Debug)]
pub enum UserEvent {
	Update(PrModel, UpdateInfo),
}

#[derive(Debug)]
pub struct UpdateInfo {
	pub load: f32,
	pub coll_len: usize,
}
