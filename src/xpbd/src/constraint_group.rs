use std::collections::HashMap;

use crate::constraint::leash::LeashConstraint;
use crate::constraint::CRef;
use protocol::pr_model::PrConstraint;

#[derive(Default)]
pub struct ConstraintGroup {
	id_alloc: i32,
	constraints: HashMap<i32, CRef>,
	tmp_constraints: Vec<CRef>,
	marionette_constraints: HashMap<usize, CRef>,
}

impl ConstraintGroup {
	pub fn add_constraint(&mut self, constraint: CRef) -> i32 {
		self.constraints.insert(self.id_alloc, constraint);
		self.id_alloc += 1;
		self.id_alloc - 1
	}

	pub fn len(&self) -> Vec<usize> {
		vec![
			self.constraints.len(),
			self.tmp_constraints.len(),
			self.marionette_constraints.len(),
		]
	}

	#[cfg(not(debug_assertions))]
	pub fn solve_constraints(&mut self, dt: f32) {
		use rayon::prelude::*;
		self.constraints
			.par_iter_mut()
			.map(|(_k, v)| v)
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
		for constraint in self.constraints.values_mut() {
			constraint.pre_iteration();
		}
	}

	pub fn set_tmp_constraints(
		&mut self,
		tmp_constraints: Vec<CRef>,
	) {
		self.tmp_constraints = tmp_constraints;
	}

	pub fn control_particle(&mut self, id: usize, con: LeashConstraint) {
		self.marionette_constraints.insert(id, Box::new(con));
	}

	pub fn uncontrol_particle(&mut self, id: usize) {
		self.marionette_constraints.remove(&id);
	}

	pub fn pr_constraints(&self) -> Vec<PrConstraint> {
		// NOTE: since only normal constraint has id
		// more consideration is needed for rendering special constraints
		self.constraints
			.iter()
			.map(|(v, k)| k.render(*v))
			.collect()
	}
}
