use ash::vk;
use ash::version::DeviceV1_0;

use std::ptr;

use crate::device::DeviceMTXG;
use crate::buffer::UniformBufferMTXG;
use crate::CleanupVkObj;


#[derive(Copy, Clone)]
pub struct SetBindingMTXG {
    ind: u32,
    typ: vk::DescriptorType,
    lyt: vk::DescriptorSetLayoutBinding,
}

impl SetBindingMTXG {
    pub fn new(index: u32, typ: vk::DescriptorType, stage: vk::ShaderStageFlags) -> Self {
        // stage parameter used to know which stage this descriptor binding will be used

        let set_binding = vk::DescriptorSetLayoutBinding {
            binding: index,
            descriptor_type: typ,
            descriptor_count: 1,
            stage_flags: stage,
            p_immutable_samplers: ptr::null(),
        };
        Self {
            ind: index,
            typ: typ,
            lyt: set_binding,
        }
    }
}

pub struct DescriptorSetsMTXG {
    pub(crate) device: DeviceMTXG,
    pub(crate) set_layout: vk::DescriptorSetLayout,
    pub(crate) pool: vk::DescriptorPool,
    pub(crate) sets: Vec<vk::DescriptorSet>,
}

impl DescriptorSetsMTXG {
    pub fn new(device: &DeviceMTXG, bindings: &[SetBindingMTXG], copies: u32) -> Self {
        let layout_cinfo = vk::DescriptorSetLayoutCreateInfo {
            binding_count: bindings.len() as u32,
            p_bindings: bindings.clone().into_iter().map(|b| b.lyt).collect::<Vec<_>>().as_slice().as_ptr(),
            ..Default::default()
        };

        // to define a layout, a skeleton, for the descriptor sets
        let dscrp_set_lyt = unsafe { device.device.create_descriptor_set_layout(&layout_cinfo, None) }.expect("Failed to create descriptor set layout");

        let mut dscrp_set_lyt_copies = Vec::new();
        for _ in 0..copies {
            // ash Vulkan API lib clones only the pointers, so they will all be freed with single command
            dscrp_set_lyt_copies.push(dscrp_set_lyt.clone());
        }

        // --- descriptor pool allocation ---

        let mut dscrp_pool_sizes = Vec::with_capacity(bindings.len());

        for i in 0..bindings.len() {
            dscrp_pool_sizes.push(
                vk::DescriptorPoolSize {
                    ty: bindings[i].typ,
                    descriptor_count: copies,
                }
            )
        }

        let dscrp_pool_cinfo = vk::DescriptorPoolCreateInfo {
            pool_size_count: bindings.len() as u32,
            p_pool_sizes: dscrp_pool_sizes.as_slice().as_ptr(),
            max_sets: copies,
            ..Default::default()
        };

        // to allocate heaps for descriptor sets to be in
        let dscrp_pool = unsafe { device.device.create_descriptor_pool(&dscrp_pool_cinfo, None) }.expect("Failed to create descriptor pool");

        // --- descriptor sets allocation ---
        let dscrp_set_ainfo = vk::DescriptorSetAllocateInfo {
            descriptor_pool: dscrp_pool,
            descriptor_set_count: copies,
            p_set_layouts: dscrp_set_lyt_copies.as_slice().as_ptr(),
            ..Default::default()
        };

        // to allocate each individual descriptor sets to be used in the shaders
        let dscrp_sets = unsafe { device.device.allocate_descriptor_sets(&dscrp_set_ainfo) }.expect("Failed to create descriptor set");

        Self {
            device: device.clone(),
            set_layout: dscrp_set_lyt,
            pool: dscrp_pool,
            sets: dscrp_sets,
        }
    }

    // TODO-CHECKED: somehow add an option for array sets like SamplerArray2D

    // binds uniform buffers (not uniform device memory) to the descriptor sets to be "connected" with the shaders from CPU
    pub fn bind_uniform(&self, binding: u32, buffers: &[UniformBufferMTXG], distribute: bool) {
        // distribution means to distribute each element of the buffer slice to each of the sets created

        if distribute {
            assert_eq!(self.sets.len(), buffers.len());

            for i in 0..self.sets.len() {
                let buffer_info = vk::DescriptorBufferInfo {
                    buffer: buffers[i].buffer,
                    offset: 0,
                    range: buffers[i].size,
                };

                let dscrp_write = vk::WriteDescriptorSet {
                    dst_set: self.sets[i],
                    dst_binding: binding,
                    dst_array_element: 0,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    p_buffer_info: &buffer_info,
                    ..Default::default()
                };

                unsafe { self.device.device.update_descriptor_sets(&[dscrp_write], &[]) };
            }
        } else {
            assert_eq!(1, buffers.len());

            for i in 0..self.sets.len() {
                let buffer_info = vk::DescriptorBufferInfo {
                    buffer: buffers[0].buffer,
                    offset: 0,
                    range: buffers[0].size,
                };

                let dscrp_write = vk::WriteDescriptorSet {
                    dst_set: self.sets[i],
                    dst_binding: binding,
                    dst_array_element: 0,
                    descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    p_buffer_info: &buffer_info,
                    ..Default::default()
                };

                unsafe { self.device.device.update_descriptor_sets(&[dscrp_write], &[]) };
            }
        }
    }

    // binds image/texture buffers
    pub fn bind_sampler(&self, binding: u32, image_views: &[vk::ImageView], sampler: vk::Sampler, distribute: bool) {
        // distribution means to distribute each element of the image slice to each of the sets created

        if distribute {
            assert_eq!(self.sets.len(), image_views.len());

            for i in 0..self.sets.len() {
                let image_info = vk::DescriptorImageInfo {
                    sampler: sampler,
                    image_view: image_views[i],
                    ..Default::default()
                };

                let dscrp_write = vk::WriteDescriptorSet {
                    dst_set: self.sets[i],
                    dst_binding: binding,
                    dst_array_element: 0,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    p_image_info: &image_info,
                    ..Default::default()
                };

                unsafe { self.device.device.update_descriptor_sets(&[dscrp_write], &[]) };
            }
        } else {
            assert_eq!(1, image_views.len());

            for i in 0..self.sets.len() {
                let image_info = vk::DescriptorImageInfo {
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    sampler: sampler,
                    image_view: image_views[0],
                };

                let dscrp_write = vk::WriteDescriptorSet {
                    dst_set: self.sets[i],
                    dst_binding: binding,
                    dst_array_element: 0,
                    descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    p_image_info: &image_info,
                    ..Default::default()
                };

                unsafe { self.device.device.update_descriptor_sets(&[dscrp_write], &[]) };
            }
        }
    }
}

impl CleanupVkObj for DescriptorSetsMTXG {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.destroy_descriptor_pool(self.pool, None);
        device.device.destroy_descriptor_set_layout(self.set_layout, None);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {

    }
}

