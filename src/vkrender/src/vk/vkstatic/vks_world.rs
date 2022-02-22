use crate::vk::vkwrapper::*;
use material::face::TextureData;
use super::vks::Vks;

pub struct VksWorld {
	pub framebuffers: Vec<VkwFramebuffer>,
	pub pipeline: VkwPipeline,
	pub pipeline_wf: VkwPipeline,
	pub render_pass: VkwRenderPass,
	pub texture_set: VkwTextureSet,
	pub tex_coords: VkwTexCoords,
}

impl VksWorld {
	pub fn new(vks: &Vks, textures: Vec<TextureData>) -> Self {
		let render_pass = get_render_pass(vks.device.clone(), vks.swapchain.clone());
		let pipelines = get_pipelines(render_pass.clone(), vks.device.clone());
		let pipeline = pipelines[0].clone();
		let pipeline_wf = pipelines[1].clone();

		let framebuffers =
			window_size_dependent_setup(render_pass.clone(), &vks.images);
		let (texture_set, tex_coords) = get_textures(
			textures,
			vks.device.clone(),
			vks.queue.clone(),
			pipeline.clone(),
		);
		VksWorld {
			framebuffers,
			pipeline,
			pipeline_wf,
			render_pass,
			texture_set,
			tex_coords,
		}
	}
}
