use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::image::ImageAccess;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::swapchain::{self, AcquireError, SwapchainCreationError};
use vulkano::sync::{self, FlushError, GpuFuture};
use winit::event_loop::EventLoopWindowTarget;

use super::vkstatic::vks::Vks;
use super::vkstatic::vks_overlay::VksOverlay;
use super::vkstatic::vks_world::VksWorld;
use super::vkwrapper::*;
use crate::camera::Camera;
use crate::render_mode::RenderMode;
use material::face::TextureData;
use material::texture_indexer::TextureIndexerRef;
use protocol::pr_model::PrModel;

pub struct VkRender {
	recreate_swapchain: bool,
	viewport: Viewport,
	previous_frame_end: Option<VkwFuture>,

	vks: Vks,
	r_world: VksWorld,
	r_overlay: VksOverlay,
}

impl VkRender {
	pub fn set_text(&mut self, name: &str, text: Vec<u8>, bad: bool) {
		self.r_overlay.simple_set_text(name, text, bad);
	}

	pub fn set_primitives(&mut self, primitives: Vec<crate::vertex::VertexWf>) {
		self.r_world.primitives = primitives;
	}

	pub fn flush_swapchain(&mut self) {
		self.recreate_swapchain = true;
	}

	pub fn set_render_mode(&mut self, render_mode: RenderMode) {
		self.r_world.set_render_mode(render_mode);
	}

	pub fn new<E>(
		el: &EventLoopWindowTarget<E>,
		window_size: [u32; 2],
		textures: Vec<TextureData>,
		indexer: TextureIndexerRef,
	) -> Self {
		let viewport = Viewport {
			origin: [0.0, 0.0],
			dimensions: [window_size[0] as f32, window_size[1] as f32],
			depth_range: 0.0..1.0,
		};
		let vks = Vks::new(el, window_size);
		let r_world = VksWorld::new(vks.clone(), textures, indexer);
		let r_overlay = VksOverlay::new(vks.clone());
		let previous_frame_end = Some(sync::now(vks.device.clone()).boxed());
		Self {
			recreate_swapchain: false,
			viewport,
			vks,
			r_world,
			r_overlay,
			previous_frame_end,
		}
	}

	pub fn render(&mut self, pr_model: &PrModel, camera: Camera) {
		self.previous_frame_end.as_mut().unwrap().cleanup_finished();
		if self.recreate_swapchain {
			self.create_swapchain();
			self.recreate_swapchain = false;
		}

		let (image_num, suboptimal, acquire_future) =
			match swapchain::acquire_next_image(
				self.vks.swapchain.clone(),
				None,
			) {
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

		let mut builder = AutoCommandBufferBuilder::primary(
			self.vks.device.clone(),
			self.vks.queue.family(),
			CommandBufferUsage::OneTimeSubmit,
		)
		.unwrap();
		self.r_world.build_command(
			&mut builder,
			image_num,
			pr_model,
			camera,
			self.viewport.clone(),
		);
		self.r_overlay.build_command(
			&mut builder,
			image_num,
			self.viewport.clone(),
		);
		let command_buffer = Box::new(builder.build().unwrap());

		let future = self
			.previous_frame_end
			.take()
			.unwrap()
			.join(acquire_future)
			.then_execute(self.vks.queue.clone(), command_buffer)
			.unwrap()
			.then_swapchain_present(
				self.vks.queue.clone(),
				self.vks.swapchain.clone(),
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
					Some(sync::now(self.vks.device.clone()).boxed());
			}
			Err(e) => {
				println!("Failed to flush future: {:?}", e);
				self.previous_frame_end =
					Some(sync::now(self.vks.device.clone()).boxed());
			}
		}
	}

	fn create_swapchain(&mut self) {
		eprintln!("Recreate swapchain");
		let dimensions: [u32; 2] =
			self.vks.surface.window().inner_size().into();
		self.r_overlay
			.set_text_scaler(self.vks.surface.window().scale_factor() as f32);
		let (new_swapchain, new_images) = match self
			.vks
			.swapchain
			.recreate()
			.dimensions(dimensions)
			.build()
		{
			Ok(r) => r,
			Err(SwapchainCreationError::UnsupportedDimensions) => {
				eprintln!("Error: unsupported dimensions");
				return;
			}
			Err(e) => {
				panic!("Failed to recreate swapchain: {:?}", e)
			}
		};
		self.vks.swapchain = new_swapchain;

		let dimensions = new_images[0].dimensions().width_height();
		self.viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];
		self.r_world.update_framebuffers(&new_images);
		self.r_overlay.update_framebuffers(&new_images);
		self.vks.images = new_images;
	}
}
