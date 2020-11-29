use ash::vk;
use ash::version::{DeviceV1_0, InstanceV1_0};

use std::mem;

use crate::device::DeviceMTXG;
use crate::{InstanceMTXG, CleanupVkObj};


pub fn create_image_view(device: &DeviceMTXG, image: vk::Image, format: vk::Format, aspects: vk::ImageAspectFlags) -> vk::ImageView {
    let img_view_cinfo = vk::ImageViewCreateInfo {
        image: image,
        view_type: vk::ImageViewType::TYPE_2D,
        format: format,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: aspects,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1
        },
        ..Default::default()
    };

    unsafe { device.device.create_image_view(&img_view_cinfo, None) }.unwrap()
}

// mipmap-less sampler creation
pub fn create_sampler(device: &DeviceMTXG,
                      mag_filter: vk::Filter,
                      min_filter: vk::Filter,
                      address_mode_u: vk::SamplerAddressMode,
                      address_mode_v: vk::SamplerAddressMode,
                      anisotropy_enable: bool,
                      max_anisotropy: f32) -> vk::Sampler {
    let texture_smplr_cinfo = vk::SamplerCreateInfo {
        // filters
        mag_filter: mag_filter,
        min_filter: min_filter,
        // transformation
        address_mode_u: address_mode_u,
        address_mode_v: address_mode_v,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        // anisotropy
        anisotropy_enable: if device.feat_anisotropy && anisotropy_enable {vk::TRUE} else {vk::FALSE},
        max_anisotropy: if device.feat_anisotropy && anisotropy_enable {max_anisotropy} else {1.0},
        // misc.
        border_color: vk::BorderColor::INT_OPAQUE_BLACK,
        unnormalized_coordinates: vk::FALSE,
        compare_enable: vk::FALSE,
        compare_op: vk::CompareOp::ALWAYS,
        // mipmapping
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        mip_lod_bias: 0.0,
        min_lod: 0.0,
        max_lod: 0.0,
        ..Default::default()
    };

    unsafe { device.device.create_sampler(&texture_smplr_cinfo, None) }.expect("Failed to create a no-mipmap texture sampler")
}

#[derive(Clone)]
pub struct BufferMemoryMTXG {
    device: DeviceMTXG,
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: u64,  // size of the buffer
    pub len: u32,  // length of the actual data per element size of the data collection
    pub host_visible: bool,  // checks if this buffer should be able to be mapped and filled with data
}

impl BufferMemoryMTXG {
    pub fn new<D: Copy>(instance: &InstanceMTXG, device: &DeviceMTXG, len: usize, usage: vk::BufferUsageFlags, prop: vk::MemoryPropertyFlags) -> Self {
        // len parameter for finding pre-buffer-alloc size from the length of each type D
        let buf_size = (mem::size_of::<D>() * len) as u64;
        let (buf, mem, alloc_size) = create_buf(instance, device, buf_size, usage, prop);

        Self {
            device: device.clone(),
            buffer: buf,
            memory: mem,
            size: buf_size,
            len: len as u32,
            host_visible: prop & vk::MemoryPropertyFlags::HOST_VISIBLE == vk::MemoryPropertyFlags::HOST_VISIBLE,
        }
    }

    // an extension function of the instantiation function for buffers with HOST_VISIBLE bit set true
    pub fn fill<D: Copy>(self, data: Vec<D>) -> Self {
        if !self.host_visible {
            panic!("Cannot map and fill a buffer without enabling HOST_VISIBLE flag first in the buffer memory properties");
        }
        let buf_ptr = unsafe { self.device.device.map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty()) }.unwrap();
        let mut buf_align = unsafe {
            ash::util::Align::new(buf_ptr, mem::align_of::<D>() as u64, self.size)
        };
        buf_align.copy_from_slice(data.as_slice());
        unsafe { self.device.device.unmap_memory(self.memory) };

        Self {
            device: self.device,
            buffer: self.buffer,
            memory: self.memory,
            size: self.size,
            len: self.len,
            host_visible: self.host_visible,
        }
    }
}


#[derive(Copy, Clone)]
pub struct ImageMemoryMTXG {
    pub image: vk::Image,  // a image representation in memory
    pub memory: vk::DeviceMemory,  // the memory will be located on GPU locally
    pub size: u64,  // actual buffer size
    pub width: u32,  // texture width
    pub height: u32,  // texture height
    pub format: vk::Format,
}

impl ImageMemoryMTXG {
    pub fn new(instance: &InstanceMTXG, device: &DeviceMTXG,
               txtr_width: u32, txtr_height: u32, txtr_format: vk::Format,
               usage: vk::ImageUsageFlags, prop: vk::MemoryPropertyFlags) -> Self {
        let (image, image_mem, alloc_size) = create_img(instance, device, txtr_width, txtr_height, txtr_format,
                                                        vk::ImageTiling::OPTIMAL, usage, prop);
        Self {
            image: image,
            memory: image_mem,
            size: alloc_size,
            width: txtr_width,
            height: txtr_height,
            format: txtr_format,
        }
    }
}


#[derive(Clone)]
pub struct UniformBufferMTXG {
    pub(crate) buffer: vk::Buffer,
    pub(crate) memory: vk::DeviceMemory,
    pub(crate) size: u64,
}

