use ash::vk;
use ash::version::DeviceV1_0;

use crate::device::DeviceMTXG;
use crate::swapchain::SwapchainMTXG;
use crate::pipeline::GraphicsPipelineMTXG;
use crate::buffer::{ImageMemoryMTXG, BufferMemoryMTXG};
use crate::descriptors::DescriptorSetsMTXG;
use crate::CleanupVkObj;


pub const MAX_INFLIGHT_FRAMES: u32 = 2;


/*

// map design

CommandPool

CommandBuffer

ExecutableRendering

// code

let cmdpl = CommandPool::new();  // free at end

// free at swapchain recreation; so we can control whether it is using a new command pool
let cmdbf = CommandBuffer::new(device, fb, cmdpl)
    .cmds()
    .cmds()
    .build();

// Presenter executes the command buffer

let prsnt = Presenter::new(cmdbuf);

prsnt.submit_and_present();

 */

#[derive(Copy, Clone)]
pub struct CommandPoolMTXG {
    pool: vk::CommandPool,
}

impl CommandPoolMTXG {
    pub fn new(device: &DeviceMTXG) -> Self {
        let command_pool_cinfo = vk::CommandPoolCreateInfo {
            queue_family_index: device.graphics_queue_fam_id,
            ..Default::default()
        };
        let cmd_pool = unsafe { device.device.create_command_pool(&command_pool_cinfo, None) }.expect("Failed to create the command pool");

        Self {
            pool: cmd_pool,
        }
    }
}

pub struct CommandBufferMTXG {
    pub (crate) device: DeviceMTXG,
    pub (crate) buffer: Vec<vk::CommandBuffer>,
    pub (crate) pool: CommandPoolMTXG,
}

impl CommandBufferMTXG {
    pub fn new(device: &DeviceMTXG, pool: &CommandPoolMTXG, copies: u32) -> Self {
        let command_buffer_ainfo = vk::CommandBufferAllocateInfo {
            command_pool: pool.pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: copies,
            ..Default::default()
        };

        let command_buffers = unsafe { device.device.allocate_command_buffers(&command_buffer_ainfo) }.expect("Failed to create the commandbuffers");

        Self {
            device: device.clone(),
            buffer: command_buffers,
            pool: *pool,
        }
    }

    // begins command recording
    pub fn start_recording(&self, flags: vk::CommandBufferUsageFlags) {
        let cmd_buffer_ainfo = vk::CommandBufferBeginInfo {
            flags: flags,
            ..Default::default()
        };

        for i in 0..self.buffer.len() {
            unsafe { self.device.device.begin_command_buffer(self.buffer[i], &cmd_buffer_ainfo) }.expect("Failed to begin the command buffer recording");
        }
    }

    // TODO-CHECKED: commands to be added later
    // copy_staging_buffer();
    // draw_pipeline_instanced();

    pub fn copy_buffer(&self, src: &BufferMemoryMTXG, dst: &BufferMemoryMTXG) {
        let copy_info = vk::BufferCopy {
            size: src.size,
            ..Default::default()
        };

        for i in 0..self.buffer.len() {
            unsafe { self.device.device.cmd_copy_buffer(self.buffer[i], src.buffer, dst.buffer, &[copy_info]) };
        }
    }

