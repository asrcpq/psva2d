use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{
	AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
	SubpassContents,
};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::swapchain::{self, AcquireError, SwapchainCreationError};
use vulkano::sync::{self, FlushError, GpuFuture};
use winit::event_loop::EventLoopWindowTarget;

use crate::camera::Camera;
use crate::shader;
use crate::vertex::Vertex;
use crate::vkstatic::VkStatic;
use crate::vkwrapper::{
	window_size_dependent_setup, VkwPipeline, VkwTextureSet,
};
use material::face::TextureData;
use material::render_model::RenderModel;

type VertexBuffers = Vec<(i32, Arc<CpuAccessibleBuffer<[Vertex]>>)>;

#[derive(PartialEq)]
pub enum VkRenderMode {
	Normal,
	Wireframe,
}

pub struct VkRender {
	pub recreate_swapchain: bool,
	render_mode: VkRenderMode,
	viewport: Viewport,
	v: VkStatic,
}

impl VkRender {
	pub fn new(
		el: &EventLoopWindowTarget<protocol::pr_model::PrModel>,
		textures: Vec<TextureData>,
		window_size: [u32; 2],
	) -> Self {
		let mut viewport = Viewport {
			origin: [0.0, 0.0],
			dimensions: [0.0, 0.0],
			depth_range: 0.0..1.0,
		};
		let v = VkStatic::new(el, textures, window_size, &mut viewport);
		Self {
			recreate_swapchain: false,
			render_mode: VkRenderMode::Normal,
			viewport,
			v,
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
			self.v.pipeline.clone()
		} else {
			self.v.pipeline_wf.clone()
		}
	}

	fn generate_vertex_buffers(
		&self,
		render_model: &RenderModel,
	) -> VertexBuffers {
		let mut vertex_buffers = vec![];
		for (&id, face_group) in &render_model.face_groups {
			if id < 0 || id >= self.v.tex_coords.len() as i32 {
				continue;
			}
			let vertex_buffer = CpuAccessibleBuffer::from_iter(
				self.v.device.clone(),
				BufferUsage::all(),
				false,
				face_group
					.faces
					.iter()
					.map(|x| {
						(0..3).map(|i| Vertex {
							pos: *render_model.vs.get(&x.vid[i]).unwrap(),
							tex_coord: self.v.tex_coords[id as usize]
								[x.uvid[i]],
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

	fn build_command(
		&self,
		image_num: usize,
		pipeline: VkwPipeline,
		set: VkwTextureSet,
		vertex_buffers: VertexBuffers,
	) -> PrimaryAutoCommandBuffer {
		let mut builder = AutoCommandBufferBuilder::primary(
			self.v.device.clone(),
			self.v.queue.family(),
			CommandBufferUsage::OneTimeSubmit,
		)
		.unwrap();

		let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into()];
		builder
			.begin_render_pass(
				self.v.framebuffers[image_num].clone(),
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
				vec![set, self.v.texture_set.clone()],
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
		builder.build().unwrap()
	}

	pub fn render(&mut self, render_model: RenderModel, camera: Camera) {
		let vertex_buffers = self.generate_vertex_buffers(&render_model);
		let uniform_buffer = CpuAccessibleBuffer::from_data(
			self.v.device.clone(),
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

		self.v
			.previous_frame_end
			.as_mut()
			.unwrap()
			.cleanup_finished();
		if self.recreate_swapchain {
			self.create_swapchain();
			self.recreate_swapchain = false;
		}

		let (image_num, suboptimal, acquire_future) =
			match swapchain::acquire_next_image(self.v.swapchain.clone(), None)
			{
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

		let command_buffer =
			self.build_command(image_num, pipeline, set, vertex_buffers);

		let future = self
			.v
			.previous_frame_end
			.take()
			.unwrap()
			.join(acquire_future)
			.then_execute(self.v.queue.clone(), command_buffer)
			.unwrap()
			.then_swapchain_present(
				self.v.queue.clone(),
				self.v.swapchain.clone(),
				image_num,
			)
			.then_signal_fence_and_flush();

		match future {
			Ok(future) => {
				self.v.previous_frame_end = Some(future.boxed());
			}
			Err(FlushError::OutOfDate) => {
				self.recreate_swapchain = true;
				self.v.previous_frame_end =
					Some(sync::now(self.v.device.clone()).boxed());
			}
			Err(e) => {
				println!("Failed to flush future: {:?}", e);
				self.v.previous_frame_end =
					Some(sync::now(self.v.device.clone()).boxed());
			}
		}
	}

	fn create_swapchain(&mut self) {
		eprintln!("Recreate swapchain");
		let dimensions: [u32; 2] = self.v.surface.window().inner_size().into();
		let (new_swapchain, new_images) =
			match self.v.swapchain.recreate().dimensions(dimensions).build() {
				Ok(r) => r,
				Err(SwapchainCreationError::UnsupportedDimensions) => {
					eprintln!("Error: unsupported dimensions");
					return;
				}
				Err(e) => {
					panic!("Failed to recreate swapchain: {:?}", e)
				}
			};
		self.v.swapchain = new_swapchain;

		// Because framebuffers contains an Arc on the old swapchain, we need to
		// recreate framebuffers as well.
		let mut viewport = self.viewport.clone();
		self.v.framebuffers = window_size_dependent_setup(
			self.v.render_pass.clone(),
			&new_images,
			&mut viewport,
		);
		self.viewport = viewport;
	}
}
