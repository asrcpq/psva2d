use vulkano::instance::Instance;
use vulkano::sync::{self, GpuFuture};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::dpi::{LogicalSize, Size};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};

use crate::vk::vkwrapper::*;

pub struct Vks {
	pub device: VkwDevice,
	pub queue: VkwQueue,
	pub surface: VkwSurface<Window>,
	pub swapchain: VkwSwapchain<Window>,
	pub images: VkwImages<Window>,
	pub previous_frame_end: Option<VkwFuture>,
}

fn winit_size(size: [u32; 2]) -> Size {
	Size::new(LogicalSize::new(size[0], size[1]))
}

impl Vks {
	pub fn new<E>(
		el: &EventLoopWindowTarget<E>,
		window_size: [u32; 2],
	) -> Self {
		let required_extensions = vulkano_win::required_extensions();
		let instance =
			Instance::new(None, Version::V1_1, &required_extensions, None)
				.unwrap();
		let surface = WindowBuilder::new()
			.with_inner_size(winit_size(window_size))
			//.with_resizable(false)
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
		let previous_frame_end = Some(sync::now(device.clone()).boxed());
		Self {
			device,
			queue,
			surface,
			swapchain,
			images,
			previous_frame_end,
		}
	}
}
