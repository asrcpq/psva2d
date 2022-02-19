#![allow(clippy::all)]

pub mod vs {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "src/shader/vert.glsl"
	}
}

pub mod fs {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "src/shader/frag.glsl"
	}
}
