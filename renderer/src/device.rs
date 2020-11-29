use ash::vk;
use ash::version::{InstanceV1_0, DeviceV1_0};

use crate::InstanceMTXG;


#[derive(Clone)]
pub struct DeviceMTXG {
    pub(crate) instance: InstanceMTXG,
    pub(crate) debug_mode: bool,
    pub(crate) physical: vk::PhysicalDevice,  // Physical Device
    pub(crate) device: ash::Device,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    pub(crate) graphics_queue_fam_id: u32,
    pub(crate) present_queue_fam_id: u32,
    pub(crate) transfer_queue: vk::Queue,
    pub(crate) transfer_queue_fam_id: u32,
    pub feat_anisotropy: bool,
}

impl DeviceMTXG {
    pub fn wait_idle(&self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
        }
    }

    pub fn find_format(&self, candidates: Vec<vk::Format>, tiling: vk::ImageTiling, features: vk::FormatFeatureFlags) -> vk::Format {
        for format in &candidates {
            let props = unsafe { self.instance.instance.get_physical_device_format_properties(self.physical, *format) };
            if self.debug_mode {
                println!("Supported selected format properties of <Format: {:?}>", format);
                println!("\tLinear Tiling Features: {:?}", props.linear_tiling_features);
                println!("\tOptimal Tiling Features: {:?}", props.optimal_tiling_features);
                println!("\tBuffer Features: {:?}", props.buffer_features);
            }

            if tiling == vk::ImageTiling::LINEAR && (props.linear_tiling_features & features == features) {
                return *format;
            } else if tiling == vk::ImageTiling::OPTIMAL && (props.optimal_tiling_features & features == features) {
                return *format;
            }
        }
        panic!(format!("The current physical device does not support any formats listed <{:?}>", candidates));
    }

    pub fn find_depth_format(&self) -> vk::Format {
        self.find_format(
            vec![vk::Format::D32_SFLOAT, vk::Format::D32_SFLOAT_S8_UINT, vk::Format::D24_UNORM_S8_UINT],
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
        )
    }

    pub fn has_stencil_format(format: vk::Format) -> bool {
        format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT
    }
}
