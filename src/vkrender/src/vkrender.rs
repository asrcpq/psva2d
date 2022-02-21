use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents,
};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{self, AcquireError, SwapchainCreationError};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::camera::Camera;
use crate::shader;
use crate::vertex::Vertex;
use crate::vkwrapper::*;
use material::face::TextureData;
use material::render_model::RenderModel;

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

	device: VkwDevice,
	queue: VkwQueue,
	surface: VkwSurface<Window>,
	swapchain: VkwSwapchain<Window>,
	framebuffers: Vec<Arc<Framebuffer>>,
	viewport: Viewport,
	//vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
	previous_frame_end: Option<Box<dyn GpuFuture>>,
	pipeline: VkwPipeline,
	pipeline_wf: VkwPipeline,
	render_pass: Arc<RenderPass>,
	texture_set: VkwTextureSet,
	tex_coords: VkwTexCoords,

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

		let (physical_device, device, queue) =
			get_device_and_queue(&instance, surface.clone());

		let (swapchain, images) = get_swapchain_and_images(
			physical_device,
			device.clone(),
			queue.clone(),
			surface.clone(),
		);

		let render_pass = get_render_pass(device.clone(), swapchain.clone());
		let pipelines = get_pipelines(render_pass.clone(), device.clone());
		let pipeline = pipelines[0].clone();
		let pipeline_wf = pipelines[1].clone();

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
		let (texture_set, tex_coords, previous_frame_end) = get_textures(
			textures,
			device.clone(),
			queue.clone(),
			pipeline.clone(),
		);

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

	fn generate_vertex_buffers(
		&self,
		render_model: &RenderModel,
	) -> Vec<(i32, Arc<CpuAccessibleBuffer<[Vertex]>>)> {
		let mut vertex_buffers = vec![];
		for (&id, face_group) in &render_model.face_groups {
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
		vertex_buffers
	}

	pub fn render(&mut self, render_model: RenderModel, camera: Camera) {
		let vertex_buffers = self.generate_vertex_buffers(&render_model);
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
