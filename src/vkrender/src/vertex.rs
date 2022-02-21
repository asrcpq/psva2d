#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct Vertex {
	pub pos: [f32; 2],
	pub tex_coord: [f32; 2],
}
vulkano::impl_vertex!(Vertex, pos, tex_coord);
