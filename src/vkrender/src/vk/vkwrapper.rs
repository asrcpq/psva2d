use std::sync::Arc;
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, PrimaryAutoCommandBuffer,
};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{
	Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
	QueueCreateInfo,
};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::image::{
	ImageDimensions, ImageUsage, ImmutableImage, MipmapsCount, SwapchainImage,
};
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::color_blend::ColorBlendState;
use vulkano::pipeline::graphics::input_assembly::{
	InputAssemblyState, PrimitiveTopology,
};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::{GraphicsPipeline, Pipeline};
use vulkano::render_pass::{
	Framebuffer, FramebufferCreateInfo, RenderPass, Subpass,
};
use vulkano::sampler::{Sampler, SamplerCreateInfo};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::sync::GpuFuture;
use winit::window::Window;

use crate::shader;
use crate::vertex::{Vertex, VertexText, VertexWf};
use material::face::TextureData;

pub type VkwCommandBuilder = AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>;
pub type VkwDevice = Arc<Device>;
pub type VkwFramebuffer = Arc<Framebuffer>;
pub type VkwFuture = Box<dyn GpuFuture>;
pub type VkwImages = Vec<Arc<SwapchainImage<Window>>>;
pub type VkwInstance = Arc<Instance>;
pub type VkwPipeline = Arc<GraphicsPipeline>;
pub type VkwQueue = Arc<Queue>;
pub type VkwRenderPass = Arc<RenderPass>;
pub type VkwSurface<W> = Arc<Surface<W>>;
pub type VkwSwapchain<W> = Arc<Swapchain<W>>;
pub type VkwTexCoords = Vec<Vec<[f32; 2]>>;
pub type VkwTextureSet = Arc<PersistentDescriptorSet>;

pub fn get_device_and_queue<W>(
	instance: &VkwInstance,
	surface: VkwSurface<W>,
) -> (PhysicalDevice, VkwDevice, VkwQueue) {
	let device_extensions = DeviceExtensions {
		khr_swapchain: true,
		..DeviceExtensions::none()
	};
	let (physical_device, queue_family) = PhysicalDevice::enumerate(instance)
		.filter(|&p| {
			p.supported_extensions().is_superset_of(&device_extensions)
		})
		.filter_map(|p| {
			p.queue_families()
				.find(|&q| {
					q.supports_graphics()
						&& q.supports_surface(&surface).unwrap_or(false)
				})
				.map(|q| (p, q))
		})
		.min_by_key(|(p, _)| match p.properties().device_type {
			PhysicalDeviceType::DiscreteGpu => 0,
			PhysicalDeviceType::IntegratedGpu => 1,
			PhysicalDeviceType::VirtualGpu => 2,
			PhysicalDeviceType::Cpu => 3,
			PhysicalDeviceType::Other => 4,
		})
		.unwrap();

	println!(
		"Using device: {} (type: {:?})",
		physical_device.properties().device_name,
		physical_device.properties().device_type,
	);

	let (device, mut queues) = Device::new(
		physical_device,
		DeviceCreateInfo {
			enabled_extensions: physical_device
				.required_extensions()
				.union(&device_extensions),
			enabled_features: Features {
				fill_mode_non_solid: true,
				..Features::none()
			},
			queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
			..Default::default()
		},
	)
	.unwrap();

	let queue = queues.next().unwrap();

	(physical_device, device, queue)
}

pub fn get_swapchain_and_images(
	physical_device: PhysicalDevice,
	device: VkwDevice,
	surface: VkwSurface<Window>,
) -> (VkwSwapchain<Window>, VkwImages) {
	let caps = physical_device
		.surface_capabilities(&surface, Default::default())
		.unwrap();
	let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
	let format = physical_device
		.surface_formats(&surface, Default::default())
		.unwrap()[0]
		.0;
	let format = Some(format);
	let dimensions: [u32; 2] = surface.window().inner_size().into();

	Swapchain::new(
		device,
		surface,
		SwapchainCreateInfo {
			min_image_count: caps.min_image_count,
			image_format: format,
			image_extent: dimensions,
			image_usage: ImageUsage::color_attachment(),
			composite_alpha,
			..Default::default()
		},
	)
	.unwrap()
}

pub fn get_render_pass<W>(
	device: VkwDevice,
	swapchain: VkwSwapchain<W>,
) -> VkwRenderPass {
	vulkano::single_pass_renderpass!(
		device,
		attachments: {
			color: {
				load: Clear,
				store: Store,
				format: swapchain.image_format(),
				samples: 1,
			}
		},
		pass: {
			color: [color],
			depth_stencil: {}
		}
	)
	.unwrap()
}

pub fn get_render_pass_overlay<W>(
	device: VkwDevice,
	swapchain: VkwSwapchain<W>,
) -> VkwRenderPass {
	vulkano::single_pass_renderpass!(
		device,
		attachments: {
			color: {
				load: Load,
				store: Store,
				format: swapchain.image_format(),
				samples: 1,
			}
		},
		pass: {
			color: [color],
			depth_stencil: {}
		}
	)
	.unwrap()
}

