use std::collections::HashMap;

use crate::constraint::distance::DistanceConstraint;
use crate::constraint::Constraint;
use crate::particle::PRef;
use crate::{C2, V2};
use protocol::pr_model::PrParticle;

pub struct ParticleGroup {
	// Note: compress function? but maybe not necessary
	id_alloc: usize,
	csize: f32, // = 2 x radius
	shp: HashMap<C2, Vec<PRef>>,
	data: HashMap<usize, PRef>,
	speed_limit_k: f32,
}

impl Default for ParticleGroup {
	fn default() -> Self {
		Self {
			id_alloc: 0,
			csize: 0.08,
			shp: HashMap::new(),
			data: HashMap::new(),
			// particle cannot move more than k * csize in dt
			speed_limit_k: 0.7,
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
				let mut locked = pref.try_lock().unwrap();
				locked.update(dt, self.speed_limit_k * self.csize);
				// sticky ground, just for debugging
				if locked.pos[1] > 0. {
					// locked.pos[1] = -locked.pos[1];
					// locked.ppos[1] = -locked.ppos[1];
					locked.pos[1] = 0.;
					locked.ppos[1] = 0.;
					locked.ppos[0] = locked.pos[0];
				}
				locked.pos
			};
			let cpos = self.get_cpos(pos);
			let e = self.shp.entry(cpos).or_insert_with(Vec::new);
			(*e).push(pref);
		}
	}

	#[allow(clippy::needless_range_loop)]
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
						let dl = (pp1.get_pos() - pp2.get_pos()).magnitude();
						// Note: is it enough or we should make is looser?
						// during iteration more collisions could happen
						if dl > self.csize {
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
				.with_id(1)
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
				C2::new(-1, -1),
				C2::new(-1, 0),
				C2::new(-1, 1),
				C2::new(0, -1),
				C2::new(0, 0),
				C2::new(0, 1),
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
		C2::new(
			(p[0] / self.csize).floor() as i32,
			(p[1] / self.csize).floor() as i32,
		)
	}

	pub fn add_pref(&mut self, p: PRef) -> usize {
		let pos = {
			let mut p = p.try_lock().unwrap();
			p.set_id(self.id_alloc);
			p.get_pos()
		};
		assert!(self.data.insert(self.id_alloc, p.clone()).is_none());
		let cpos = self.get_cpos(pos);
		let e = self.shp.entry(cpos).or_insert_with(Vec::new);
		(*e).push(p);
		self.id_alloc += 1;
		self.id_alloc - 1
	}
}
