use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::SubpassContents;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{Pipeline, PipelineBindPoint};

use super::vks::Vks;
use crate::camera::Camera;
use crate::shader;
use crate::vertex::{Vertex, VertexWf};
use crate::vk::vkwrapper::*;
use material::face::TextureData;
use material::render_model::RenderModel;
use material::texture_indexer::TextureIndexerRef;
use protocol::pr_model::PrModel;

type VertexBuffer<V> = Arc<CpuAccessibleBuffer<[V]>>;
type VertexBuffers<V> = Vec<(i32, VertexBuffer<V>)>;
type CameraBuffer = Arc<CpuAccessibleBuffer<Camera>>;

#[derive(PartialEq)]
enum RenderMode {
	Normal,
	Wireframe,
}

pub struct VksWorld {
	vks: Vks,
	framebuffers: Vec<VkwFramebuffer>,
	pipeline: VkwPipeline,
	pipeline_wf: VkwPipeline,
	render_pass: VkwRenderPass,
	texture_set: VkwTextureSet,

	render_mode: RenderMode,
	indexer: TextureIndexerRef,
	tex_coords: VkwTexCoords,

	pub primitives: Vec<VertexWf>,
}

impl VksWorld {
	pub fn new(
		vks: Vks,
		textures: Vec<TextureData>,
		indexer: TextureIndexerRef,
	) -> Self {
		let render_pass =
			get_render_pass(vks.device.clone(), vks.swapchain.clone());
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
			vks,
			framebuffers,
			pipeline,
			pipeline_wf,
			render_pass,
			texture_set,

			render_mode: RenderMode::Normal,
			indexer,
			tex_coords,

			primitives: Vec::new(),
		}
	}

	pub fn toggle_render_mode(&mut self) {
		if self.render_mode == RenderMode::Normal {
			self.render_mode = RenderMode::Wireframe;
		} else {
			self.render_mode = RenderMode::Normal;
		}
	}

	fn generate_vertex_buffers(
		&self,
		render_model: &RenderModel,
	) -> VertexBuffers<Vertex> {
		let mut vertex_buffers = vec![];
		for (&id, face_group) in &render_model.face_groups {
			if id < 0 || id >= self.tex_coords.len() as i32 {
				continue;
			}
			let vertex_buffer = CpuAccessibleBuffer::from_iter(
				self.vks.device.clone(),
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
		let mut vertices = self.primitives.clone();
		let color1 = [0.0, 0.0, 1.0, 0.5];
		let color2 = [0.0, 1.0, 0.0, 1.0];
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
				vertices.extend(vec![0, 1].into_iter().map(|i| VertexWf {
					color: if constraint.id == -1 { color1 } else { color2 },
					pos: positions[i],
				}));
			}
		}
		CpuAccessibleBuffer::from_iter(
			self.vks.device.clone(),
			BufferUsage::all(),
			false,
			vertices.into_iter(),
		)
		.unwrap()
	}

	pub fn build_command_wireframe(
		&self,
		builder: &mut VkwCommandBuilder,
		pr_model: &PrModel,
		uniform_buffer: CameraBuffer,
	) {
		let layout = self
			.pipeline_wf
			.layout()
			.descriptor_set_layouts()
			.get(0)
			.unwrap();
		let set = PersistentDescriptorSet::new(
			layout.clone(),
			[WriteDescriptorSet::buffer(0, uniform_buffer)],
		)
		.unwrap();
		let vertex_buffer = self.generate_vertex_wf_buffer(pr_model);
		let buflen = vertex_buffer.len();
		builder
			.bind_pipeline_graphics(self.pipeline_wf.clone())
			.bind_descriptor_sets(
				PipelineBindPoint::Graphics,
				self.pipeline_wf.layout().clone(),
				0,
				set,
			)
			.bind_vertex_buffers(0, vertex_buffer)
			.draw(buflen as u32, 1, 0, 0)
			.unwrap();
	}

	pub fn build_command_world(
		&self,
		builder: &mut VkwCommandBuilder,
		pr_model: &PrModel,
		uniform_buffer: CameraBuffer,
	) {
		let layout = self
			.pipeline
			.layout()
			.descriptor_set_layouts()
			.get(0)
			.unwrap();
		let set = PersistentDescriptorSet::new(
			layout.clone(),
			[WriteDescriptorSet::buffer(0, uniform_buffer)],
		)
		.unwrap();
		let render_model = self.indexer.borrow().compile_model(pr_model);
		let vertex_buffers = self.generate_vertex_buffers(&render_model);
		builder
			.bind_pipeline_graphics(self.pipeline.clone())
			.bind_descriptor_sets(
				PipelineBindPoint::Graphics,
				self.pipeline.layout().clone(),
				0,
				vec![set, self.texture_set.clone()],
			);
		for (id, vertex_buffer) in vertex_buffers.into_iter() {
			let push_constants = shader::fs::ty::PushConstants { layer: id };
			builder.push_constants(
				self.pipeline.layout().clone(),
				0,
				push_constants,
			);
			let buflen = vertex_buffer.len();
			builder
				.bind_vertex_buffers(0, vertex_buffer)
				.draw(buflen as u32, 1, 0, 0)
				.unwrap();
		}
	}

	pub fn build_command(
		&self,
		builder: &mut VkwCommandBuilder,
		image_num: usize,
		pr_model: &PrModel,
		camera: Camera,
		viewport: Viewport,
	) {
		let uniform_buffer = CpuAccessibleBuffer::from_data(
			self.vks.device.clone(),
			BufferUsage::uniform_buffer(),
			false,
			camera,
		)
		.unwrap();

		let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into()];
		builder
			.begin_render_pass(
				self.framebuffers[image_num].clone(),
				SubpassContents::Inline,
				clear_values,
			)
			.unwrap()
			.set_viewport(0, [viewport]);
		self.build_command_world(builder, pr_model, uniform_buffer.clone());
		if self.render_mode == RenderMode::Wireframe {
			self.build_command_wireframe(builder, pr_model, uniform_buffer);
		}
		builder.end_render_pass().unwrap();
	}

	pub fn update_framebuffers(&mut self, images: &VkwImages) {
		self.framebuffers =
			window_size_dependent_setup(self.render_pass.clone(), images);
	}
}
