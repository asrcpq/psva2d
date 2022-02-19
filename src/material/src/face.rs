use crate::V2;

pub struct Face {
	pub vid: [usize; 3],
	pub uvid: [usize; 3],
}

pub struct FaceGroup {
	pub faces: Vec<Face>,
}

pub struct TextureData {
	pub image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
	pub tex_coords: Vec<V2>,
}
