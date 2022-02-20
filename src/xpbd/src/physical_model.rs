use crate::constraint::distance::DistanceConstraint;
use crate::constraint::volume::VolumeConstraint;
use crate::constraint::Constraint;
use crate::particle::{PRef, Particle};
use crate::V2;

#[derive(Clone, Default)]
pub struct PhysicalModel {
	pub particles: Vec<PRef>,
	pub constraints: Vec<Box<dyn Constraint>>,
}

impl PhysicalModel {
	#[allow(clippy::needless_range_loop)]
	pub fn new_block(
		mass: f32,
		x: usize,
		y: usize,
		size: f32,
		compl_d: f32,
		compl_v: f32,
	) -> Self {
		let mut id_alloc = 0;
		let mut particles = vec![];
		let mut constraints = vec![];
		let mut ps = vec![];
		for idx in 0..x {
			let mut pline = vec![];
			for idy in 0..y {
				let pos = V2::new(size * idx as f32, size * idy as f32);
				let accel = V2::new(0., -9.8);
				let p = Particle::new_ref(id_alloc, mass, pos, accel);
				id_alloc += 1;
				particles.push(p.clone());
				pline.push(p);
			}
			ps.push(pline);
		}
		for idx in 1..x {
			for idy in 0..y {
				let dc = DistanceConstraint::new(
					ps[idx][idy].clone(),
					ps[idx - 1][idy].clone(),
				)
				.attractive_only()
				.with_compliance(compl_d)
				.build();
				constraints.push(dc);
			}
		}
		for idx in 0..x {
			for idy in 1..y {
				let dc = DistanceConstraint::new(
					ps[idx][idy].clone(),
					ps[idx][idy - 1].clone(),
				)
				.attractive_only()
				.with_compliance(compl_d)
				.build();
				constraints.push(dc);
			}
		}
		for idx in 1..x {
			for idy in 1..y {
				let dc = DistanceConstraint::new(
					ps[idx - 1][idy].clone(),
					ps[idx][idy - 1].clone(),
				)
				.attractive_only()
				.with_compliance(compl_d)
				.build();
				constraints.push(dc);
				let dc = DistanceConstraint::new(
					ps[idx - 1][idy - 1].clone(),
					ps[idx][idy].clone(),
				)
				.attractive_only()
				.with_compliance(compl_d)
				.build();
				constraints.push(dc);
				let vc = VolumeConstraint::new(vec![
					ps[idx][idy].clone(),
					ps[idx][idy - 1].clone(),
					ps[idx - 1][idy - 1].clone(),
				])
				.with_compliance(compl_v)
				.build();
				constraints.push(vc);
				let vc = VolumeConstraint::new(vec![
					ps[idx][idy].clone(),
					ps[idx - 1][idy].clone(),
					ps[idx - 1][idy - 1].clone(),
				])
				.with_compliance(compl_v)
				.build();
				constraints.push(vc);
				// let vc = VolumeConstraint::new(vec![
				// 	ps[idx - 1][idy].clone(),
				// 	ps[idx][idy - 1].clone(),
				// 	ps[idx - 1][idy - 1].clone(),
				// ])
				// .with_compliance(compl_v)
				// .build();
				// self.constraints.push(vc);
				// let vc = VolumeConstraint::new(vec![
				// 	ps[idx - 1][idy].clone(),
				// 	ps[idx][idy - 1].clone(),
				// 	ps[idx][idy].clone(),
				// ])
				// .with_compliance(compl_v)
				// .build();
				// self.constraints.push(vc);
			}
		}
		Self {
			particles,
			constraints,
		}
	}
}