    // to optimize the image memory layout for whatever operation it will be doing
    pub fn transition_img_layout(&self,
                                 img: &ImageMemoryMTXG,
                                 old_lyt: vk::ImageLayout,
                                 new_lyt: vk::ImageLayout) {
        let mut img_barrier = vk::ImageMemoryBarrier {
            old_layout: old_lyt,
            new_layout: new_lyt,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: img.image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: if new_lyt == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
                    if DeviceMTXG::has_stencil_format(img.format) {
                        // must include stencil image aspect flags if the format states the depth sampler contains stencil bit
                        vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
                    } else {
                        vk::ImageAspectFlags::DEPTH
                    }
                } else {
                    vk::ImageAspectFlags::COLOR
                },
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1
            },
            ..Default::default()
        };

        let src_stage;
        let dst_stage;

        if old_lyt == vk::ImageLayout::UNDEFINED && new_lyt == vk::ImageLayout::TRANSFER_DST_OPTIMAL {
            img_barrier.src_access_mask = vk::AccessFlags::empty();
            img_barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

            src_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            dst_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_lyt == vk::ImageLayout::TRANSFER_DST_OPTIMAL && new_lyt == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
            img_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            img_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            src_stage = vk::PipelineStageFlags::TRANSFER;
            dst_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else if old_lyt == vk::ImageLayout::UNDEFINED && new_lyt == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            img_barrier.src_access_mask = vk::AccessFlags::empty();
            img_barrier.dst_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;

            src_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            dst_stage = vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
        } else {
            panic!(format!("Unsupported layout transition: from <{:?}> to <{:?}>", old_lyt, new_lyt));
        }

        for i in 0..self.buffer.len() {
            unsafe {
                self.device.device.cmd_pipeline_barrier(
                    self.buffer[i],
                    src_stage, dst_stage,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[img_barrier])
            };
        }
    }

    pub fn copy_buffer_to_image(&self, buf: &BufferMemoryMTXG, img: &ImageMemoryMTXG) {
        let copy_info = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D { width: img.width, height: img.height, depth: 1 },
        };

        for i in 0..self.buffer.len() {
            unsafe {
                self.device.device.cmd_copy_buffer_to_image(
                    self.buffer[i],
                    buf.buffer, img.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[copy_info])
            };
        }
    }

    pub fn begin_pass(&self,
                      renderpass: vk::RenderPass,
                      framebuffers: &Vec<vk::Framebuffer>,
                      extent: vk::Extent2D,
                      clear: (f32, f32, f32)) {
        // numbers of buffers remains same as framebuffers for maximal performance
        // so each command buffer can work on each of their framebuffers
        assert_eq!(self.buffer.len(), framebuffers.len());

        for i in 0..self.buffer.len() {
            let attachment_clear_values = &[
                vk::ClearValue {color: vk::ClearColorValue {float32: [clear.0, clear.1, clear.2, 1.0]}},
            ];

            let renderpass_binfo = vk::RenderPassBeginInfo {
                render_pass: renderpass,
                framebuffer: framebuffers[i],
                render_area: vk::Rect2D {
                    offset: Default::default(),
                    extent: extent,
                },
                clear_value_count: 1,
                p_clear_values: attachment_clear_values.as_ptr(),
                ..Default::default()
            };

            unsafe { self.device.device.cmd_begin_render_pass(self.buffer[i], &renderpass_binfo, vk::SubpassContents::INLINE) };
        }
    }

    pub fn begin_depth_pass(&self,
                            renderpass: vk::RenderPass,
                            framebuffers: &Vec<vk::Framebuffer>,
                            extent: vk::Extent2D,
                            clear: (f32, f32, f32)) {
        // numbers of buffers remains same as framebuffers for maximal performance
        // so each command buffer can work on each of their framebuffers
        assert_eq!(self.buffer.len(), framebuffers.len());

        for i in 0..self.buffer.len() {
            // the order of clear values are in relationship to the order when the renderpass created these subpasses
            let attachment_clear_values = &[
                vk::ClearValue {color: vk::ClearColorValue {float32: [clear.0, clear.1, clear.2, 1.0]}},
                vk::ClearValue {depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 }},
            ];

            let renderpass_binfo = vk::RenderPassBeginInfo {
                render_pass: renderpass,
                framebuffer: framebuffers[i],
                render_area: vk::Rect2D {
                    offset: Default::default(),
                    extent: extent,
                },
                clear_value_count: attachment_clear_values.len() as u32,
                p_clear_values: attachment_clear_values.as_ptr(),
                ..Default::default()
            };

            unsafe { self.device.device.cmd_begin_render_pass(self.buffer[i], &renderpass_binfo, vk::SubpassContents::INLINE) };
        }
    }

    pub fn draw_pipeline(&self,
                         pipeline: &GraphicsPipelineMTXG,
                         binding: u32,
                         buffer: &BufferMemoryMTXG,
                         sets: Option<&DescriptorSetsMTXG>) {
        let dscrp_sets = if let Some(s) = sets { s.sets.as_slice() } else { &[] };
        for i in 0..self.buffer.len() {
            unsafe { self.device.device.cmd_bind_pipeline(self.buffer[i], vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline) };
            unsafe { self.device.device.cmd_bind_vertex_buffers(self.buffer[i], binding, &[buffer.buffer], &[0]) };
            unsafe { self.device.device.cmd_bind_descriptor_sets(self.buffer[i], vk::PipelineBindPoint::GRAPHICS, pipeline.layout, 0, &[dscrp_sets[i]], &[]) };
            unsafe { self.device.device.cmd_draw(self.buffer[i], buffer.size as u32, 1, 0, 0) };
        }
    }

    // assumes each frames of sets for each DescriptorSetMTXG has more than or equal to number of image frames
    pub fn draw_pipeline_indexed(&self,
                                 pipeline: &GraphicsPipelineMTXG,
                                 binding: u32,
                                 vb: &BufferMemoryMTXG,
                                 ib: &BufferMemoryMTXG,
                                 ind_typ: vk::IndexType,
                                 sets: Option<&DescriptorSetsMTXG>) {
        let dscrp_sets = if let Some(s) = sets { s.sets.as_slice() } else { &[] };
        for i in 0..self.buffer.len() {
            unsafe { self.device.device.cmd_bind_pipeline(self.buffer[i], vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline) };
            unsafe { self.device.device.cmd_bind_vertex_buffers(self.buffer[i], binding, &[vb.buffer], &[0]) };
            unsafe { self.device.device.cmd_bind_index_buffer(self.buffer[i], ib.buffer, 0, ind_typ) };
            unsafe { self.device.device.cmd_bind_descriptor_sets(self.buffer[i], vk::PipelineBindPoint::GRAPHICS, pipeline.layout, 0, &[dscrp_sets[i]], &[]) };
            unsafe { self.device.device.cmd_draw_indexed(self.buffer[i], ib.len, 1, 0, 0, 0) };
        }
    }

    pub fn end_pass(&self) {
        for i in 0..self.buffer.len() {
            unsafe { self.device.device.cmd_end_render_pass(self.buffer[i]) };
        }
    }

    pub fn finish_recording(&self) {
        for i in 0..self.buffer.len() {
            unsafe { self.device.device.end_command_buffer(self.buffer[i]) }.expect("Failed to record the commands");
        }
    }

    // builds and submits this command buffer once
    pub fn build_submit_once(&self, device: &DeviceMTXG) {
        for i in 0..self.buffer.len() {
            let submit_info = vk::SubmitInfo {
                command_buffer_count: 1,
                p_command_buffers: &self.buffer[i],
                ..Default::default()
            };

            unsafe { device.device.queue_submit(device.graphics_queue, &[submit_info], vk::Fence::null()) }.unwrap();
            // TODO-CHECKED: put queue wait idle somewhere else as we can optimize the graphics to run while CPU does its thing before calling idle again for the next GPU cmd functions
            unsafe { device.device.queue_wait_idle(device.graphics_queue) }.unwrap();
        }
    }

    // builds all the required semaphores/fences to start the PresenterMTXG
    pub fn build(self, swapchain: &SwapchainMTXG) -> PresenterMTXG {
        let mut semaphores_img_acquired = Vec::new();
        let mut semaphores_render_finished = Vec::new();
        let mut fences_inflight = Vec::new();

        // creates all the sync object before passing onto PresenterMTXG
        let semaphore_cinfo = vk::SemaphoreCreateInfo::default();
        let fence_cinfo = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        // to use the MAX_INFLIGHT_FRAMES constant to denote these semaphores/fences are to be used concurrently
        // only when frames are in flight (are creating and be ready for presenting in the next moment)
        for i in 0..MAX_INFLIGHT_FRAMES {
            semaphores_img_acquired.push(
                unsafe { self.device.device.create_semaphore(&semaphore_cinfo, None) }.expect(&format!("Failed to create img_acquired semaphore on frame <{:?}>", i))
            );
            semaphores_render_finished.push(
                unsafe { self.device.device.create_semaphore(&semaphore_cinfo, None) }.expect(&format!("Failed to create render_finished semaphore on frame <{:?}>", i))
            );
            fences_inflight.push(
                unsafe { self.device.device.create_fence(&fence_cinfo, None) }.expect(&format!("Failed to create in-flight fence on frame <{:?}>", i))
            );
        }

        // if we were to recreate, sync objects would not matter since they are only created on once.
        PresenterMTXG {
            device: self.device,
            cmd_pool: self.pool,
            cmd_buffer: self.buffer,
            semaphores_img_acquired: semaphores_img_acquired,
            semaphores_render_finished: semaphores_render_finished,
            fences_inflight: fences_inflight,
            fences_inflighted: vec![None; swapchain.image_views.len()],
            current_frame: 0,
            suboptimal: false
        }
    }
}

