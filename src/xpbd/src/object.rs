pub mod ObjectTopology {
	// all ccw
	// fmap: HashMap<i32, [i32; 6]>,
	// v1, v2, f1, f2
	// emap: HashMap<i32, [i32; 4]>,
	// e1, f1, e2, f2, ...
	pub vmap: HashMap<i32, Vec<[i32, 2]>>
}

pub mod Object {
	id_alloc: i32,
	texture_id: i32,
	face_constraints: HashMap<i32, CRef>,
	edge_constraints: HashMap<i32, CRef>,
	vs: HashMap<i32, PRef>,
	topology: ObjectTopology,
}
