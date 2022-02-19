pub struct Face {
	pub vid: [usize; 3],
}

pub struct FaceGroup {
	// texture_id
	pub faces: Vec<Face>,
}
