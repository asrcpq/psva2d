use std::sync::Arc;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewType};
use vulkano::image::{
	ImageAccess, ImageDimensions, ImageUsage, ImmutableImage, MipmapsCount,
	SwapchainImage,
};
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::input_assembly::{
	InputAssemblyState, PrimitiveTopology,
};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline};
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
use vulkano::sampler::Sampler;
use vulkano::swapchain::{Surface, Swapchain};
use vulkano::sync::GpuFuture;
use winit::window::Window;

use crate::shader;
use crate::vertex::{Vertex, VertexWf};
use material::face::TextureData;

pub type VkwInstance = Arc<Instance>;
pub type VkwDevice = Arc<Device>;
pub type VkwSurface<W> = Arc<Surface<W>>;
pub type VkwQueue = Arc<Queue>;
pub type VkwSwapchain<W> = Arc<Swapchain<W>>;
pub type VkwImages<W> = Vec<Arc<SwapchainImage<W>>>;
pub type VkwTextureSet = Arc<PersistentDescriptorSet>;
pub type VkwPipeline = Arc<GraphicsPipeline>;
pub type VkwTexCoords = Vec<Vec<[f32; 2]>>;
pub type VkwFuture = Option<Box<dyn GpuFuture>>;
pub type VkwRenderPass = Arc<RenderPass>;

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
						&& surface.is_supported(q).unwrap_or(false)
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
		&Features {
			fill_mode_non_solid: true,
			..Features::none()
		},
		&physical_device
			.required_extensions()
			.union(&device_extensions),
		[(queue_family, 0.5)].iter().cloned(),
	)
	.unwrap();

	let queue = queues.next().unwrap();

	(physical_device, device, queue)
}

pub fn get_swapchain_and_images(
	physical_device: PhysicalDevice,
	device: VkwDevice,
	queue: VkwQueue,
	surface: VkwSurface<Window>,
) -> (VkwSwapchain<Window>, VkwImages<Window>) {
	let caps = surface.capabilities(physical_device).unwrap();
	let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
	let format = caps.supported_formats[0].0;
	let dimensions: [u32; 2] = surface.window().inner_size().into();

	Swapchain::start(device, surface)
		.num_images(caps.min_image_count)
		.format(format)
		.dimensions(dimensions)
		.usage(ImageUsage::color_attachment())
		.sharing_mode(&queue)
		.composite_alpha(composite_alpha)
		.build()
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
				format: swapchain.format(),
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

pub fn get_textures(
	textures: Vec<TextureData>,
	device: VkwDevice,
	queue: VkwQueue,
	pipeline: VkwPipeline,
) -> (Arc<PersistentDescriptorSet>, VkwTexCoords, VkwFuture) {
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
		let (image, future) = ImmutableImage::from_iter(
			arrays.into_iter(),
			dimensions,
			MipmapsCount::One,
			Format::R8G8B8A8_SRGB,
			queue,
		)
		.unwrap();
		let image_view = ImageView::start(image)
			.ty(ImageViewType::Dim2dArray)
			.build()
			.unwrap();
		(image_view, future)
	};

	let sampler = Sampler::simple_repeat_linear(device).unwrap();
	let layout = pipeline.layout().descriptor_set_layouts().get(1).unwrap();
	let texture_set = PersistentDescriptorSet::new(
		layout.clone(),
		[WriteDescriptorSet::image_view_sampler(0, texture, sampler)],
	)
	.unwrap();
	let previous_frame_end = Some(tex_future.boxed());
	(texture_set, tex_coords, previous_frame_end)
}

pub fn window_size_dependent_setup(
	render_pass: Arc<RenderPass>,
	images: &[Arc<SwapchainImage<Window>>],
	viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
	let dimensions = images[0].dimensions().width_height();
	viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

	images
		.iter()
		.map(|image| {
			let view = ImageView::new(image.clone()).unwrap();
			Framebuffer::start(render_pass.clone())
				.add(view)
				.unwrap()
				.build()
				.unwrap()
		})
		.collect::<Vec<_>>()
}
