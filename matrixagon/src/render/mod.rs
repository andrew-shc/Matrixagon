use ash::vk;
use ash::extensions::khr;

use winit::window::Window;

use renderer::{InstanceMTXG, CleanupVkObj};
use renderer::device::DeviceMTXG;
use renderer::swapchain::SwapchainMTXG;
use renderer::buffer::{BufferMemoryMTXG, ImageMemoryMTXG, create_image_view, UniformBufferMTXG};
use renderer::command_buffers::{CommandBufferMTXG, CommandPoolMTXG};
use renderer::pipeline::{GraphicsPipelineMTXG, VertexInfo};

use crate::render::buffers::{CreateImageMemory, CreateSampler, CreateBufferMemory, CreateUniformBuffer};
use crate::render::command::{CreateDescriptors, CreatePresenter};
use crate::render::pipeline::{CreateRenderpass, CreateGraphicPipelines};

use std::mem;

pub mod buffers;
pub mod command;
pub mod pipeline;


// GOAL:
//      The goal of this render module is to enable a more flexible, cleaner syntax, and easier usage
//      towards the game itself. In other words, this renderer is targeted to this game specifically.

// TODO: replace all costly clones with references once possible

#[allow(dead_code)]
pub enum TextureFormat {
    Grayscale,
    GrayscaleAlpha,
    RGB,
    RGBA,
}

#[allow(dead_code)]
pub enum BufferUsages {
    Vertex,
    Index,
    TransferSrc,
    TransferDst,
    Uniform,
}

#[allow(dead_code)]
pub enum MemoryProperties {
    Host,  // memory managed by the host program (e.g. computer memory)
    Local,  // local device memory (e.g. GPU memory)
}

#[allow(dead_code)]
pub enum ShaderStages {
    Vertex,
    TessCtrl,
    TessEval,
    Geometry,
    Fragment,
}

// used in the command buffer creation
#[allow(dead_code)]
pub enum Commands {
    Draw(CreateGraphicPipelines, u32, CreateBufferMemory, Option<CreateDescriptors>),  // binding, vert buf, descriptors
    DrawIndexed(CreateGraphicPipelines, u32, CreateBufferMemory, CreateBufferMemory, Option<CreateDescriptors>),  // binding, vert buf, indx buf, descriptors
    DrawInstanced,
}

#[allow(dead_code)]
pub enum VertexRates {
    Vertex,
    Instance,
}

#[allow(dead_code)]
pub enum DataType {
    Vec2,
    Vec3,
}

impl From<ShaderStages> for vk::ShaderStageFlags  {
    fn from(item: ShaderStages) -> Self {
        match item {
            ShaderStages::Vertex => Self::VERTEX,
            ShaderStages::TessCtrl => Self::TESSELLATION_CONTROL,
            ShaderStages::TessEval => Self::TESSELLATION_EVALUATION,
            ShaderStages::Geometry => Self::GEOMETRY,
            ShaderStages::Fragment => Self::FRAGMENT,
        }
    }
}

impl From<DataType> for vk::Format {
    fn from(item: DataType) -> Self {
        match item {
            DataType::Vec2 => vk::Format::R32G32_SFLOAT,
            DataType::Vec3 => vk::Format::R32G32B32_SFLOAT,
        }
    }
}


// https://github.com/MaikKlein/ash/blob/master/examples/src/lib.rs#L17
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = mem::zeroed();
            (&b.$field as *const _ as isize) - (&b as *const _ as isize)
        }
    }};
}


// implements VertexInfo trait for any struct used for vertices
#[macro_export]
macro_rules! impl_vertex {
    ($obj:path, $($field:ident => $ty:expr,)+) => {
        impl renderer::pipeline::VertexInfo for $obj {
            fn attributes() -> Vec<ash::vk::VertexInputAttributeDescription> {
                let mut _counter = 0u32;
                vec![
                    $(
                        {
                            _counter += 1;
                            vk::VertexInputAttributeDescription {  // individual fields for each attributes
                                binding: 0,
                                location: _counter-1,
                                format: $ty.into(),
                                offset: offset_of!($obj, $field) as u32,
                            }
                        },
                    )*
                ]
            }
        }
    };
}

#[derive(Copy, Clone, Debug)]
struct CubeVertex {
    pos: [f32; 3],
    col: [f32; 3],
    crd: [f32; 2],
}

impl_vertex!(CubeVertex, pos => DataType::Vec3, );