impl UniformBufferMTXG {
    pub fn new<U>(instance: &InstanceMTXG, device: &DeviceMTXG) -> Self {
        let data_size = mem::size_of::<U>() as u64;
        let (buf, mem, alloc_size) = create_buf(instance, device, data_size,
                                    vk::BufferUsageFlags::UNIFORM_BUFFER, vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT);
        Self {
            buffer: buf,
            memory: mem,
            size: data_size,
        }
    }

    pub fn update<T: Copy>(&self, device: &DeviceMTXG, data: T) {
        let buf_ptr = unsafe { device.device.map_memory(self.memory, 0, self.size, vk::MemoryMapFlags::empty()) }.unwrap();
        let mut align = unsafe {
            ash::util::Align::new(buf_ptr, mem::align_of::<T>() as u64, self.size)
        };
        align.copy_from_slice(&[data]);
        unsafe { device.device.unmap_memory(self.memory) };
    }
}


fn create_img(instance: &InstanceMTXG,
              device: &DeviceMTXG,
              txtr_width: u32,
              txtr_height: u32,
              txtr_format: vk::Format,
              txtr_tiling: vk::ImageTiling,
              usage: vk::ImageUsageFlags,
              prop: vk::MemoryPropertyFlags) -> (vk::Image, vk::DeviceMemory, vk::DeviceSize) {
    let image_cinfo = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_2D,
        extent: vk::Extent3D {
            width: txtr_width,
            height: txtr_height,
            depth: 1,
        },
        mip_levels: 1,
        array_layers: 1,
        format: txtr_format,
        tiling: txtr_tiling,
        initial_layout: vk::ImageLayout::UNDEFINED,
        usage: usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };

    let txtr_image = unsafe { device.device.create_image(&image_cinfo, None) }.expect("Failed to create an image");

    let image_mem_req = unsafe { device.device.get_image_memory_requirements(txtr_image) };
    let image_mem_prop = unsafe { instance.instance.get_physical_device_memory_properties(device.physical) };
    if device.debug_mode {
        println!("TXTR IMG MEM REQ: {:?}", image_mem_req);
        println!("TXTR IMG PRP REQ: {:?}", image_mem_prop);
    }

    let mem_ainfo = vk::MemoryAllocateInfo {
        allocation_size: image_mem_req.size,
        memory_type_index: find_memtype_ind(&image_mem_req, &image_mem_prop, prop).expect("Failed to find a suitable memory buffer"),
        ..Default::default()
    };

    let image_mem = unsafe { device.device.allocate_memory(&mem_ainfo, None) }.expect("Failed tor create an image memory");

    unsafe { device.device.bind_image_memory(txtr_image, image_mem, 0) };

    // note: memory requested size does not necessarily means the actual size of the buffer
    (txtr_image, image_mem, image_mem_req.size)
}

fn create_buf(instance: &InstanceMTXG,
              device: &DeviceMTXG,
              size: vk::DeviceSize,
              usage: vk::BufferUsageFlags,
              prop: vk::MemoryPropertyFlags) -> (vk::Buffer, vk::DeviceMemory, vk::DeviceSize) {
    let buffer_cinfo = vk::BufferCreateInfo {
        size: size,
        usage: usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    let buffer = unsafe { device.device.create_buffer(&buffer_cinfo, None) }.unwrap();
    let mem_req = unsafe { device.device.get_buffer_memory_requirements(buffer) };
    let mem_prop = unsafe { instance.instance.get_physical_device_memory_properties(device.physical) };
    if device.debug_mode {
        println!("Mem requirements: {:?}", mem_req);
        println!("Phys mem requirements: {:?}", mem_prop);
    }

    let mem_ainfo = vk::MemoryAllocateInfo {
        allocation_size: mem_req.size,
        memory_type_index: find_memtype_ind(&mem_req, &mem_prop, prop).expect("Failed to find a suitable memory buffer"),
        ..Default::default()
    };

    let buffer_mem = unsafe { device.device.allocate_memory(&mem_ainfo, None) }.unwrap();
    unsafe { device.device.bind_buffer_memory(buffer, buffer_mem, 0) }.unwrap();

    // note: memory requested size does not necessarily means the actual size of the buffer
    (buffer, buffer_mem, mem_req.size)
}

// https://github.com/MaikKlein/ash/blob/master/examples/src/lib.rs#L120
fn find_memtype_ind(
    mem_req: &vk::MemoryRequirements,
    mem_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    mem_prop.memory_types[..mem_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(ind, mem_type)| {
            (1 << ind) & mem_req.memory_type_bits != 0
                && mem_type.property_flags & flags == flags
        })
        .map(|(ind, _mem_type)| ind as _)
}


impl CleanupVkObj for vk::ImageView {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.destroy_image_view(*self, None);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {}
}

impl CleanupVkObj for vk::Sampler {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.destroy_sampler(*self, None);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {}
}

impl CleanupVkObj for BufferMemoryMTXG {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.destroy_buffer(self.buffer, None);
        device.device.free_memory(self.memory, None);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {}
}

impl CleanupVkObj for ImageMemoryMTXG {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.destroy_image(self.image, None);
        device.device.free_memory(self.memory, None);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {}
}

impl CleanupVkObj for UniformBufferMTXG {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.destroy_buffer(self.buffer, None);  // the buffer used
        device.device.free_memory(self.memory, None);  // the actual memory where data is stored
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {}
}
