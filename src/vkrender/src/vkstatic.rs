use std::sync::Arc;
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::RenderPass;
use vulkano::sync::{self, GpuFuture};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::vkwrapper::*;
use material::face::TextureData;

pub struct VkStatic {
	pub device: VkwDevice,
	pub queue: VkwQueue,
	pub surface: VkwSurface<Window>,
	pub swapchain: VkwSwapchain<Window>,
	pub framebuffers: Vec<VkwFramebuffer>,
	pub framebuffers_overlay: Vec<VkwFramebuffer>,
	pub previous_frame_end: Option<VkwFuture>,
	pub pipeline: VkwPipeline,
	pub pipeline_text: VkwPipeline,
	pub pipeline_wf: VkwPipeline,
	pub render_pass: Arc<RenderPass>,
	pub render_pass_overlay: Arc<RenderPass>,
	pub texture_set: VkwTextureSet,
	pub texture_set_text: VkwTextureSet,
	pub tex_coords: VkwTexCoords,
}

fn winit_size(size: [u32; 2]) -> Size {
	Size::new(LogicalSize::new(size[0], size[1]))
}

impl VkStatic {
	pub fn new<E>(
		el: &EventLoopWindowTarget<E>,
		textures: Vec<TextureData>,
		window_size: [u32; 2],
		viewport: &mut Viewport,
	) -> Self {
		let required_extensions = vulkano_win::required_extensions();
		let instance =
			Instance::new(None, Version::V1_1, &required_extensions, None)
				.unwrap();
		let surface = WindowBuilder::new()
			.with_inner_size(winit_size(window_size))
			.with_resizable(false)
			.build_vk_surface(el, instance.clone())
			.unwrap();

		let (physical_device, device, queue) =
			get_device_and_queue(&instance, surface.clone());

		let (swapchain, images) = get_swapchain_and_images(
			physical_device,
			device.clone(),
			queue.clone(),
			surface.clone(),
		);

		let render_pass = get_render_pass(device.clone(), swapchain.clone());
		let render_pass_overlay =
			get_render_pass_overlay(device.clone(), swapchain.clone());
		let pipelines = get_pipelines(render_pass.clone(), device.clone());
		let pipeline = pipelines[0].clone();
		let pipeline_wf = pipelines[1].clone();
		let pipeline_text =
			get_pipeline_text(render_pass_overlay.clone(), device.clone());

		let framebuffers =
			window_size_dependent_setup(render_pass.clone(), &images, viewport);
		let framebuffers_overlay = window_size_dependent_setup(
			render_pass_overlay.clone(),
			&images,
			viewport,
		);
		let (texture_set, tex_coords) = get_textures(
			textures,
			device.clone(),
			queue.clone(),
			pipeline.clone(),
		);
		let texture_set_text = get_text_texture(
			device.clone(),
			queue.clone(),
			pipeline_text.clone(),
		);
		let previous_frame_end = Some(sync::now(device.clone()).boxed());
		Self {
			device,
			queue,
			surface,
			swapchain,
			framebuffers,
			framebuffers_overlay,
			previous_frame_end,
			pipeline,
			pipeline_text,
			pipeline_wf,
			render_pass,
			render_pass_overlay,
			texture_set,
			texture_set_text,
			tex_coords,
		}
	}
}