#[derive(Clone)]
pub struct Renderer {
    instance: InstanceMTXG,
    device: DeviceMTXG,
    swapchain: SwapchainMTXG,
    renderpass: vk::RenderPass,  // main renderpass
    framebuffer: Vec<vk::Framebuffer>,
    command_pool: CommandPoolMTXG,
}

impl Renderer {
    // a constant to state whether there will be any depth sampling in this current Renderer
    const ENABLE_DEPTH: bool = true;

    pub fn new(window: &Window, app_name: &str, debug_mode: bool) -> (Self, DeviceMTXG) {
        let instance = InstanceMTXG::new(window, app_name, debug_mode, (1,2,0));
        let device = instance.find_device(vec![khr::Swapchain::name()], true);
        let swapchain = SwapchainMTXG::new(&instance, &device, 2, Self::ENABLE_DEPTH, None);
        // renderpass is used for the end user display screen, so any depth sampling will also must
        // include renderpasses with depth buffering enabled
        let renderpass = if Self::ENABLE_DEPTH {
            GraphicsPipelineMTXG::create_renderpass_with_depth(&device, swapchain.image_format().format, device.find_depth_format())
        } else {
            GraphicsPipelineMTXG::create_renderpass(&device, swapchain.image_format().format)
        };
        let framebuffer = swapchain.get_framebuffers(&device, renderpass);
        let cmd_pool = CommandPoolMTXG::new(&device);

        (
            Self {
                instance: instance,
                device: device.clone(),
                swapchain: swapchain,
                renderpass: renderpass,
                framebuffer: framebuffer,
                command_pool: cmd_pool,
            },
            device.clone(),
        )
    }

    // TODO: replace Extent2D with dimension
    pub fn current_extent(&self) -> vk::Extent2D {
        self.swapchain.current_extent()
    }

    // recreates objects internally once invoked
    pub fn recreate(&mut self, mut graphics_pipelines: Vec<&mut CreateGraphicPipelines>, others: Vec<&dyn CleanupVkObj>) {
        // note: renderpass does not need to be recreate because its independent from window sizes
        // note: CreateGraphicsPipelines are being internally recreated

        self.device.wait_idle();

        unsafe {
            // TODO: make a tracker to track objects to be cleaned up

            let mut objects: Vec<&dyn CleanupVkObj> = Vec::new();
            objects.push(&self.swapchain);
            for graphics_pipeline in graphics_pipelines.iter() {
                objects.push(&graphics_pipeline.pipeline);
            }
            objects.push(&self.framebuffer);
            for o in others {
                objects.push(o);
            }

            // swapchain recreation / object cleanup
            self.swapchain = self.swapchain.clone().recreate_swapchain(&self.instance, &self.device, objects);

            // retrieving recreated object
            self.framebuffer = self.swapchain.get_framebuffers(&self.device, self.renderpass);

            for graphics_pipeline in graphics_pipelines.iter_mut() {
                graphics_pipeline.pipeline.recreate_pipeline(&self.device, self.swapchain.current_extent());
            }
        }
    }

    // gets the same renderpass used in the Render struct
    pub fn get_renderpass(&self) -> CreateRenderpass {
        CreateRenderpass {
            renderpass: self.renderpass,
            depth_enabled: Self::ENABLE_DEPTH,
        }
    }

    // once the sampler is used, changing the sampler properties does not change the sampler itself
    pub fn create_sampler(&self) -> CreateSampler {
        CreateSampler::new()
    }

