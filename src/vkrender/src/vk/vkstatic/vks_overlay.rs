use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::SubpassContents;
use vulkano::format::ClearValue;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{Pipeline, PipelineBindPoint};

use super::vks::Vks;
use crate::label_stack::line::Line;
use crate::label_stack::LabelStack;
use crate::vk::vkwrapper::*;

pub struct VksOverlay {
	vks: Vks,
	labels: LabelStack,

	framebuffers: Vec<VkwFramebuffer>,
	pipeline_text: VkwPipeline,
	texture_set_text: VkwTextureSet,
	render_pass: VkwRenderPass,
}

impl VksOverlay {
	pub fn simple_set_text(&mut self, name: &str, text: Vec<u8>, bad: bool) {
		let text_color = if bad {
			[1.0, 0.0, 0.0, 1.0]
		} else {
			[0.7, 0.8, 0.7, 1.0]
		};
		self.labels
			.add_text(name, Line::new_colored(text, text_color));
	}

	pub fn set_text_scaler(&mut self, k: f32) {
		self.labels.set_scaler(k);
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
			labels: LabelStack::new([16, 32]),
		}
	}

	pub fn build_command(
		&self,
		builder: &mut VkwCommandBuilder,
		image_num: usize,
		viewport: Viewport,
	) {
		let vertex_buffer = self.labels.to_vertices(&viewport);
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
