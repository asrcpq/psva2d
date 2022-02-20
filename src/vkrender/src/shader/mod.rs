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

pub mod fs_wf {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "src/shader/frag_wf.glsl"
	}
}