    // TODO: this is staged, maybe add a non-staged image memory in the future?
    pub fn create_sampled_image(&self, txtr_buf: Vec<u8>, width: u32, height: u32, format: TextureFormat, sampler: CreateSampler) -> CreateImageMemory {
        let txtr_fmt = match format {
            TextureFormat::Grayscale => {vk::Format::R8_SRGB}
            TextureFormat::GrayscaleAlpha => {vk::Format::R8G8_SRGB}
            TextureFormat::RGB => {vk::Format::R8G8B8_SRGB}
            TextureFormat::RGBA => {vk::Format::R8G8B8A8_SRGB}
        };

        let stage_img = BufferMemoryMTXG::new::<u8>(&self.instance, &self.device, txtr_buf.len(), vk::BufferUsageFlags::TRANSFER_SRC,
                                                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
            .fill(txtr_buf);
        let texture_img = ImageMemoryMTXG::new(&self.instance, &self.device, width, height, txtr_fmt,
                                               vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED, vk::MemoryPropertyFlags::DEVICE_LOCAL);

        let cmd_buf = CommandBufferMTXG::new(&self.device, &self.command_pool, 1);
        cmd_buf.start_recording(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        cmd_buf.transition_img_layout(&texture_img, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
        cmd_buf.copy_buffer_to_image(&stage_img, &texture_img);
        cmd_buf.transition_img_layout(&texture_img, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        cmd_buf.finish_recording();
        cmd_buf.build_submit_once(&self.device);

        unsafe { cmd_buf.cleanup(&self.device) };
        unsafe { stage_img.cleanup(&self.device) };

        let image_view = create_image_view(&self.device, texture_img.image, txtr_fmt, vk::ImageAspectFlags::COLOR);

        CreateImageMemory {
            image_mem: texture_img,
            image_view: image_view,
            image_sampler: sampler,
        }
    }

    pub fn create_buffer<D: Copy>(&self, usage: BufferUsages, prop: MemoryProperties, data: Vec<D>) -> CreateBufferMemory {
        let buf_usages = match usage {
            BufferUsages::Vertex => {vk::BufferUsageFlags::VERTEX_BUFFER}
            BufferUsages::Index => {vk::BufferUsageFlags::INDEX_BUFFER}
            BufferUsages::TransferSrc => {vk::BufferUsageFlags::TRANSFER_SRC}
            BufferUsages::TransferDst => {vk::BufferUsageFlags::TRANSFER_DST}
            BufferUsages::Uniform => {vk::BufferUsageFlags::UNIFORM_BUFFER}
        };
        let mem_prop_flags = match prop {
            MemoryProperties::Host => {vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT},
            MemoryProperties::Local => {vk::MemoryPropertyFlags::DEVICE_LOCAL}
        };

        let buf_mem = BufferMemoryMTXG::new::<D>(&self.instance, &self.device, data.len(), buf_usages, mem_prop_flags)
            .fill(data);

        CreateBufferMemory {
            buffer_mem: buf_mem,
        }
    }

    // all uniforms corresponds to each one of the framebuffers: so to maximize performance
    pub fn create_uniform_buffer<U>(&self) -> CreateUniformBuffer {
        let mut uniform_buf_vec = Vec::with_capacity(self.swapchain.image_count());
        for _ in 0..self.swapchain.image_count() {
            uniform_buf_vec.push(UniformBufferMTXG::new::<U>(&self.instance, &self.device));
        }

        CreateUniformBuffer {
            uniform_buffers: uniform_buf_vec,
        }
    }

    pub fn create_descriptors(&self) -> CreateDescriptors {
        CreateDescriptors::new(self.swapchain.image_count() as u32)
    }

    pub fn create_renderpass(&self) -> CreateRenderpass {
        CreateRenderpass {
            renderpass: GraphicsPipelineMTXG::create_renderpass(&self.device, self.swapchain.image_format().format),
            depth_enabled: false,
        }
    }

    pub fn create_renderpass_with_depth(&self) -> CreateRenderpass {
        CreateRenderpass {
            renderpass: GraphicsPipelineMTXG::create_renderpass_with_depth(&self.device, self.swapchain.image_format().format, self.device.find_depth_format()),
            depth_enabled: true,
        }
    }

    pub fn create_graphics_pipeline<V: VertexInfo>(&self,
                                                   renderpass: &CreateRenderpass,
                                                   descriptors: &CreateDescriptors,
                                                   vertex_file: &str,
                                                   fragment_file: &str,
                                                   input_rate: VertexRates,
                                                   cull: bool,
                                                   alpha: bool) -> CreateGraphicPipelines {
        let binding = vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<V>() as u32,
            input_rate: if let VertexRates::Vertex = input_rate {vk::VertexInputRate::VERTEX} else {vk::VertexInputRate::INSTANCE},
        };

        CreateGraphicPipelines {
            pipeline: GraphicsPipelineMTXG::new(
                &self.device,
                self.swapchain.current_extent(),
                renderpass.renderpass,
                Some(descriptors.retrieve_descriptor()),
                vertex_file,
                fragment_file,
                binding,
                V::attributes(),
                vk::PolygonMode::FILL,
                if cull {vk::CullModeFlags::BACK} else {vk::CullModeFlags::NONE},
                alpha,
                true
            ),
        }
    }

    pub fn create_vertex_buffer<D: Copy>(&self, data: Vec<D>) -> CreateBufferMemory {
        self.create_buffer::<D>(
            BufferUsages::Vertex,
            MemoryProperties::Host,
            data)
    }

    pub fn create_index_buffer(&self, data: Vec<u32>) -> CreateBufferMemory {
        self.create_buffer::<u32>(
            BufferUsages::Index,
            MemoryProperties::Host,
            data)
    }

    // creates a simple command buffer
    pub fn create_command_buffers(&self,
                                  renderpass: CreateRenderpass,
                                  clear: (f32, f32, f32),
                                  cmds: Vec<Commands>) -> CreatePresenter {
        // COMMAND BUFFERING (presenting main command buffer)
        let cmd_buf = CommandBufferMTXG::new(&self.device, &self.command_pool, self.framebuffer.len() as u32);
        cmd_buf.start_recording(vk::CommandBufferUsageFlags::empty());
        cmd_buf.begin_depth_pass(renderpass.renderpass, &self.framebuffer, self.swapchain.current_extent(), clear);

        for cmd in &cmds {
            match cmd {
                Commands::Draw(grph_pipeline, bind, vb, dscrps) => {
                    let dscrps = if let Some(d) = dscrps { Some(d.retrieve_descriptor()) } else { None };
                    cmd_buf.draw_pipeline(&grph_pipeline.pipeline,*bind,&vb.buffer_mem,dscrps);
                },
                Commands::DrawIndexed(grph_pipeline, bind, vb, ib, dscrps) => {
                    let dscrps = if let Some(d) = dscrps { Some(d.retrieve_descriptor()) } else { None };
                    cmd_buf.draw_pipeline_indexed(&grph_pipeline.pipeline, *bind, &vb.buffer_mem, &ib.buffer_mem, vk::IndexType::UINT32, dscrps);
                },
                Commands::DrawInstanced => {
                    // TODO
                },
            }
        }

        cmd_buf.end_pass();
        cmd_buf.finish_recording();
        let present = cmd_buf.build(&self.swapchain);

        CreatePresenter {
            presenter: present,
        }
    }

    // replaces current internal command buffer with a new command buffer
    pub fn replace_command_buffers(&self,
                                  old_cmd_buf: &mut CreatePresenter,
                                  renderpass: CreateRenderpass,
                                  clear: (f32, f32, f32),
                                  pipelines: Vec<Commands>) {
        // COMMAND BUFFERING (presenting main command buffer)
        let cmd_buf = CommandBufferMTXG::new(&self.device, &self.command_pool, self.framebuffer.len() as u32);
        cmd_buf.start_recording(vk::CommandBufferUsageFlags::empty());
        cmd_buf.begin_depth_pass(renderpass.renderpass, &self.framebuffer, self.swapchain.current_extent(), clear);

        for cmd in &pipelines {
            match cmd {
                Commands::Draw(grph_pipeline, bind, vb, dscrps) => {
                    let dscrps = if let Some(d) = dscrps { Some(d.retrieve_descriptor()) } else { None };
                    cmd_buf.draw_pipeline(&grph_pipeline.pipeline,*bind,&vb.buffer_mem,dscrps);
                },
                Commands::DrawIndexed(grph_pipeline, bind, vb, ib, dscrps) => {
                    let dscrps = if let Some(d) = dscrps { Some(d.retrieve_descriptor()) } else { None };
                    cmd_buf.draw_pipeline_indexed(&grph_pipeline.pipeline, *bind, &vb.buffer_mem, &ib.buffer_mem, vk::IndexType::UINT32, dscrps);
                },
                Commands::DrawInstanced => {
                    // TODO
                },
            }
        }

        cmd_buf.end_pass();
        cmd_buf.finish_recording();
        old_cmd_buf.presenter.replace_cmd_buffer(cmd_buf);
    }

    pub fn end(&self, pipelines: Vec<&dyn CleanupVkObj>, renderpasses: Vec<&dyn CleanupVkObj>, presenter: &dyn CleanupVkObj, after_cmd_pool: Vec<&dyn CleanupVkObj>) {
        // this takes &self, so calling this twice will free twice
        let mut objs: Vec<&dyn CleanupVkObj> = Vec::new();
        objs.push(&self.swapchain);
        for o in pipelines {
            objs.push(o);
        }
        for o in renderpasses {
            objs.push(o);
        }
        objs.push(&self.framebuffer);
        objs.push(presenter);
        objs.push(&self.command_pool);
        for o in after_cmd_pool {
            objs.push(o);
        }

        unsafe {
            self.instance.cleanup(&self.device, objs);
        }
    }
}
