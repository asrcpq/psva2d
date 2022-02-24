use std::collections::HashMap;

use crate::constraint::leash::LeashConstraint;
use crate::constraint::Constraint;
use protocol::pr_model::PrConstraint;

#[derive(Default)]
pub struct ConstraintGroup {
	constraints: Vec<Box<dyn Constraint>>,
	tmp_constraints: Vec<Box<dyn Constraint>>,
	marionette_constraints: HashMap<usize, Box<dyn Constraint>>,
}

impl ConstraintGroup {
	pub fn add_constraint(&mut self, constraint: Box<dyn Constraint>) {
		self.constraints.push(constraint);
	}

	#[cfg(not(debug_assertions))]
	pub fn solve_constraints(&mut self, dt: f32) {
		use rayon::prelude::*;
		self.constraints
			.par_iter_mut()
			.chain(self.tmp_constraints.par_iter_mut())
			.chain(self.marionette_constraints.par_iter_mut().map(|(_k, v)| v))
			.for_each(|constraint| constraint.step(dt));
	}

	#[cfg(debug_assertions)]
	pub fn solve_constraints(&mut self, dt: f32) {
		self.constraints
			.iter_mut()
			.chain(self.tmp_constraints.iter_mut())
			.chain(self.marionette_constraints.values_mut())
			.for_each(|constraint| constraint.step(dt));
	}

	pub fn pre_iteration(&mut self) {
		for constraint in self.constraints.iter_mut() {
			constraint.pre_iteration();
		}
	}

	pub fn set_tmp_constraints(
		&mut self,
		tmp_constraints: Vec<Box<dyn Constraint>>,
	) {
		self.tmp_constraints = tmp_constraints;
	}

	pub fn tmp_len(&mut self) -> usize {
		self.tmp_constraints.len()
	}

	pub fn control_particle(&mut self, id: usize, con: LeashConstraint) {
		self.marionette_constraints.insert(id, Box::new(con));
	}

	pub fn uncontrol_particle(&mut self, id: usize) {
		self.marionette_constraints.remove(&id);
	}

	pub fn pr_constraints(&self) -> Vec<PrConstraint> {
		self.constraints
			.iter()
			.chain(self.tmp_constraints.iter())
			.chain(self.marionette_constraints.values())
			.map(|x| x.render())
			.collect()
	}
}
