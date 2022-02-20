use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents,
};
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
use vulkano::pipeline::graphics::rasterization::{
	PolygonMode, RasterizationState,
};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
use vulkano::sampler::Sampler;
use vulkano::swapchain::{
	self, AcquireError, Surface, Swapchain, SwapchainCreationError,
};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::camera::Camera;
use crate::shader;
use material::face::TextureData;
use material::render_model::RenderModel;

#[repr(C)]
#[derive(Default, Debug, Clone)]
struct Vertex {
	pos: [f32; 2],
	tex_coord: [f32; 2],
}
vulkano::impl_vertex!(Vertex, pos, tex_coord);

fn winit_size(size: [u32; 2]) -> Size {
	Size::new(LogicalSize::new(size[0], size[1]))
}

#[derive(PartialEq)]
pub enum VkRenderMode {
	Normal,
	Wireframe,
}

pub struct VkRender {
	pub recreate_swapchain: bool,

	device: Arc<Device>,
	queue: Arc<Queue>,
	surface: Arc<Surface<Window>>,
	swapchain: Arc<Swapchain<Window>>,
	framebuffers: Vec<Arc<Framebuffer>>,
	viewport: Viewport,
	//vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
	previous_frame_end: Option<Box<dyn GpuFuture>>,
	pipeline: Arc<GraphicsPipeline>,
	pipeline_wf: Arc<GraphicsPipeline>,
	render_pass: Arc<RenderPass>,
	texture_set: Arc<PersistentDescriptorSet>,
	tex_coords: Vec<Vec<[f32; 2]>>,

	render_mode: VkRenderMode,
}

impl VkRender {
	pub fn new(
		el: &EventLoopWindowTarget<protocol::pr_model::PrModel>,
		textures: Vec<TextureData>,
		window_size: [u32; 2],
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
		let device_extensions = DeviceExtensions {
			khr_swapchain: true,
			..DeviceExtensions::none()
		};
		let (physical_device, queue_family) =
			PhysicalDevice::enumerate(&instance)
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

		let (swapchain, images) = {
			let caps = surface.capabilities(physical_device).unwrap();
			let composite_alpha =
				caps.supported_composite_alpha.iter().next().unwrap();
			let format = caps.supported_formats[0].0;
			let dimensions: [u32; 2] = surface.window().inner_size().into();

			Swapchain::start(device.clone(), surface.clone())
				.num_images(caps.min_image_count)
				.format(format)
				.dimensions(dimensions)
				.usage(ImageUsage::color_attachment())
				.sharing_mode(&queue)
				.composite_alpha(composite_alpha)
				.build()
				.unwrap()
		};

		let vs = shader::vs::load(device.clone()).unwrap();
		let fs = shader::fs::load(device.clone()).unwrap();
		let fs_wf = shader::fs_wf::load(device.clone()).unwrap();

		let render_pass = vulkano::single_pass_renderpass!(
			device.clone(),
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
		.unwrap();

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
				MipmapsCount::Log2,
				Format::R8G8B8A8_SRGB,
				queue.clone(),
			)
			.unwrap();
			let image_view = ImageView::start(image)
				.ty(ImageViewType::Dim2dArray)
				.build()
				.unwrap();
			(image_view, future)
		};

		let pipeline = GraphicsPipeline::start()
			.vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
			.vertex_shader(vs.entry_point("main").unwrap(), ())
			.input_assembly_state(
				InputAssemblyState::new()
					.topology(PrimitiveTopology::TriangleList),
			)
			.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
			.fragment_shader(fs.entry_point("main").unwrap(), ())
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
			.build(device.clone())
			.unwrap();

		let sampler = Sampler::simple_repeat_linear(device.clone()).unwrap();
		let layout = pipeline.layout().descriptor_set_layouts().get(1).unwrap();
		let texture_set = PersistentDescriptorSet::new(
			layout.clone(),
			[WriteDescriptorSet::image_view_sampler(0, texture, sampler)],
		)
		.unwrap();

		let pipeline_wf = GraphicsPipeline::start()
			.vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
			.vertex_shader(vs.entry_point("main").unwrap(), ())
			.input_assembly_state(
				InputAssemblyState::new()
					.topology(PrimitiveTopology::TriangleList),
			)
			.viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
			.rasterization_state(
				RasterizationState::new().polygon_mode(PolygonMode::Line),
			)
			.fragment_shader(fs_wf.entry_point("main").unwrap(), ())
			.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
			.build(device.clone())
			.unwrap();

		let mut viewport = Viewport {
			origin: [0.0, 0.0],
			dimensions: [0.0, 0.0],
			depth_range: 0.0..1.0,
		};

		let framebuffers = window_size_dependent_setup(
			render_pass.clone(),
			&images,
			&mut viewport,
		);
		let previous_frame_end = Some(tex_future.boxed());

