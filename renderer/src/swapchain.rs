use ash::extensions::khr;
use ash::vk;
use ash::version::DeviceV1_0;

use crate::device::DeviceMTXG;
use crate::{InstanceMTXG, CleanupVkObj};
use crate::buffer::{create_image_view, ImageMemoryMTXG};
use ash::vk::ImageView;


#[derive(Clone)]
pub struct SwapchainMTXG {
    pub(crate) swapchain_handler: khr::Swapchain,
    pub(crate) swapchain: vk::SwapchainKHR,
    pub(crate) format: vk::SurfaceFormatKHR,
    pub(crate) present_mode: vk::PresentModeKHR,
    pub(crate) image_min_count: u32,
    pub(crate) current_extent: vk::Extent2D,  // aka swapchain/window Dimensions
    pub(crate) image_views: Vec<vk::ImageView>,
    pub(crate) depth_views: Option<(ImageMemoryMTXG, vk::ImageView)>,  // depth sampler
    pub(crate) enable_depth_sampling: bool,
}

impl SwapchainMTXG {
    pub fn new(instance: &InstanceMTXG, device: &DeviceMTXG, min_image_count: u32, depth_sampling: bool, old_swapchain: Option<vk::SwapchainKHR>) -> SwapchainMTXG {
        let surf_cap = unsafe { instance.surface_handler.get_physical_device_surface_capabilities(device.physical, instance.surface) }.expect("Failed to get surface capabilities");
        let formats = unsafe { instance.surface_handler.get_physical_device_surface_formats(device.physical, instance.surface) }.expect("Failed to get surface formats");
        let presentations = unsafe { instance.surface_handler.get_physical_device_surface_present_modes(device.physical, instance.surface) }.expect("Failed to get surface present modes");

        if instance.debug_mode {
            println!("Surface capabilities: {:?}", surf_cap);
            println!("Available formats: {:?}", formats);
            println!("Available presentation modes: {:?}", presentations);
        }

        let spec_format = formats.into_iter()
            .find(|f| f == &vk::SurfaceFormatKHR { format: vk::Format::B8G8R8A8_SRGB, color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR })
            .unwrap();
        let present_mode = presentations.clone().into_iter()
            .find(|f| f == &vk::PresentModeKHR::MAILBOX)
            .unwrap_or(
                presentations.into_iter()
                    .find(|f| f == &vk::PresentModeKHR::FIFO)
                    .unwrap()
            );
        let swapchain_extent = surf_cap.current_extent;
        let img_count = if surf_cap.min_image_count == surf_cap.max_image_count {
            surf_cap.min_image_count
        } else {
            surf_cap.min_image_count+1
        };

        if instance.debug_mode {
            println!("Selected format of {:?}", spec_format);
            println!("Selected present mode of {:?}", present_mode);
            println!("Selected swap extent of {:?}", swapchain_extent);
            println!("Selected image count of {:?}", img_count);
        }

        let swapchain_cinfo = vk::SwapchainCreateInfoKHR {
            surface: instance.surface,
            min_image_count: if min_image_count > img_count { min_image_count } else { img_count },  // number of images in swapchain buffer
            image_format: spec_format.format,
            image_color_space: spec_format.color_space,
            image_extent: swapchain_extent,  // image resolution
            image_array_layers: 1,  // layers per image
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: vk::SharingMode::EXCLUSIVE,
            pre_transform: surf_cap.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,  // alpha against other windows in the OS
            present_mode: present_mode,
            clipped: vk::TRUE,
            old_swapchain: if let Some(s) = old_swapchain { s } else { vk::SwapchainKHR::null() },  // handling of invalidated old swapchain after window resizes, etc.
            ..Default::default()
        };

        let swapchain_handler = ash::extensions::khr::Swapchain::new(&instance.instance, &device.device);
        let swapchain = unsafe { swapchain_handler.create_swapchain(&swapchain_cinfo, None) }.expect("Failed to create swapchain");

        let swapchain_images = unsafe { swapchain_handler.get_swapchain_images(swapchain) }.expect("Failed to get swapchain images");

        let swapchain_image_views = swapchain_images.into_iter()
            .map(|img| {
                create_image_view(device, img, spec_format.format, vk::ImageAspectFlags::COLOR)
            })
            .collect::<Vec<_>>();

        let depth_image = if depth_sampling {
            let format = device.find_depth_format();
            let vk::Extent2D { width, height } = swapchain_extent;
            let depth_image = ImageMemoryMTXG::new(instance, device, width, height, format,
                                                   vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, vk::MemoryPropertyFlags::DEVICE_LOCAL);
            let depth_image_view = create_image_view(&device, depth_image.image, format, vk::ImageAspectFlags::DEPTH);
            Some((depth_image, depth_image_view))
        } else {
            None
        };

        SwapchainMTXG {
            swapchain_handler: swapchain_handler,
            swapchain: swapchain,
            format: spec_format,
            present_mode: present_mode,
            current_extent: swapchain_extent,
            image_min_count: min_image_count,
            image_views: swapchain_image_views,
            depth_views: depth_image,
            enable_depth_sampling: depth_sampling,
        }
    }

    pub fn current_extent(&self) -> vk::Extent2D {
        self.current_extent
    }

    pub fn image_format(&self) -> vk::SurfaceFormatKHR {
        self.format
    }

    pub fn image_count(&self) -> usize {
        self.image_views.len()
    }

    pub unsafe fn recreate_swapchain(self, instance: &InstanceMTXG, device: &DeviceMTXG,
                                     objects: Vec<&dyn CleanupVkObj>) -> Self {
        let swap = Self::new(instance, device, 2, self.enable_depth_sampling, Some(self.swapchain));
        for obj in objects {
            obj.cleanup_recreation(device);
        }
        swap
    }

    pub fn get_framebuffers(&self,
                            device: &DeviceMTXG,
                            renderpass: vk::RenderPass) -> Vec<vk::Framebuffer> {
        let mut swapchain_framebuffers = Vec::new();

        for i in 0..self.image_views.len() {
            let attachment_views = if self.enable_depth_sampling {
                vec![self.image_views[i], self.depth_views.unwrap().1]
            } else {
                vec![self.image_views[i]]
            };

            let framebuffer_cinfo = vk::FramebufferCreateInfo {
                render_pass: renderpass,
                attachment_count: attachment_views.len() as u32,
                p_attachments: attachment_views.as_slice().as_ptr(),
                width: self.current_extent.width,
                height: self.current_extent.height,
                layers: 1,
                ..Default::default()
            };
            swapchain_framebuffers.push(unsafe { device.device.create_framebuffer(&framebuffer_cinfo, None) }.expect("Failed to create a framebuffer"))
        }
        swapchain_framebuffers
    }
}

impl CleanupVkObj for SwapchainMTXG {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        for imgv in self.image_views.clone() {
            device.device.destroy_image_view(imgv, None);
        }
        if let Some(depth_view) = self.depth_views {
            device.device.destroy_image_view(depth_view.1, None);
            depth_view.0.cleanup(device);
        }
        self.swapchain_handler.destroy_swapchain(self.swapchain, None);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        self.cleanup(device);
    }
}

impl CleanupVkObj for Vec<vk::Framebuffer> {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        for framebuffer in self {
            device.device.destroy_framebuffer(*framebuffer, None);
        }
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        for framebuffer in self {
            device.device.destroy_framebuffer(*framebuffer, None);
        }
    }
}

