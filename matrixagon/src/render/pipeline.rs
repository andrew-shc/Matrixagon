use ash::vk;

use renderer::pipeline::GraphicsPipelineMTXG;
use renderer::device::DeviceMTXG;
use renderer::CleanupVkObj;


#[derive(Copy, Clone)]
pub struct CreateRenderpass {
    pub(super) renderpass: vk::RenderPass,
    pub(super) depth_enabled: bool,
}

impl CleanupVkObj for CreateRenderpass {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        self.renderpass.cleanup(device);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        self.renderpass.cleanup_recreation(device);
    }
}


#[derive(Clone)]
pub struct CreateGraphicPipelines {
    pub(super) pipeline: GraphicsPipelineMTXG,
}

impl CleanupVkObj for CreateGraphicPipelines {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        self.pipeline.cleanup(device);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        self.pipeline.cleanup_recreation(device);
    }
}

