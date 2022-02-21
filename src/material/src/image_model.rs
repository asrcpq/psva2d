use crate::face::TextureData;
use crate::texture_indexer::{FaceInfo, TextureIndexer};
use crate::V2;
use xpbd::constraint::distance::DistanceConstraint;
use xpbd::constraint::volume::VolumeConstraint;
use xpbd::particle::{PRef, Particle};
use xpbd::physical_model::PhysicalModel;

#[derive(Clone)]
struct Cell {
	pub uvid: usize,
	pub pref: PRef,
}

pub struct ImageModelBuilder {
	len: [isize; 2],
	grid_size: [isize; 2],
	csize: f32,
	texture_id: i32,
	image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
	indexer: TextureIndexer,
	cells: Vec<Vec<Option<Cell>>>,
	pid_alloc: usize,
	tex_coords: Vec<V2>,
}

impl ImageModelBuilder {
	pub fn new(
		texture_id: i32,
		indexer: TextureIndexer,
		image_path: &str,
	) -> Self {
		eprintln!("Loading {}", image_path);
		let image = image::open(image_path).unwrap().into_rgba8();
		let len = [32, 32];
		assert_eq!(image.width(), 1024);
		assert_eq!(image.height(), 1024);
		Self {
			len,
			grid_size: [1024 / len[0], 1024 / len[1]],
			csize: 0.1,
			texture_id,
			indexer,
			image,
			cells: vec![vec![None; len[1] as usize]; len[0] as usize],
			pid_alloc: 0,
			tex_coords: Vec::new(),
		}
	}

	pub fn compute_cells(&mut self) {
		for idx in 0..self.len[0] {
			for idy in 0..self.len[1] {
				let x = (idx * self.grid_size[0]) as u32;
				let y = (idy * self.grid_size[1]) as u32;
				let color = self.image.get_pixel(x, y);
				if color[3] == 0 {
					continue;
				}
				let mass = 1.0;
				let pos =
					V2::new(self.csize * idx as f32, self.csize * idy as f32);
				let accel = V2::new(0., 9.8);
				let p = Particle::new_ref(self.pid_alloc, mass, pos, accel);
				self.cells[idx as usize][idy as usize] = Some(Cell {
					uvid: self.pid_alloc,
					pref: p,
				});
				self.tex_coords
					.push(V2::new(x as f32 / 1024f32, y as f32 / 1024f32));
				self.pid_alloc += 1;
			}
		}
	}

	fn get_cells(&self, offsets: Vec<[isize; 2]>) -> Vec<Vec<Cell>> {
		let mut result = vec![];
		for idx in 0..self.len[0] as isize {
			'cell_loop: for idy in 0..self.len[1] as isize {
				let mut pvec_tmp = vec![];
				for offset in offsets.iter() {
					let x = idx + offset[0];
					if x < 0 || x >= self.len[0] {
						continue 'cell_loop;
					}
					let y = idy + offset[1];
					if y < 0 || y >= self.len[0] {
						continue 'cell_loop;
					}
					if let Some(cell) = &self.cells[x as usize][y as usize] {
						pvec_tmp.push(cell.clone());
					} else {
						continue 'cell_loop;
					}
				}
				result.push(pvec_tmp);
			}
		}
		result
	}

	pub fn build_physical_model(&mut self) -> PhysicalModel {
		let mut particles: Vec<PRef> = vec![];
		let mut constraints = vec![];
		self.get_cells(vec![[0, 0]])
			.into_iter()
			.for_each(|v| particles.push(v[0].pref.clone()));
		let mut pairs = vec![];
		pairs.extend(self.get_cells(vec![[0, 0], [-1, 0]]));
		pairs.extend(self.get_cells(vec![[0, 0], [0, -1]]));
		pairs.extend(self.get_cells(vec![[0, 0], [-1, -1]]));
		pairs.extend(self.get_cells(vec![[0, -1], [-1, 0]]));
		pairs.into_iter().for_each(|v| {
			let dc =
				DistanceConstraint::new(v[0].pref.clone(), v[1].pref.clone())
					.attractive_only()
					.with_compliance(1e-5)
					.build();
			constraints.push(dc);
		});

		let mut pairs = vec![];
		pairs.extend(self.get_cells(vec![[0, 0], [-1, 0], [-1, -1]]));
		pairs.extend(self.get_cells(vec![[0, 0], [0, -1], [-1, -1]]));
		pairs.into_iter().for_each(|v| {
			let mut data: Vec<(usize, PRef)> = vec![
				(v[0].uvid, v[0].pref.clone()),
				(v[1].uvid, v[1].pref.clone()),
				(v[2].uvid, v[2].pref.clone()),
			];
			data.sort_by_key(|x| x.0);
			let (uvid, prefs): (Vec<usize>, _) = data.into_iter().unzip();
			let cid = self.indexer.alloc_id(FaceInfo {
				texture_id: self.texture_id,
				uvid: uvid.try_into().unwrap(),
			});
			let vc = VolumeConstraint::new(prefs)
				.with_id(cid)
				.with_compliance(1e-7)
				.build();
			constraints.push(vc);
		});
		PhysicalModel {
			particles,
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
