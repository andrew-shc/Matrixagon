use ash::vk;
use ash::version::DeviceV1_0;

use std::ffi::CString;
use std::io::Cursor;

use crate::device::DeviceMTXG;
use crate::descriptors::DescriptorSetsMTXG;
use crate::CleanupVkObj;


pub trait VertexInfo {
    fn attributes() -> Vec<vk::VertexInputAttributeDescription>;
}

#[derive(Clone)]
pub struct GraphicsPipelineMTXG {
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) layout: vk::PipelineLayout,
    pub(crate) polygon: vk::PolygonMode,
    pub(crate) cull: vk::CullModeFlags,
    pub(crate) alpha: bool,
    pub(crate) depth: bool,
    pub(crate) shaders: Vec<vk::ShaderModule>,  // [vertex shader, fragment shader]
    pub(crate) renderpass: vk::RenderPass,  // stores renderpass for recreation use
    pub(crate) vert_bindings: vk::VertexInputBindingDescription,
    pub(crate) vert_attributes: Vec<vk::VertexInputAttributeDescription>,
}

impl GraphicsPipelineMTXG {
    pub fn new(device: &DeviceMTXG,
               extent: vk::Extent2D,
               renderpass: vk::RenderPass,
               descriptors: Option<&DescriptorSetsMTXG>,
               vertex: &str,
               fragment: &str,
               vert_bindings: vk::VertexInputBindingDescription,
               vert_attributes: Vec<vk::VertexInputAttributeDescription>,
               polygon: vk::PolygonMode,
               cull: vk::CullModeFlags,
               alpha: bool,
               depth: bool) -> Self {
        let vert_modl = Self::load_shader(&device.device, vertex, device.debug_mode);
        let frag_modl = Self::load_shader(&device.device, fragment, device.debug_mode);

        // graphics pipeline

        let main_func = CString::new("main").unwrap();

        let vertx_shd_sinfo = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: vert_modl,
            p_name: main_func.as_ptr(),  // shader module function entry point
            ..Default::default()
        };