#[derive(Clone)]
pub struct PresenterMTXG {
    pub (crate) device: DeviceMTXG,
    pub (crate) cmd_pool: CommandPoolMTXG,
    pub (crate) cmd_buffer: Vec<vk::CommandBuffer>,  // each command buffer executes each frames of framebuffer
    pub (crate) semaphores_img_acquired: Vec<vk::Semaphore>,
    pub (crate) semaphores_render_finished: Vec<vk::Semaphore>,
    pub (crate) fences_inflight: Vec<vk::Fence>,  // new fences in-flight waiting
    pub (crate) fences_inflighted: Vec<Option<vk::Fence>>,  // fences that are already in-flight(ed) and to be ready to be present again
    pub (crate) current_frame: usize,  // the current frame on which image index to render onto the swapchain
    pub (crate) suboptimal: bool,
}

impl PresenterMTXG {
    // this function changes the suboptimal status after invoked
    pub fn check_suboptimal_once(&mut self) -> bool {
        if self.suboptimal { self.suboptimal = false; true } else { false }
    }

    // when the cmd_buffer is recreated, use this to directly replace the in use in PresenterMTXG
    // without calling CommandBufferMTXG::build()
    pub fn replace_cmd_buffer(&mut self, cmd_buffer: CommandBufferMTXG) {
        self.cmd_buffer = cmd_buffer.buffer;
    }

