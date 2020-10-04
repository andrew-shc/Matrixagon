use crate::datatype as dt;
use crate::ui;
use crate::ui::{App, Layout};
use crate::ui::layout as lyt;
use crate::datatype::Dimension;
use crate::world::World;
use crate::event::EventDispatcher;

use vulkano::device::{Device, Queue};
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::swapchain::{Swapchain, SurfaceTransform, PresentMode, ColorSpace, FullscreenExclusive, SwapchainCreationError, AcquireError, Surface};
use vulkano::swapchain;
use vulkano::format::Format;
use vulkano::image::{SwapchainImage, AttachmentImage};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract, Framebuffer};
use vulkano::sync;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::GraphicsPipelineAbstract;

use winit::window::Window;

use std::sync::Arc;
use std::any::{TypeId, Any};
use std::rc::Rc;


pub struct MainApp<L: Layout> {
    device: Arc<Device>,
    queue: Arc<Queue>,
    event: Rc<EventDispatcher>,

    prev_frame: Option<Box<dyn GpuFuture>>,  // previous frame
    swapchain: Arc<Swapchain<Window>>,  // swapchain is used for "swapping" the chain of images rendered from the GPU
    images: Vec<Arc<SwapchainImage<Window>>>,  // swapchain images
    framebuffer: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,  // framebuffer is the chain of images
    renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,  // a sort of like configuration on how to create an image from multiple layers rendered

    recreate: bool, // recreates old swapchain (e.g. window size changed)

    pub ui: App<L>,
    ui_gp: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,

    pub world: World,
}

impl MainApp<lyt::StackLayout> {
    pub fn new(device: Arc<Device>,
               queue: Arc<Queue>,
               evd: Rc<EventDispatcher>,
               surface: Arc<Surface<Window>>,
               physical: PhysicalDevice,
               dimensions: Dimension<u32>,
    ) -> Self {
        println!("APP - INITIALIZED");

        let caps = surface.clone().capabilities(physical)
            .expect("failed to get surface capabilities");

        let dimn = caps.current_extent.unwrap_or(dimensions.into());
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let (swapchain, images) = Swapchain::new(
            device.clone(), surface.clone(), caps.min_image_count, format, dimn, 1,
            caps.supported_usage_flags, &queue, SurfaceTransform::Identity, alpha, PresentMode::Fifo,
            FullscreenExclusive::Default,  true, ColorSpace::SrgbNonLinear)
            .expect("failed to create swapchain");

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

        let future = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);

        let app = App::new(device.clone(), swapchain.clone());
        let app_gp = app.render_gp(device.clone(), dimensions);
        // TODO: a separate struct to write pure ui code for the app?

        let mut world = World::new(device.clone(), queue.clone(), evd.clone(), renderpass.clone(), dimensions);

        let future = Some(world.bind_texture(future.unwrap()));

        Self {
            device: device.clone(),
            queue: queue.clone(),
            event: evd.clone(),

            prev_frame: future,
            swapchain: swapchain.clone(),
            images: images.clone(),
            framebuffer: Self::frames(device.clone(), &images, renderpass.clone()),
            renderpass: renderpass.clone(),

            recreate: false,

            ui: app,
            ui_gp: app_gp,

            world: world,
        }
    }

    // updates the app; the app also should automatically renders the screen
    pub fn update(&mut self, dimensions: dt::Dimension<u32>) {
        // println!("APP - UPDATE");

        // cleans the previous buffer
        self.prev_frame.as_mut().unwrap().cleanup_finished();

        if self.recreate {
            println!("Frame recreated: {:?}", dimensions);

            let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimensions(dimensions.into()) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e)
            };
            self.swapchain = new_swapchain;
            // recreates the framebuffer after recreating swapchain
            self.framebuffer = Self::frames(self.device.clone(), &new_images, self.renderpass.clone());

            self.ui_gp = self.ui.render_gp(self.device.clone(), dimensions);
        }

        let (image_num, suboptimal, acquire_future) = match swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                self.recreate = true;
                return;
            },
            Err(e) => panic!("Failed to acquire next image: {:?}", e)
        };

        if suboptimal { self.recreate = true; }

        self.world.update(dimensions, self.renderpass.clone(),
                          self.framebuffer[image_num].clone(), self.recreate
        );

        let world_cp = self.world.render(
            self.device.clone(),
            self.queue.clone(),
            self.framebuffer[image_num].clone(),
            dimensions,
        );

        let ui_cmd_buf = self.ui.render_cp(self.device.clone(), self.queue.clone(),
                                           self.ui_gp.clone(), self.framebuffer[image_num].clone(), );

        let future = self.prev_frame.take().unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), world_cp).unwrap()
            // .then_execute(self.queue.clone(), ui_cmd_buf).unwrap()
            // submits present command to the GPU to the end of queue
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(fut) => {
                self.prev_frame = Some(Box::new(fut) as Box<_>);
            },
            Err(FlushError::OutOfDate) => {
                self.recreate = true;
                self.prev_frame = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.prev_frame = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
            }
        }

        self.recreate = false;
    }

    // generates a new frames for framebuffer
    fn frames(device: Arc<Device>,
              images: &Vec<Arc<SwapchainImage<Window>>>,
              render_pass: Arc<dyn RenderPassAbstract + Send + Sync>
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let dimensions = images[0].dimensions();

        let depth_buffer = AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();

        images.iter().map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone()).unwrap()
                    .add(depth_buffer.clone()).unwrap()
                    .build().unwrap()
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>()
    }
}