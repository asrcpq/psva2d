use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::SubpassContents;
use vulkano::format::ClearValue;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{Pipeline, PipelineBindPoint};

use super::vks::Vks;
use crate::vertex::VertexText;
use crate::vk::vkwrapper::*;

pub struct VksOverlay {
	vks: Vks,
	text_scaler: f32,
	text_color: [f32; 4],
	text: Vec<u8>,

	framebuffers: Vec<VkwFramebuffer>,
	pipeline_text: VkwPipeline,
	texture_set_text: VkwTextureSet,
	render_pass: VkwRenderPass,
}

impl VksOverlay {
	pub fn set_text(&mut self, text: Vec<u8>, bad: bool) {
		self.text = text;
		if bad {
			self.text_color = [1.0, 0.0, 0.0, 1.0];
		} else {
			self.text_color = [0.7, 0.8, 0.7, 1.0];
		}
	}

	pub fn set_text_scaler(&mut self, k: f32) {
		self.text_scaler = k;
	}

	pub fn new(vks: Vks) -> Self {
		let render_pass =
			get_render_pass_overlay(vks.device.clone(), vks.swapchain.clone());
		let pipeline_text =
			get_pipeline_text(render_pass.clone(), vks.device.clone());
		let framebuffers =
			window_size_dependent_setup(render_pass.clone(), &vks.images);
		let texture_set_text = get_text_texture(
			vks.device.clone(),
			vks.queue.clone(),
			pipeline_text.clone(),
		);
		VksOverlay {
			vks,
			framebuffers,
			pipeline_text,
			render_pass,
			texture_set_text,

			text_scaler: 1.0,
			text: b"hello, world".to_vec(),
			text_color: [1.0, 0.0, 0.0, 1.0],
		}
	}

	pub fn build_command(
		&self,
		builder: &mut VkwCommandBuilder,
		image_num: usize,
		viewport: Viewport,
	) {
		let mut coord_list = vec![];
		let mut pos_list = vec![];
		let w = 16;
		let h = 32;
		let size_x = 1024 / w;
		// let size_y = 1024 / h;
		for (idx, &ch) in self.text.iter().enumerate() {
			let idx = idx as u32;
			let ux = ch as u32 % size_x;
			let uy = ch as u32 / size_x;
			let upos_list =
				vec![[0, 0], [0, 1], [1, 1], [0, 0], [1, 0], [1, 1]];
			coord_list.extend(upos_list.iter().map(|upos| {
				[
					((ux + upos[0]) * w) as f32 / 1024f32,
					((uy + upos[1]) * h) as f32 / 1024f32,
				]
			}));
			pos_list.extend(upos_list.iter().map(|upos| {
				[
					-1.0 + ((idx + upos[0]) * w) as f32
						/ viewport.dimensions[0] * self.text_scaler,
					-1.0 + (upos[1] * h) as f32 / viewport.dimensions[1]
						* self.text_scaler,
				]
			}));
		}
		let vertex_buffer = pos_list
			.into_iter()
			.zip(coord_list.into_iter())
			.map(|(p, c)| VertexText {
				color: self.text_color,
				pos: p,
				tex_coord: c,
			});
		let vertex_buffer = CpuAccessibleBuffer::from_iter(
			self.vks.device.clone(),
			BufferUsage::all(),
			false,
			vertex_buffer,
		)
		.unwrap();

		builder
			.begin_render_pass(
				self.framebuffers[image_num].clone(),
				SubpassContents::Inline,
				vec![ClearValue::None],
			)
			.unwrap()
			.set_viewport(0, [viewport])
			.bind_pipeline_graphics(self.pipeline_text.clone());

		builder.bind_descriptor_sets(
			PipelineBindPoint::Graphics,
			self.pipeline_text.layout().clone(),
			0,
			self.texture_set_text.clone(),
		);
		let buflen = vertex_buffer.len();
		builder
			.bind_vertex_buffers(0, vertex_buffer)
			.draw(buflen as u32, 1, 0, 0)
			.unwrap();

		builder.end_render_pass().unwrap();
	}

	pub fn update_framebuffers(&mut self, images: &VkwImages) {
		self.framebuffers =
			window_size_dependent_setup(self.render_pass.clone(), images);
	}
}
