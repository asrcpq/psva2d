use crate::vk::vkwrapper::*;
use super::vks::Vks;

pub struct VksOverlay {
	pub framebuffers: Vec<VkwFramebuffer>,
	pub pipeline_text: VkwPipeline,
	pub texture_set_text: VkwTextureSet,
	pub render_pass: VkwRenderPass,
}

impl VksOverlay {
	pub fn new(vks: &Vks) -> Self {
		let render_pass = get_render_pass_overlay(vks.device.clone(), vks.swapchain.clone());
		let pipeline_text =
			get_pipeline_text(render_pass.clone(), vks.device.clone());
		let framebuffers = window_size_dependent_setup(
			render_pass.clone(),
			&vks.images,
		);
		let texture_set_text = get_text_texture(
			vks.device.clone(),
			vks.queue.clone(),
			pipeline_text.clone(),
		);
		VksOverlay {
			framebuffers,
			pipeline_text,
			render_pass,
			texture_set_text,
		}
	}
}