    // the make function to submit and present the rendering to the screen
    pub fn submit_and_present(&mut self, swapchain: &SwapchainMTXG) {
        unsafe { self.device.device.wait_for_fences(&[self.fences_inflight[self.current_frame]], true, u64::max_value()) }.expect("");

        let acquired = unsafe { swapchain.swapchain_handler.acquire_next_image(swapchain.swapchain, u64::max_value(), self.semaphores_img_acquired[self.current_frame], vk::Fence::null()) };
        if self.device.debug_mode {
            println!("IMG ACQUIRE SUBOPTIMAL: {:?}", acquired);
        }

        if let Ok((img_ind, _)) = acquired {
            if let Some(fence_inflighted) = self.fences_inflighted[img_ind as usize] {
                unsafe { self.device.device.wait_for_fences(&[fence_inflighted], true, u64::max_value()) }.expect("");
            }
            self.fences_inflighted[img_ind as usize] = Some(self.fences_inflight[self.current_frame]);

            let queue_sinfo = vk::SubmitInfo {
                wait_semaphore_count: 1,
                p_wait_semaphores: &self.semaphores_img_acquired[self.current_frame],
                p_wait_dst_stage_mask: &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                command_buffer_count: 1,
                p_command_buffers: &self.cmd_buffer[img_ind as usize],
                signal_semaphore_count: 1,
                p_signal_semaphores: &self.semaphores_render_finished[self.current_frame],
                ..Default::default()
            };

            unsafe { self.device.device.reset_fences(&[self.fences_inflight[self.current_frame]]) }.expect("");

            unsafe { self.device.device.queue_submit(self.device.graphics_queue, &[queue_sinfo], self.fences_inflight[self.current_frame]) }.expect("Failed to submit the recorded command buffer");

            let present_info = vk::PresentInfoKHR {
                wait_semaphore_count: 1,
                p_wait_semaphores: &self.semaphores_render_finished[self.current_frame],
                swapchain_count: 1,
                p_swapchains: &swapchain.swapchain,
                p_image_indices: &img_ind,
                ..Default::default()
            };
            let suboptimal_present = unsafe { swapchain.swapchain_handler.queue_present(self.device.present_queue, &present_info) };
            if let Err(_) = suboptimal_present {
                self.suboptimal = true;
            }

            self.current_frame = (self.current_frame+1)%MAX_INFLIGHT_FRAMES as usize;
        } else {
            self.suboptimal = true;
        }
    }
}

#[allow(unused_variables)]
impl CleanupVkObj for CommandPoolMTXG {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.destroy_command_pool(self.pool, None);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {}
}

#[allow(unused_variables)]
impl CleanupVkObj for CommandBufferMTXG {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.free_command_buffers(self.pool.pool, self.buffer.as_slice());
        // command pool should be independently freed by CommandPoolMTXG struct
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        device.device.free_command_buffers(self.pool.pool, self.buffer.as_slice());
    }
}

#[allow(unused_variables)]
impl CleanupVkObj for PresenterMTXG {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        for i in 0..MAX_INFLIGHT_FRAMES {
            device.device.destroy_semaphore(self.semaphores_img_acquired[i as usize], None);
            device.device.destroy_semaphore(self.semaphores_render_finished[i as usize], None);
            device.device.destroy_fence(self.fences_inflight[i as usize], None);
        }
        device.device.free_command_buffers(self.cmd_pool.pool, self.cmd_buffer.as_slice());
        // command pool should be independently freed by CommandPoolMTXG struct
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        device.device.free_command_buffers(self.cmd_pool.pool, self.cmd_buffer.as_slice());
    }
}
