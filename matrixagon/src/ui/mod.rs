use crate::datatype as dt;
use crate::world::shader::{UIVert, ui_simpl_vs as vs, ui_simpl_fs as fs};

use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::device::{Device, Queue};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState, AutoCommandBuffer};
use vulkano::command_buffer::pool::standard::StandardCommandPoolAlloc;
use vulkano::pipeline::{GraphicsPipelineAbstract, GraphicsPipeline};
use vulkano::pipeline::viewport::Viewport;
use vulkano::framebuffer::{Subpass, RenderPassAbstract, FramebufferAbstract};
use vulkano::format::Format;
use vulkano::swapchain::Swapchain;

use winit::event::Event;
use winit::window::Window;

use std::sync::Arc;
use std::rc::Rc;
use std::iter;

pub mod layout;
pub mod widgets;

// this acts both like a widget and as a UI context
pub struct App<L: Layout> {
    main_layout: Option<Rc<L>>,

    renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
    vert_shd: vs::Shader,
    frag_shd: fs::Shader,
    vertices: Vec<UIVert>,
    indices: Vec<u32>,
}

impl<L: Layout> App<L> {
    pub fn new(device: Arc<Device>, swapchain: Arc<Swapchain<Window>>) -> Self {  // **: swapchain is used within the macro
        let renderpass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        ).unwrap());

        Self {
            main_layout: None,

            renderpass: renderpass.clone(),
            vert_shd: vs::Shader::load(device.clone()).expect("Failed to load UI Simple vertex shaders module"),
            frag_shd: fs::Shader::load(device.clone()).expect("Failed to load UI Simple fragment shaders module"),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn set_layout(&mut self, lyt: L) -> Rc<L> {
        let lyt = Rc::new(lyt);
        self.main_layout = Some(lyt.clone());
        lyt.clone()
    }

    // renders the graphics pipeline
    pub fn render_gp(&self,
                     device: Arc<Device>,
                     dimension: dt::Dimension<u32>)
        -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<UIVert>()
            .vertex_shader(self.vert_shd.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .viewports(iter::once(Viewport {
                origin: [0.0, 0.0],
                dimensions: dimension.into(),
                depth_range: 0.0 .. 1.0,
            }))
            .fragment_shader(self.frag_shd.main_entry_point(), ())
            .cull_mode_front()  // face culling for optimization
            .alpha_to_coverage_enabled()
            .depth_stencil_simple_depth()
            .render_pass(Subpass::from(self.renderpass.clone(), 0).unwrap())
            .build(device.clone()).unwrap()
        )
    }

    // renders/builds the command buffer
    pub fn render_cp(&self,
                     device: Arc<Device>,
                     queue: Arc<Queue>,
                     gp: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
                     fb: Arc<dyn FramebufferAbstract + Send + Sync>)
        -> AutoCommandBuffer<StandardCommandPoolAlloc> {
        // a contextual info style paradigm thus there will be mutability
        let mut ctx = Context::new();

        if let Some(lyt) = &self.main_layout {
            lyt.clone().render(&mut ctx);
        } else {
            // println!("The UI central app layout has not been set yet");
        }

        let (vbo, ibo) = ctx.flush(device.clone());

        let mut cmd_builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap();

        cmd_builder
            .begin_render_pass(fb.clone(), false, vec![[0.1, 0.3, 1.0, 1.0].into(), 1f32.into()]).unwrap()
            .draw_indexed(gp.clone(), &DynamicState::none(), vec!(vbo.clone()), ibo.clone(), (), ()).unwrap()
            .end_render_pass().unwrap();

        cmd_builder.build().unwrap()
    }

    pub fn update() {

    }
}

// a place where the widgets/layouts can render onto the screen
pub struct Context {
    vertices: Vec<UIVert>,
    indices: Vec<u32>,
}

impl Context {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn flush(self, device: Arc<Device>) -> (Arc<CpuAccessibleBuffer<[UIVert]>>, Arc<CpuAccessibleBuffer<[u32]>>) {
        (
            CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::vertex_buffer(),
                                           false, self.vertices.into_iter()).unwrap(),
            CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::index_buffer(),
                                           false, self.indices.into_iter()).unwrap()
        )
    }

    // add each discrete square
    fn add_square(&mut self) {

    }

    // TODO:
    // an optimized version of rendering each characters instead of each square
    fn add_char(&mut self) {

    }
}


pub enum ObjType<W: Widget + ?Sized + 'static, L: Layout + ?Sized + 'static> {
    Widget(Box<W>),
   Layout(Box<L>),
}

pub trait Widget {
    fn update(&mut self, e: &Event<()>); // updates the widget with states; &mut self
    fn render(&self, ctx: &mut Context); // renders the widget states; handles rendering and user input; &self
}

pub trait Layout: Widget {
    fn add_widget(&mut self);
    fn add_layout(&mut self);
    fn remove_layout(&mut self);
    fn remove_widget(&mut self);
}

// TODO
pub trait Transition {

}