pub fn get_pipeline_text(
	render_pass: VkwRenderPass,
	device: VkwDevice,
) -> VkwPipeline {
	let vs_text = shader::vs_text::load(device.clone()).unwrap();
	let fs_text = shader::fs_text::load(device.clone()).unwrap();
	let pipeline_text = GraphicsPipeline::start()
		.color_blend_state(ColorBlendState::default().blend_alpha())
		.vertex_input_state(BuffersDefinition::new().vertex::<VertexText>())
		.vertex_shader(vs_text.entry_point("main").unwrap(), ())
		.input_assembly_state(
			InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
		)
		.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
		.fragment_shader(fs_text.entry_point("main").unwrap(), ())
		.render_pass(Subpass::from(render_pass, 0).unwrap())
		.build(device)
		.unwrap();

	pipeline_text
}

pub fn get_pipelines(
	render_pass: VkwRenderPass,
	device: VkwDevice,
) -> Vec<VkwPipeline> {
	let vs = shader::vs::load(device.clone()).unwrap();
	let fs = shader::fs::load(device.clone()).unwrap();
	let pipeline = GraphicsPipeline::start()
		.vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
		.vertex_shader(vs.entry_point("main").unwrap(), ())
		.input_assembly_state(
			InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
		)
		.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
		.fragment_shader(fs.entry_point("main").unwrap(), ())
		.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
		.build(device.clone())
		.unwrap();

	let vs_wf = shader::vs_wf::load(device.clone()).unwrap();
	let fs_wf = shader::fs_wf::load(device.clone()).unwrap();
	let pipeline_wf = GraphicsPipeline::start()
		.vertex_input_state(BuffersDefinition::new().vertex::<VertexWf>())
		.vertex_shader(vs_wf.entry_point("main").unwrap(), ())
		.input_assembly_state(
			InputAssemblyState::new().topology(PrimitiveTopology::LineList),
		)
		.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
		.fragment_shader(fs_wf.entry_point("main").unwrap(), ())
		.render_pass(Subpass::from(render_pass, 0).unwrap())
		.build(device)
		.unwrap();

	vec![pipeline, pipeline_wf]
}

pub fn get_text_texture(
	device: VkwDevice,
	queue: VkwQueue,
	pipeline: VkwPipeline,
) -> Arc<PersistentDescriptorSet> {
	let (texture, tex_future) = {
		let dimensions = ImageDimensions::Dim2d {
			width: 1024,
			height: 1024,
			array_layers: 1,
		};
		let array: Vec<u8> = image::open("assets/images/font.png")
			.unwrap()
			.into_luma8()
			.into_raw();
		let (image, future) = ImmutableImage::from_iter(
			array.into_iter(),
			dimensions,
			MipmapsCount::One,
			Format::R8_UNORM,
			queue,
		)
		.unwrap();
		let image_view = ImageView::new_default(image).unwrap();
		(image_view, future)
	};

	let sampler =
		Sampler::new(device, SamplerCreateInfo::simple_repeat_linear())
			.unwrap();
	let layout = pipeline.layout().set_layouts().get(0).unwrap();
	let texture_set = PersistentDescriptorSet::new(
		layout.clone(),
		[WriteDescriptorSet::image_view_sampler(0, texture, sampler)],
	)
	.unwrap();
	tex_future.flush().unwrap();
	texture_set
}

pub fn get_textures(
	textures: Vec<TextureData>,
	device: VkwDevice,
	queue: VkwQueue,
	pipeline: VkwPipeline,
) -> (Arc<PersistentDescriptorSet>, VkwTexCoords) {
	let tex_len = textures.len() as u32;
	let (arrays, tex_coords): (Vec<Vec<u8>>, Vec<Vec<[f32; 2]>>) = textures
		.into_iter()
		.map(|t| {
			(
				t.image.as_raw().clone(),
				t.tex_coords
					.into_iter()
					.map(|x| x.into())
					.collect::<Vec<[f32; 2]>>(),
			)
		})
		.unzip();
	let (texture, tex_future) = {
		let dimensions = ImageDimensions::Dim2d {
			width: 1024,
			height: 1024,
			array_layers: tex_len,
		};
		#[allow(clippy::needless_collect)]
		let arrays: Vec<u8> = arrays.into_iter().flat_map(|x| x.into_iter()).collect();
		let format = Format::R8G8B8A8_SRGB;
		// TODO: default texture, solve size=0 problem
		let (image, future) = ImmutableImage::from_iter(
			arrays.into_iter(),
			dimensions,
			MipmapsCount::One,
			format,
			queue,
		)
		.unwrap();
		let image_view = ImageView::new(
			image.clone(),
			ImageViewCreateInfo {
				view_type: ImageViewType::Dim2dArray,
				..ImageViewCreateInfo::from_image(&image)
			},
		)
		.unwrap();
		(image_view, future)
	};

	let sampler =
		Sampler::new(device, SamplerCreateInfo::simple_repeat_linear())
			.unwrap();

	let layout = pipeline.layout().set_layouts().get(1).unwrap();
	let texture_set = PersistentDescriptorSet::new(
		layout.clone(),
		[WriteDescriptorSet::image_view_sampler(0, texture, sampler)],
	)
	.unwrap();
	tex_future.flush().unwrap();
	(texture_set, tex_coords)
}

pub fn window_size_dependent_setup(
	render_pass: VkwRenderPass,
	images: &VkwImages,
) -> Vec<VkwFramebuffer> {
	images
		.iter()
		.map(|image| {
			let view = ImageView::new_default(image.clone()).unwrap();
			Framebuffer::new(
				render_pass.clone(),
				FramebufferCreateInfo {
					attachments: vec![view],
					..Default::default()
				},
			)
			.unwrap()
		})
		.collect::<Vec<_>>()
}