		Self {
			device,
			queue,
			surface,
			swapchain,
			recreate_swapchain: false,
			framebuffers,
			viewport,
			previous_frame_end,
			pipeline,
			pipeline_wf,
			render_pass,
			texture_set,
			tex_coords,

			render_mode: VkRenderMode::Normal,
		}
	}

	pub fn toggle_render_mode(&mut self) {
		if self.render_mode == VkRenderMode::Normal {
			self.render_mode = VkRenderMode::Wireframe;
		} else {
			self.render_mode = VkRenderMode::Normal;
		}
	}

	pub fn get_pipeline(&self) -> Arc<GraphicsPipeline> {
		if self.render_mode == VkRenderMode::Normal {
			self.pipeline.clone()
		} else {
			self.pipeline_wf.clone()
		}
	}

	pub fn render(&mut self, render_model: RenderModel, camera: Camera) {
		let mut vertex_buffers = vec![];
		for (id, face_group) in render_model.face_groups {
			if id < 0 || id >= self.tex_coords.len() as i32 {
				continue;
			}
			let vertex_buffer = CpuAccessibleBuffer::from_iter(
				self.device.clone(),
				BufferUsage::all(),
				false,
				face_group
					.faces
					.iter()
					.map(|x| {
						(0..3).map(|i| Vertex {
							pos: *render_model.vs.get(&x.vid[i]).unwrap(),
							tex_coord: self.tex_coords[id as usize][x.uvid[i]],
						})
					})
					.flatten()
					.collect::<Vec<Vertex>>()
					.into_iter(),
			)
			.unwrap();
			vertex_buffers.push((id, vertex_buffer));
		}

		let uniform_buffer = CpuAccessibleBuffer::from_data(
			self.device.clone(),
			BufferUsage::uniform_buffer(),
			false,
			camera,
		)
		.unwrap();

		let pipeline = self.get_pipeline();
		let layout = pipeline.layout().descriptor_set_layouts().get(0).unwrap();
		let set = PersistentDescriptorSet::new(
			layout.clone(),
			[WriteDescriptorSet::buffer(0, uniform_buffer)],
		)
		.unwrap();

		self.previous_frame_end.as_mut().unwrap().cleanup_finished();
		if self.recreate_swapchain {
			self.create_swapchain();
			self.recreate_swapchain = false;
		}

		let (image_num, suboptimal, acquire_future) =
			match swapchain::acquire_next_image(self.swapchain.clone(), None) {
				Ok(r) => r,
				Err(AcquireError::OutOfDate) => {
					self.recreate_swapchain = true;
					return;
				}
				Err(e) => {
					panic!("Failed to acquire next image: {:?}", e)
				}
			};
		if suboptimal {
			self.recreate_swapchain = true;
		}
		let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into()];
		let mut builder = AutoCommandBufferBuilder::primary(
			self.device.clone(),
			self.queue.family(),
			CommandBufferUsage::OneTimeSubmit,
		)
		.unwrap();

		builder
			.begin_render_pass(
				self.framebuffers[image_num].clone(),
				SubpassContents::Inline,
				clear_values,
			)
			.unwrap()
			.set_viewport(0, [self.viewport.clone()])
			.bind_pipeline_graphics(pipeline.clone());

		if self.render_mode == VkRenderMode::Normal {
			builder.bind_descriptor_sets(
				PipelineBindPoint::Graphics,
				pipeline.layout().clone(),
				0,
				vec![set, self.texture_set.clone()],
			);
			for (id, vertex_buffer) in vertex_buffers.into_iter() {
				let push_constants =
					shader::fs::ty::PushConstants { layer: id };
				builder.push_constants(
					pipeline.layout().clone(),
					0,
					push_constants,
				);
				builder
					.bind_vertex_buffers(0, vertex_buffer.clone())
					.draw(vertex_buffer.len() as u32, 1, 0, 0)
					.unwrap();
			}
		} else {
			builder.bind_descriptor_sets(
				PipelineBindPoint::Graphics,
				pipeline.layout().clone(),
				0,
				set,
			);
			for (_, vertex_buffer) in vertex_buffers.into_iter() {
				builder
					.bind_vertex_buffers(0, vertex_buffer.clone())
					.draw(vertex_buffer.len() as u32, 1, 0, 0)
					.unwrap();
			}
		}

		builder.end_render_pass().unwrap();

		// Finish building the command buffer by calling `build`.
		let command_buffer = builder.build().unwrap();

		let future = self
			.previous_frame_end
			.take()
			.unwrap()
			.join(acquire_future)
			.then_execute(self.queue.clone(), command_buffer)
			.unwrap()
			.then_swapchain_present(
				self.queue.clone(),
				self.swapchain.clone(),
				image_num,
			)
			.then_signal_fence_and_flush();

		match future {
			Ok(future) => {
				self.previous_frame_end = Some(future.boxed());
			}
			Err(FlushError::OutOfDate) => {
				self.recreate_swapchain = true;
				self.previous_frame_end =
					Some(sync::now(self.device.clone()).boxed());
			}
			Err(e) => {
				println!("Failed to flush future: {:?}", e);
				self.previous_frame_end =
					Some(sync::now(self.device.clone()).boxed());
			}
		}
	}

	fn create_swapchain(&mut self) {
		eprintln!("Recreate swapchain");
		let dimensions: [u32; 2] = self.surface.window().inner_size().into();
		let (new_swapchain, new_images) =
			match self.swapchain.recreate().dimensions(dimensions).build() {
				Ok(r) => r,
				Err(SwapchainCreationError::UnsupportedDimensions) => {
					eprintln!("Error: unsupported dimensions");
					return;
				}
				Err(e) => {
					panic!("Failed to recreate swapchain: {:?}", e)
				}
			};
		self.swapchain = new_swapchain;

		// Because framebuffers contains an Arc on the old swapchain, we need to
		// recreate framebuffers as well.
		let mut viewport = self.viewport.clone();
		self.framebuffers = window_size_dependent_setup(
			self.render_pass.clone(),
			&new_images,
			&mut viewport,
		);
		self.viewport = viewport;
	}
}

fn window_size_dependent_setup(
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
