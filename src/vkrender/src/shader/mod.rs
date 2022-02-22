#![allow(clippy::all)]

pub mod vs {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "src/shader/normal/vert.glsl"
	}
}

pub mod fs {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "src/shader/normal/frag.glsl"
	}
}

pub mod vs_wf {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "src/shader/wireframe/vert.glsl"
	}
}

pub mod fs_wf {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "src/shader/wireframe/frag.glsl"
	}
}

pub mod vs_text {
	vulkano_shaders::shader! {
		ty: "vertex",
		path: "src/shader/text/vert.glsl"
	}
}

pub mod fs_text {
	vulkano_shaders::shader! {
		ty: "fragment",
		path: "src/shader/text/frag.glsl"
	}
}
