use crate::pr_model::PrModel;

#[derive(Debug)]
pub enum UserEvent {
	Update(PrModel, f32),
}
