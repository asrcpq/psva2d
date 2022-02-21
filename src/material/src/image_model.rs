use crate::face::TextureData;
use crate::texture_indexer::{FaceInfo, TextureIndexer};
use crate::V2;
use xpbd::constraint::constraint_template::ConstraintTemplate::{
	Distance, Volume,
};
use xpbd::constraint::distance::{
	DistanceConstraintTemplate, DistanceConstraintType as DCTy,
};
use xpbd::constraint::volume::VolumeConstraintTemplate;
use xpbd::particle::ParticleTemplate;
use xpbd::physical_model::PhysicalModel;

#[derive(Clone)]
struct Cell {
	pub pid: usize,
	// expand: for distinguish expand particles in neighour check
	pub expand: bool,
}

pub struct ImageModelBuilder {
	len: [isize; 2],
	grid_size: [isize; 2],
	csize: f32,
	texture_id: i32,
	image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
	indexer: TextureIndexer,
	cells: Vec<Vec<Option<Cell>>>,
	particles: Vec<ParticleTemplate>,
	pid_alloc: usize,
	tex_coords: Vec<V2>,
}

impl ImageModelBuilder {
	pub fn new(
		texture_id: i32,
		indexer: TextureIndexer,
		image_path: &str,
	) -> Self {
		eprintln!("INFO: Loading {}", image_path);
		let image = image::open(image_path).unwrap().into_rgba8();
		let len = [32, 32];
		assert_eq!(image.width(), 1024);
		assert_eq!(image.height(), 1024);
		Self {
			len,
			grid_size: [1024 / len[0], 1024 / len[1]],
			csize: 0.08,
			texture_id,
			indexer,
			image,
			cells: vec![vec![None; len[1] as usize]; len[0] as usize],
			particles: Vec::new(),
			pid_alloc: 0,
			tex_coords: Vec::new(),
		}
	}

	pub fn add_particle(&mut self, idx: isize, idy: isize, color_check: bool) {
		let x = (idx * self.grid_size[0]) as u32;
		let y = (idy * self.grid_size[1]) as u32;
		if color_check {
			let color = self.image.get_pixel(x, y);
			if color[3] == 0 {
				return;
			}
		}
		let imass = 1.0;
		let pos = V2::new(self.csize * idx as f32, self.csize * idy as f32);
		let p = ParticleTemplate { imass, pos };
		self.particles.push(p);
		self.cells[idx as usize][idy as usize] = Some(Cell {
			pid: self.pid_alloc,
			expand: !color_check,
		});
		self.tex_coords
			.push(V2::new(x as f32 / 1024f32, y as f32 / 1024f32));
		self.pid_alloc += 1;
	}

	pub fn compute_cells(&mut self) {
		for idx in 0..self.len[0] {
			for idy in 0..self.len[1] {
				self.add_particle(idx, idy, true);
			}
		}
	}

	pub fn expand_cells(&mut self) {
		let offsets = vec![[0, -1], [0, 1], [-1, 0], [1, 0]];
		for idx in 0..self.len[0] {
			for idy in 0..self.len[1] {
				if self.cells[idx as usize][idy as usize].is_some() {
					continue;
				}
				let cells = self.get_cell(&offsets, false, idx, idy);
				if cells.iter().any(|x| !x.expand) {
					self.add_particle(idx, idy, false);
				}
			}
		}
	}

	fn get_cell(
		&self,
		offsets: &[[isize; 2]],
		must_all: bool,
		idx: isize,
		idy: isize,
	) -> Vec<Cell> {
		let mut pvec_tmp = vec![];
		for offset in offsets.iter() {
			let x = idx + offset[0];
			let y = idy + offset[1];
			if x >= 0 && x < self.len[0] && y >= 0 && y < self.len[1] {
				if let Some(cell) = &self.cells[x as usize][y as usize] {
					pvec_tmp.push(cell.clone());
					continue;
				}
			}
			if must_all {
				return Vec::new();
			}
		}
		pvec_tmp
	}

	fn get_cells(&self, offsets: Vec<[isize; 2]>) -> Vec<Vec<Cell>> {
		let mut result = vec![];
		for idx in 0..self.len[0] as isize {
			for idy in 0..self.len[1] as isize {
				let pvec_tmp = self.get_cell(&offsets, true, idx, idy);
				if !pvec_tmp.is_empty() {
					result.push(pvec_tmp);
				}
			}
		}
		result
	}

	pub fn build_physical_model(&mut self) -> PhysicalModel {
		let mut constraints = vec![];
		let mut pairs = vec![];
		pairs.extend(self.get_cells(vec![[0, 0], [-1, 0]]));
		pairs.extend(self.get_cells(vec![[0, 0], [0, -1]]));
		pairs.extend(self.get_cells(vec![[0, 0], [-1, -1]]));
		pairs.extend(self.get_cells(vec![[0, -1], [-1, 0]]));
		pairs.into_iter().for_each(|v| {
			let pos0 = self.particles[v[0].pid].pos;
			let pos1 = self.particles[v[1].pid].pos;
			let dc = DistanceConstraintTemplate {
				id: -1,
				l0: (pos0 - pos1).magnitude(),
				ps: vec![v[0].pid, v[1].pid],
				compliance: 1e-5,
				ty: DCTy::Attractive,
			};
			constraints.push(Distance(dc));
		});

		let mut pairs = vec![];
		pairs.extend(self.get_cells(vec![[0, 0], [-1, 0], [-1, -1]]));
		pairs.extend(self.get_cells(vec![[0, 0], [0, -1], [-1, -1]]));
		pairs.into_iter().for_each(|v| {
			let ps = vec![v[0].pid, v[1].pid, v[2].pid];
			let cid = self.indexer.alloc_id(FaceInfo {
				texture_id: self.texture_id,
				uvid: ps.clone().try_into().unwrap(),
			});
			let vc = VolumeConstraintTemplate {
				id: cid,
				ps,
				compliance: 1e-7,
			};
			constraints.push(Volume(vc));
		});
		PhysicalModel {
			particles: std::mem::take(&mut self.particles),
			constraints,
		}
	}

	pub fn finish(self) -> (TextureData, TextureIndexer) {
		let td = TextureData {
			tex_coords: self.tex_coords,
			image: self.image,
		};
		(td, self.indexer)
	}
}
