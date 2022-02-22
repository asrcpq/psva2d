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
use crate::vertex::{Vertex, VertexWf};
use crate::vkstatic::VkStatic;
use crate::vkwrapper::{
	window_size_dependent_setup, VkwCommandBuffer, VkwPipeline, VkwTextureSet,
};
use material::face::TextureData;
use material::render_model::RenderModel;
use material::texture_indexer::TextureIndexerRef;
use protocol::pr_model::PrModel;

type VertexBuffer<V> = Arc<CpuAccessibleBuffer<[V]>>;
type VertexBuffers<V> = Vec<(i32, VertexBuffer<V>)>;

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
	indexer: TextureIndexerRef,
}

impl VkRender {
	pub fn new(
		el: &EventLoopWindowTarget<protocol::pr_model::PrModel>,
		window_size: [u32; 2],
		textures: Vec<TextureData>,
		indexer: TextureIndexerRef,
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
			indexer,
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
	) -> VertexBuffers<Vertex> {
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
					.collect::<Vec<_>>()
					.into_iter(),
			)
			.unwrap();
			vertex_buffers.push((id, vertex_buffer));
		}
		vertex_buffers
	}

	fn generate_vertex_wf_buffer(
		&self,
		pr_model: &PrModel,
	) -> VertexBuffer<VertexWf> {
		let mut vertices = vec![];
		let color1 = [0.0, 0.0, 1.0, 0.5];
		let color2 = [0.0, 1.0, 0.0, 0.0];
		for constraint in &pr_model.constraints {
			let mut positions = vec![];
			for &pid in constraint.particles.iter() {
				if let Some(p) = pr_model.particles.get(&pid) {
					positions.push(p.pos);
				} else {
					eprintln!("ERROR: vkrender found that pr model is broken");
				}
			}
			if positions.len() == 2 {
				if constraint.id != -1 && constraint.id != -2 {
					eprintln!(
						"WARNING: unknown constraint id {}",
						constraint.id
					);
				}
				vertices.extend(vec![0, 1].into_iter().map(|i| VertexWf {
					color: if constraint.id == -1 { color1 } else { color2 },
					pos: positions[i],
				}));
			} else if positions.len() != 3 {
				eprintln!(
					"ERROR: found constraint contains {} particles",
					positions.len()
				);
			}
		}
		CpuAccessibleBuffer::from_iter(
			self.v.device.clone(),
			BufferUsage::all(),
			false,
			vertices.into_iter(),
		)
		.unwrap()
	}

	fn build_command(
		&self,
		image_num: usize,
		pipeline: VkwPipeline,
		set: VkwTextureSet,
		pr_model: &PrModel,
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
			let render_model = self.indexer.borrow().compile_model(pr_model);
			let vertex_buffers = self.generate_vertex_buffers(&render_model);
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
			let vertex_buffer = self.generate_vertex_wf_buffer(pr_model);
			builder
				.bind_descriptor_sets(
					PipelineBindPoint::Graphics,
					pipeline.layout().clone(),
					0,
					set,
				)
				.bind_vertex_buffers(0, vertex_buffer.clone())
				.draw(vertex_buffer.len() as u32, 1, 0, 0)
				.unwrap();
		}

		builder.end_render_pass().unwrap();
		builder.build().unwrap()
	}

	fn render_world(&self, image_num: usize, pr_model: &PrModel, camera: Camera) -> VkwCommandBuffer {
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

		let command_buffer =
			self.build_command(image_num, pipeline, set, pr_model);

		Box::new(command_buffer)
	}

	pub fn render(&mut self, pr_model: &PrModel, camera: Camera) {
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

		let command_buffer = self.render_world(image_num, pr_model, camera);

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
