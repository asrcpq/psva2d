use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Zeroable, Pod, Default, Debug, Clone, Copy)]
pub struct Vertex {
	pub pos: [f32; 2],
	pub tex_coord: [f32; 2],
}
vulkano::impl_vertex!(Vertex, pos, tex_coord);

#[repr(C)]
#[derive(Zeroable, Pod, Default, Debug, Clone, Copy)]
pub struct VertexText {
	pub color: [f32; 4],
	pub pos: [f32; 2],
	pub tex_coord: [f32; 2],
}
vulkano::impl_vertex!(VertexText, color, pos, tex_coord);

#[repr(C)]
#[derive(Zeroable, Pod, Default, Debug, Clone, Copy)]
pub struct VertexWf {
	pub color: [f32; 4],
	pub pos: [f32; 2],
}
vulkano::impl_vertex!(VertexWf, color, pos);
