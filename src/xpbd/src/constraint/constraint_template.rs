use super::distance::DistanceConstraintTemplate;
use super::volume::VolumeConstraintTemplate;

#[derive(Clone)]
pub enum ConstraintTemplate {
	Distance(DistanceConstraintTemplate),
	Volume(VolumeConstraintTemplate),
}
