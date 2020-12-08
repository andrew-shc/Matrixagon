use ash::vk;

use renderer::descriptors::{DescriptorSetsMTXG, SetBindingMTXG};
use renderer::device::DeviceMTXG;
use renderer::command_buffers::{PresenterMTXG};
use renderer::CleanupVkObj;

use crate::render::{ShaderStages, Renderer};
use crate::render::buffers::{CreateImageMemory, CreateUniformBuffer};

#[derive(Clone)]
enum BindData {
    BindUniform(CreateUniformBuffer),
    BindSampler(CreateImageMemory),
}


#[derive(Clone)]
pub struct CreateDescriptors {
    descriptor: Option<DescriptorSetsMTXG>,
    binding_data: Vec<(SetBindingMTXG, BindData)>,  // a deferred creation of binding for creating the descriptor
    index: u32,
    copies: u32,  // how many copies of each of the uniform bindings (specifically) are needed
}

impl CreateDescriptors {
    pub fn new(copies: u32) -> CreateDescriptors {
        Self {
            descriptor: None,
            binding_data: Vec::new(),
            index: 0,
            copies: copies,
        }
    }

    // bindings are in order

    pub fn bind_uniform(&mut self, index: u32, stage: ShaderStages, data: CreateUniformBuffer) {
        let uniform_binding = SetBindingMTXG::new(index, vk::DescriptorType::UNIFORM_BUFFER, stage.into());
        self.binding_data.push((uniform_binding, BindData::BindUniform(data)));
    }

    pub fn bind_sampler(&mut self, index: u32, stage: ShaderStages, data: CreateImageMemory) {
        let sampler_binding = SetBindingMTXG::new(index, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, stage.into());
        self.binding_data.push((sampler_binding, BindData::BindSampler(data)));
    }

    // TODO: add a recreation option: so no need to remove the SetLayout if the set is same

    pub fn build(&mut self, device: &DeviceMTXG) {
        // call build method only once
        if let None = self.descriptor {
            let set_bindings = self.binding_data.iter().map(|bd|bd.0).collect::<Vec<_>>();

            let descriptors = DescriptorSetsMTXG::new(device, set_bindings.as_slice(), self.copies);

            for ind in 0..self.binding_data.len() {
                match &self.binding_data[ind].1 {
                    BindData::BindUniform(ub) => {
                        descriptors.bind_uniform(ind as u32, ub.uniform_buffers.as_slice(), true);
                    },
                    BindData::BindSampler(sb) => {
                        descriptors.bind_sampler(ind as u32, &[sb.image_view], sb.image_sampler.retrieve_sampler(), false);
                    }
                }
            }
            self.descriptor = Some(descriptors);
        } else {
            panic!("Cannot call build on CreateDescriptors more than once!");
        }
    }

    pub(super) fn retrieve_descriptor(&self) -> &DescriptorSetsMTXG {
        self.descriptor.as_ref().expect("Failed to call build() method to retrieve the descriptor")
    }
}

impl CleanupVkObj for CreateDescriptors {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        if let Some(d) = &self.descriptor {
            d.cleanup(device);
        }
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        if let Some(d) = &self.descriptor {
            d.cleanup_recreation(device);
        }
    }
}


// CreateCommandBuffer deemed useless because once the command buffer is created,
// it gets immediately returned into a presenter


#[derive(Clone)]
pub struct CreatePresenter {
    pub(super) presenter: PresenterMTXG,

}

impl CreatePresenter {
    pub fn present(&mut self, render: &Renderer) {
        self.presenter.submit_and_present(&render.swapchain);
    }

    pub fn suboptimal(&mut self) -> bool {
        self.presenter.check_suboptimal_once()
    }
}

impl CleanupVkObj for CreatePresenter {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        self.presenter.cleanup(device);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        self.presenter.cleanup_recreation(device);
    }
}

