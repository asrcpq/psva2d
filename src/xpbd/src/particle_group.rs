use std::collections::HashMap;

use crate::constraint::distance::DistanceConstraint;
use crate::constraint::Constraint;
use crate::particle::{PRef, Particle};
use crate::{C2, V2};
use protocol::pr_model::PrParticle;

pub struct ParticleGroup {
	id_alloc: usize,
	csize: f32, // = 2 x radius
	offset: V2,
	shp: HashMap<C2, Vec<PRef>>,
	data: HashMap<usize, PRef>,
}

impl Default for ParticleGroup {
	fn default() -> Self {
		Self {
			id_alloc: 0,
			csize: 0.08,
			offset: V2::new(0., 0.),
			shp: HashMap::new(),
			data: HashMap::new(),
		}
	}
}

impl ParticleGroup {
	pub fn csize(&self) -> f32 {
		self.csize
	}

	pub fn update(&mut self, dt: f32) {
		let old_shp = std::mem::take(&mut self.shp);
		for pref in old_shp.into_iter().map(|(_, p)| p).flatten() {
			let pos = {
				let mut prev_lock = pref.try_lock().unwrap();
				prev_lock.update(dt);
				prev_lock.get_pos()
			};
			let cpos = self.get_cpos(pos);
			let e = self.shp.entry(cpos).or_insert_with(Vec::new);
			(*e).push(pref);
		}
	}

	fn collcon_of_2_pvecs(
		&self,
		pv1: &[PRef],
		pv2: &[PRef],
	) -> Vec<Box<dyn Constraint>> {
		let mut result = Vec::new();
		for p1 in pv1.iter() {
			for p2 in pv2.iter() {
				{
					let pp1 = p1.try_lock().unwrap();
					if let Ok(ref mut pp2) = p2.try_lock() {
						if pp1.get_id() >= pp2.get_id() {
							continue;
						}
						// Note: is it enough or we should make is looser?
						// during iteration more collisions could happen
						if (pp1.get_pos() - pp2.get_pos()).magnitude()
							> self.csize
						{
							continue;
						}
					} else {
						continue;
					}
				}
				let collcon = DistanceConstraint::new_with_l0(
					p1.clone(),
					p2.clone(),
					self.csize,
				)
				.repulsive_only()
				.build();
				result.push(collcon);
			}
		}
		result
	}

	pub fn collision_constraints(&mut self) -> Vec<Box<dyn Constraint>> {
		let mut result = Vec::new();
		// ...
		// .xx
		// xxx
		for (cell, pvec) in &self.shp {
			if pvec.is_empty() {
				eprintln!("WARN: a cell has empty value, this is a bug");
				continue;
			}
			for dcell in vec![
				C2::new(0, 0),
				C2::new(1, 0),
				C2::new(1, -1),
				C2::new(1, 0),
				C2::new(1, 1),
			]
			.into_iter()
			{
				let cell2 = cell + dcell;
				if let Some(pvec2) = self.shp.get(&cell2) {
					let collcons = self.collcon_of_2_pvecs(pvec, pvec2);
					result.extend(collcons);
				}
			}
		}
		result
	}

	pub fn pr_particles(&self) -> HashMap<usize, PrParticle> {
		let mut result = HashMap::new();
		for (&id, p) in self.data.iter() {
			let prp = p.try_lock().unwrap().render();
			assert!(result.insert(id, prp).is_none());
		}
		result
	}

	fn get_cpos(&self, p: V2) -> C2 {
		let dp = p - self.offset;
		C2::new(
			(dp[0] / self.csize).floor() as i32,
			(dp[1] / self.csize).floor() as i32,
		)
	}

	pub fn add_particle(&mut self, mass: f32, pos: V2, accel: V2) -> PRef {
		let p = Particle::new_ref(self.id_alloc, mass, pos, accel);
		assert!(self.data.insert(self.id_alloc, p.clone()).is_none());
		let cpos = self.get_cpos(pos);
		let e = self.shp.entry(cpos).or_insert_with(Vec::new);
		(*e).push(p.clone());
		self.id_alloc += 1;
		p
	}
}
