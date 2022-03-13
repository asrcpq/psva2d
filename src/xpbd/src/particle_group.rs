use std::collections::HashMap;
type Map<K, V> = fnv::FnvHashMap<K, V>;
// type Map<K, V> = HashMap<K, V>;

use crate::constraint::distance::DistanceConstraint;
use crate::constraint::Constraint;
use crate::particle::PRef;
use crate::posbox::Posbox;
use crate::{C2, V2};
use protocol::pr_model::PrParticle;

pub struct ParticleGroup {
	id_alloc: usize,
	csize: f32, // = 2 x radius
	shp: Map<C2, Vec<PRef>>,
	data: Map<usize, PRef>,
	speed_limit_k: f32,
	posbox: Posbox,
}

impl Default for ParticleGroup {
	fn default() -> Self {
		Self {
			id_alloc: 0,
			csize: 0.08,
			shp: Default::default(),
			data: Default::default(),
			// particle cannot move more than k * csize in dt
			speed_limit_k: 1.0,
			posbox: Posbox {
				xmin: -1e3,
				xmax: 1e3,
				ymin: -1e3,
				ymax: 1e3,
			},
		}
	}
}

impl ParticleGroup {
	pub fn with_posbox(mut self, posbox: Posbox) -> Self {
		self.posbox = posbox;
		self
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn update(&mut self, dt: f32) {
		let old_shp = std::mem::take(&mut self.shp);
		for pref in old_shp.into_iter().flat_map(|(_, p)| p) {
			let pos = {
				let mut locked = pref.try_write().unwrap();
				locked.update(dt, self.speed_limit_k * self.csize);
				if self.posbox.apply(&mut locked.pos) {
					locked.ppos = locked.pos;
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
					let pp1 = p1.read().unwrap();
					if let Ok(ref mut pp2) = p2.read() {
						if pp1.id >= pp2.id {
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
				.repulsive_only()
				.build();
				result.push(collcon);
			}
		}
		result
	}

	pub fn collision_constraints(&mut self) -> Vec<Box<dyn Constraint>> {
		use rayon::prelude::*;
		self.shp
			.par_iter()
			.flat_map(|(cell, pvec)| {
				let mut result = Vec::new();
				if pvec.is_empty() {
					eprintln!("WARN: empty cell(bug)");
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
				result.into_par_iter()
			})
			.collect()
	}

	pub fn pr_particles(&self) -> HashMap<usize, PrParticle> {
		let mut result = HashMap::default();
		for (&id, p) in self.data.iter() {
			let prp = p.try_read().unwrap().render();
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
			let mut p = p.try_write().unwrap();
			p.id = self.id_alloc;
			p.get_pos()
		};
		assert!(self.data.insert(self.id_alloc, p.clone()).is_none());
		let cpos = self.get_cpos(pos);
		let e = self.shp.entry(cpos).or_insert_with(Vec::new);
		(*e).push(p);
		self.id_alloc += 1;
		self.id_alloc - 1
	}

	pub fn get_pref(&self, id: usize) -> Option<PRef> {
		self.data.get(&id).cloned()
	}
}