        let fragm_shd_sinfo = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: frag_modl,
            p_name: main_func.as_ptr(),  // shader module function entry point
            ..Default::default()
        };

        let shader_stages = [vertx_shd_sinfo, fragm_shd_sinfo];

        let vertx_inp_cinfo = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &vert_bindings,
            vertex_attribute_description_count: vert_attributes.len() as u32,
            p_vertex_attribute_descriptions: vert_attributes.as_slice().as_ptr(),
            ..Default::default()
        };

        let inp_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        };

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D {
                x: 0,
                y: 0,
            },
            extent: extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            p_viewports: &viewport,
            scissor_count: 1,
            p_scissors: &scissor,
            ..Default::default()
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: polygon,
            line_width: 1.0,
            cull_mode: cull,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            ..Default::default()
        };

        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let color_blend_attach = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::all(),
            blend_enable: vk::FALSE,
            ..Default::default()
        };

        let color_blending_cinfo = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: vk::FALSE,
            attachment_count: 1,
            p_attachments: &color_blend_attach,
            ..Default::default()
        };

        let pipeline_layout_cinfo = if let Some(s) = descriptors {
            vk::PipelineLayoutCreateInfo {
                set_layout_count: 1,
                p_set_layouts: &s.set_layout,
                ..Default::default()
            }
        } else {
            vk::PipelineLayoutCreateInfo::default()
        };

        let layout = unsafe { device.device.create_pipeline_layout( &pipeline_layout_cinfo, None) }.unwrap();

        let graphics_pipeline_cinfo = vk::GraphicsPipelineCreateInfo {
            // stages programmed in a shader
            stage_count: 2,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertx_inp_cinfo,
            p_input_assembly_state: &inp_assembly,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterizer,
            p_multisample_state: &multisampling,
            p_depth_stencil_state: if depth {&Self::generate_depth_stencil()} else {std::ptr::null()},
            p_color_blend_state: &color_blending_cinfo,
            // p_dynamic_state: std::ptr::null(),
            layout: layout,
            render_pass: renderpass,
            subpass: 0,
            // base_pipeline_handle: vk::Pipeline::null(),
            // base_pipeline_index: -1,
            ..Default::default()
        };

        let graphics_pipeline = unsafe {
            device.device.create_graphics_pipelines(vk::PipelineCache::null(), &[graphics_pipeline_cinfo], None)
        }.expect("Failed to create graphics pipeline")[0];

        Self {
            pipeline: graphics_pipeline,
            layout: layout,
            polygon: polygon,
            cull: cull,
            alpha: alpha,
            depth: depth,
            shaders: vec![vert_modl, frag_modl],
            renderpass: renderpass,
            vert_bindings: vert_bindings,
            vert_attributes: vert_attributes,
        }
    }

    pub fn recreate_pipeline(&mut self,
                             device: &DeviceMTXG,
                             extent: vk::Extent2D) {
        // graphics pipeline

        let main_func = CString::new("main").unwrap();

        let vertx_shd_sinfo = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: self.shaders[0],
            p_name: main_func.as_ptr(),  // shader module function entry point
            ..Default::default()
        };

        let fragm_shd_sinfo = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: self.shaders[1],
            p_name: main_func.as_ptr(),  // shader module function entry point
            ..Default::default()
        };

        let shader_stages = [vertx_shd_sinfo, fragm_shd_sinfo];

        let vertx_inp_cinfo = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &self.vert_bindings,
            vertex_attribute_description_count: self.vert_attributes.len() as u32,
            p_vertex_attribute_descriptions: self.vert_attributes.as_slice().as_ptr(),
            ..Default::default()
        };

        let inp_assembly = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        };

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D {
                x: 0,
                y: 0,
            },
            extent: extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            p_viewports: &viewport,
            scissor_count: 1,
            p_scissors: &scissor,
            ..Default::default()
        };

        let rasterizer = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: self.polygon,
            line_width: 1.0,
            cull_mode: self.cull,
            front_face: vk::FrontFace::CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            ..Default::default()
        };

        let multisampling = vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: vk::FALSE,
            rasterization_samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let color_blend_attach = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::all(),
            blend_enable: vk::FALSE,
            ..Default::default()
        };

        let color_blending_cinfo = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: vk::FALSE,
            attachment_count: 1,
            p_attachments: &color_blend_attach,
            ..Default::default()
        };

        let graphics_pipeline_cinfo = vk::GraphicsPipelineCreateInfo {
            // stages programmed in a shader
            stage_count: 2,
            p_stages: shader_stages.as_ptr(),
            p_vertex_input_state: &vertx_inp_cinfo,
            p_input_assembly_state: &inp_assembly,
            p_viewport_state: &viewport_state,
            p_rasterization_state: &rasterizer,
            p_multisample_state: &multisampling,
            p_depth_stencil_state: if self.depth {&Self::generate_depth_stencil()} else {std::ptr::null()},
            p_color_blend_state: &color_blending_cinfo,
            // p_dynamic_state: std::ptr::null(),
            layout: self.layout,
            render_pass: self.renderpass,
            subpass: 0,
            // base_pipeline_handle: vk::Pipeline::null(),
            // base_pipeline_index: -1,
            ..Default::default()
        };

        self.pipeline = unsafe {
            device.device.create_graphics_pipelines(vk::PipelineCache::null(), &[graphics_pipeline_cinfo], None)
        }.expect("Failed to recreate graphics pipeline")[0];
    }

    fn generate_depth_stencil() -> vk::PipelineDepthStencilStateCreateInfo {
        vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,
            stencil_test_enable: vk::FALSE,
            ..Default::default()
        }
    }

    pub fn create_renderpass(device: &DeviceMTXG, format: vk::Format) -> vk::RenderPass {
        let color_attach = vk::AttachmentDescription {
            format: format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let color_attach_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attach_ref,
            ..Default::default()
        };

        let subpass_dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::default(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let renderpass_cinfo = vk::RenderPassCreateInfo {
            attachment_count: 1,
            p_attachments: &color_attach,
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &subpass_dependency,
            ..Default::default()
        };

        unsafe { device.device.create_render_pass(&renderpass_cinfo, None) }.expect("Failed to create a renderpass")
    }

    pub fn create_renderpass_with_depth(device: &DeviceMTXG, color_format: vk::Format, depth_format: vk::Format) -> vk::RenderPass {
        let color_attach = vk::AttachmentDescription {
            format: color_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
            ..Default::default()
        };

        let depth_attach = vk::AttachmentDescription {
            format: depth_format,
            samples: vk::SampleCountFlags::TYPE_1,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };

        let color_attach_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth_attach_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            color_attachment_count: 1,
            p_color_attachments: &color_attach_ref,
            p_depth_stencil_attachment: &depth_attach_ref,
            ..Default::default()
        };

        let subpass_dependency = vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask: vk::AccessFlags::empty(),
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ..Default::default()
        };

        let attachments = &[color_attach, depth_attach];

        let renderpass_cinfo = vk::RenderPassCreateInfo {
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &subpass_dependency,
            ..Default::default()
        };

        unsafe { device.device.create_render_pass(&renderpass_cinfo, None) }.expect("Failed to create a renderpass")
    }

    fn load_shader(device: &ash::Device, fname: &str, debug_mode: bool) -> vk::ShaderModule {
        if debug_mode {
            println!("Loading shader file {:?}", fname);
        }
        let code = ash::util::read_spv(&mut Cursor::new(
            std::fs::read(fname).expect(&format!("Failed to find the shader file {:?}", fname)[..])
        )).expect("Shader file failed to load");

        let shader_modl_cinfo = vk::ShaderModuleCreateInfo {
            code_size: code.len() * std::mem::size_of::<u32>(),  // code size are in bytes, but code data is aligned to u32 (4 bytes)
            p_code: code.as_ptr(),
            ..Default::default()
        };

        unsafe { device.create_shader_module(&shader_modl_cinfo, None) }.expect("Failed to create the shader module")
    }
}

impl CleanupVkObj for GraphicsPipelineMTXG {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.destroy_pipeline_layout(self.layout, None);
        device.device.destroy_pipeline(self.pipeline, None);

        device.device.destroy_shader_module(self.shaders[0], None);
        device.device.destroy_shader_module(self.shaders[1], None);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        device.device.destroy_pipeline(self.pipeline, None);
    }
}

#[allow(unused_variables)]
impl CleanupVkObj for vk::RenderPass {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        device.device.destroy_render_pass(*self, None);
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {}
}
