use ash::vk;

use renderer::buffer::{ImageMemoryMTXG, create_sampler, BufferMemoryMTXG, UniformBufferMTXG};
use renderer::device::DeviceMTXG;
use renderer::CleanupVkObj;


pub enum SamplerFilter {
    Nearest,
    Linear,
}

impl From<SamplerFilter> for vk::Filter {
    fn from(item: SamplerFilter) -> Self {
        match item {
            SamplerFilter::Nearest => {vk::Filter::NEAREST},
            SamplerFilter::Linear => {vk::Filter::LINEAR},
        }
    }
}

pub enum SamplerAddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

impl From<SamplerAddressMode> for vk::SamplerAddressMode {
    fn from(item: SamplerAddressMode) -> Self {
        match item {
            SamplerAddressMode::Repeat => {vk::SamplerAddressMode::REPEAT},
            SamplerAddressMode::MirroredRepeat => {vk::SamplerAddressMode::MIRRORED_REPEAT},
            SamplerAddressMode::ClampToEdge => {vk::SamplerAddressMode::CLAMP_TO_EDGE},
            SamplerAddressMode::ClampToBorder => {vk::SamplerAddressMode::CLAMP_TO_BORDER},
        }
    }
}


#[derive(Clone)]
pub struct CreateSampler {
    pub(super) mag_filter: vk::Filter,
    pub(super) min_filter: vk::Filter,
    pub(super) addr_mode_u: vk::SamplerAddressMode,
    pub(super) addr_mode_v: vk::SamplerAddressMode,
    pub(super) anistropy_enable: bool,
    pub(super) max_anistropy: f32,
    // TODO: mipmapping
    pub(super) mipmapping: bool,
    sampler: Option<vk::Sampler>,
}

impl CreateSampler {
    pub(super) fn new() -> Self {
        Self {
            mag_filter: vk::Filter::NEAREST,
            min_filter: vk::Filter::NEAREST,
            addr_mode_u: vk::SamplerAddressMode::REPEAT,
            addr_mode_v: vk::SamplerAddressMode::REPEAT,
            anistropy_enable: false,
            max_anistropy: 1.0,
            mipmapping: false,
            sampler: None,
        }
    }

    pub fn filter(&mut self, mag_filter: SamplerFilter, min_filter: SamplerFilter) {
        if let Some(_) = self.sampler { panic!("Sampler has been created already"); }
        self.mag_filter = mag_filter.into();
        self.min_filter = min_filter.into();
    }

    pub fn address_mode(&mut self, u: SamplerAddressMode, v: SamplerAddressMode) {
        if let Some(_) = self.sampler { panic!("Sampler has been created already"); }
        self.addr_mode_u = u.into();
        self.addr_mode_v = v.into();
    }

    pub fn anisotropy(&mut self, max_anisotropy: f32) {
        if let Some(_) = self.sampler { panic!("Sampler has been created already"); }
        self.anistropy_enable = true;
        self.max_anistropy = max_anisotropy;
    }

    pub fn mipmapping(&mut self) {
        if let Some(_) = self.sampler { panic!("Sampler has been created already"); }
        self.mipmapping = true;
    }

    pub fn build(&mut self, device: &DeviceMTXG) {
        self.sampler = Some(create_sampler(device, self.mag_filter,self.min_filter,
                                           self.addr_mode_u,self.addr_mode_v,
                                           self.anistropy_enable, self.max_anistropy));
    }

    pub(super) fn retrieve_sampler(&self) -> vk::Sampler {
        self.sampler.expect("Sampler has not been built yet")
    }
}

impl CleanupVkObj for CreateSampler {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        if let Some(s) = self.sampler {
            s.cleanup(device);
        }
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        if let Some(s) = self.sampler {
            s.cleanup_recreation(device);
        }
    }
}


#[derive(Clone)]
pub struct CreateImageMemory {
    pub(super) image_mem: ImageMemoryMTXG,
    pub(super) image_view: vk::ImageView,
    pub(super) image_sampler: CreateSampler,
}

impl CleanupVkObj for CreateImageMemory {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        self.image_view.cleanup(device);
        self.image_mem.cleanup(device);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        self.image_view.cleanup_recreation(device);
        self.image_mem.cleanup_recreation(device);
    }
}


#[derive(Clone)]
pub struct CreateBufferMemory {
    pub(super) buffer_mem: BufferMemoryMTXG,
}

impl CleanupVkObj for CreateBufferMemory {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        self.buffer_mem.cleanup(device);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        self.buffer_mem.cleanup_recreation(device);
    }
}


#[derive(Clone)]
pub struct CreateUniformBuffer {
    pub(super) uniform_buffers: Vec<UniformBufferMTXG>,
}

impl CreateUniformBuffer {
    pub fn update<T: Copy>(&self, device: &DeviceMTXG, data: T) {
        for ub in &self.uniform_buffers {
            ub.update(device, data);
        }
    }
}

impl CleanupVkObj for CreateUniformBuffer {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        self.uniform_buffers.cleanup(device);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        self.uniform_buffers.cleanup_recreation(device);
    }
}

