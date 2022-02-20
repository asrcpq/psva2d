use xpbd::V2;
use xpbd::physical_model::PhysicalModel;
use xpbd::particle::{Particle, PRef};
use xpbd::constraint::distance::DistanceConstraint;
use xpbd::constraint::volume::VolumeConstraint;

#[derive(Clone)]
struct Cell {
	pub uvid: usize,
	pub pref: PRef,
}

pub struct ImageModelBuilder {
	len: [isize; 2],
	grid_size: [isize; 2],
	csize: f32,
	image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
	cells: Vec<Vec<Option<Cell>>>,
	id_alloc: usize,
}

impl ImageModelBuilder {
	pub fn load_image(image_path: &str) -> Self {
		eprintln!("Loading {}", image_path);
		let image = image::open(image_path).unwrap().into_rgba8();
		let len = [64, 64];
		assert_eq!(image.width(), 1024);
		assert_eq!(image.height(), 1024);
		Self {
			len,
			grid_size: [1024 / len[0], 1024 / len[1]],
			csize: 0.1,
			image,
			cells: vec![vec![None; len[1] as usize]; len[0] as usize],
			id_alloc: 0,
		}
	}

	pub fn compute_cells(&mut self) {
		for idx in 0..self.len[0] {
			for idy in 0..self.len[1] {
				let color = self.image.get_pixel(
					(idx * self.grid_size[0]) as u32,
					(idy * self.grid_size[1]) as u32,
				);
				if color[3] == 0 { continue }
				let mass = 1.0;
				let pos = V2::new(self.csize * idx as f32, self.csize * idy as f32);
				let accel = V2::new(0., -9.8);
				let p = Particle::new_ref(self.id_alloc, mass, pos, accel);
				self.cells[idx as usize][idy as usize] = Some(Cell {
					uvid: self.id_alloc,
					pref: p,
				});
				self.id_alloc += 1;
			}
		}
	}

	fn get_particles(&self, offsets: Vec<[isize; 2]>) -> Vec<Vec<PRef>> {
		let mut result = vec![];
		for idx in 0..self.len[0] as isize {
			'cell_loop: for idy in 0..self.len[1] as isize {
				let mut pvec_tmp = vec![];
				for offset in offsets.iter() {
					let x = idx + offset[0];
					if x < 0 || x >= self.len[0] {
						continue 'cell_loop
					}
					let y = idy + offset[1];
					if y < 0 || y >= self.len[0] {
						continue 'cell_loop
					}
					if let Some(cell) = &self.cells[x as usize][y as usize] {
						pvec_tmp.push(cell.pref.clone());
					} else {
						continue 'cell_loop
					}
				}
				result.push(pvec_tmp);
			}
		}
		result
	}

	pub fn build_physical_model(&self) -> PhysicalModel {
		let mut particles: Vec<PRef> = vec![];
		let mut constraints = vec![];
		self.get_particles(vec![[0, 0]])
			.into_iter()
			.for_each(|v| particles.push(v[0].clone()));
		let mut pairs = vec![];
		pairs.extend(self.get_particles(vec![[0, 0], [-1, 0]]));
		pairs.extend(self.get_particles(vec![[0, 0], [0, -1]]));
		pairs.extend(self.get_particles(vec![[0, 0], [-1, -1]]));
		pairs.extend(self.get_particles(vec![[0, -1], [-1, 0]]));
		pairs.into_iter()
			.for_each(|v| {
				let dc = DistanceConstraint::new(v[0].clone(), v[1].clone())
					.attractive_only()
					.with_compliance(1e-7)
					.build();
				constraints.push(dc);
			});

		let mut pairs = vec![];
		pairs.extend(self.get_particles(vec![[0, 0], [-1, 0], [-1, -1]]));
		pairs.extend(self.get_particles(vec![[0, 0], [0, -1], [-1, -1]]));
		pairs.into_iter()
			.for_each(|v| {
				let vc = VolumeConstraint::new(vec![v[0].clone(), v[1].clone(), v[2].clone()])
					.with_id(0)
					.with_compliance(1e-10)
					.build();
				constraints.push(vc);
			});
		PhysicalModel {
			particles,
			constraints,
		}
	}
}
