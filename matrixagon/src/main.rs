// TODO: testing new renderer

// #[macro_use]
// extern crate bytemuck;

use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use ash::vk;
use ash::version::{DeviceV1_0};
use ash::extensions::khr;

use std::mem;


/*
==== External Renderer (higher-level abstraction with absolute no ash crate exposure, to use in the game code, which uses game structs to render):

// interface to the graphics API
let interface = InternalRenderer::new(app_name, validation_layers);

// 3D/2D pipeline implementes traits of 3Drender or 2Drender

let pipeline0 = interface.new_pipeline3D(
    vert_fname,
    frag_fname,
    viewport: dimensions,
    polygon_mode,
    cull,
    transperency,
);

let pipeline1 = interface.new_pipeline2D(
    vert_fname,
    frag_fname,
    viewport: dimensions,
    polygon_mode,
    transperency,
)

interface.add_pipeline3D(id, pipeline0);
interface.add_pipeline2D(id, pipeline1);

let cmd = interface.command_data();

event_loop.loop(move || {
    cmd.clear((0,0,0,1));
    cmd.on_outdated_screen(|| {});
    cmd.render3D.world(id, world_data);
    cmd.render3D.entities(id, entity_data);
    cmd.render3D.other(id, vertex_data);
    cmd.render2D.ui(id, ui_data);
    cmd.render2D.other(id, vertex_data);
    cmd.present();
});

==== Internal Renderer (exposes minimal ash::vk, abstraction to only expose most needed stuff (vertex buffering, graphics pipeline)):

// entry within instance
let instance = Instance::new(app_name, debug<bool>, api_version);  // debug includes internal printer and the vulkan validation layer

// find devices that has queue support of
let device = instance.find_devices(required_extensions, Enum::GRAPHICS && Enum::PRESENT).expect("No supported phys devc found");

let swapchain = instance.get_swapchain(device, min_img_count, dimensions);

// just create the renderpass at the start, since there are really onle 2 major renderpasses.
let main_graphics_pipeline = GraphicsPipeline::new::<(VertexType)>(
    (fname, ...),  // vertex
    (fname, ...),  // fragment
    PolygonMode,
    CullMode,
    Alpha,
    DepthBuffering,
)

// command pool will be within the command buffers
let command_buffers = CommandBuffers::new(device)
    .begin_pass(enable_depth<bool>)
    .draw_pipeline(pipeline, )
    .end_pass()
    .build();

event_loop.loop(move || {
    if suboptimal {
        main_graphics_pipeline.recreate_viewport();
        swapchain.recreate_viewport();
    }

    let suboptimal = swapchain.suboptimal();

    device.execute_and_present(command_buffer);

    if loop_end {
        instance.cleanup(device, swapchain, pipelines, command_buffers);
    }
});

 */


use renderer::{InstanceMTXG, CleanupVkObj};
use renderer::pipeline::{GraphicsPipelineMTXG, VertexInfo};
use renderer::command_buffers::{CommandBufferMTXG, CommandPoolMTXG};
use renderer::buffer::{UniformBufferMTXG, create_image_view, create_sampler, ImageMemoryMTXG, BufferMemoryMTXG};
use renderer::descriptors::{DescriptorSetsMTXG, SetBindingMTXG};
use renderer::swapchain::SwapchainMTXG;
use renderer::device::DeviceMTXG;


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

#[derive(Copy, Clone, Debug)]
struct CubeVertex {
    pos: [f32; 3],
    col: [f32; 3],
    crd: [f32; 2],
}

impl VertexInfo for CubeVertex {
    fn impl_binding() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<CubeVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    fn impl_attributes() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {  // pos
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(CubeVertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {  // color
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(CubeVertex, col) as u32,
            },
            vk::VertexInputAttributeDescription {  // txtr coords
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(CubeVertex, crd) as u32,
            },
        ]
    }
}


extern crate nalgebra as na;

use na::{
    Point3,
    Vector3,
    Matrix4,
    Perspective3,
    Isometry3,
    Translation3,
};


type Matrix3D = [[f32; 4]; 4];

pub fn gen_mvp(aspect: f32, fovy: f32, znear: f32, zfar: f32, tx: f32, ty: f32, tz: f32, rx: f32, ry: f32, rz: f32) -> (Matrix3D, Matrix3D, Matrix3D) {
    let sx = rx.sin();
    let cx = rx.cos();
    let sy = ry.sin();
    let cy = ry.cos();
    let sz = rz.sin();
    let cz = rz.cos();

    let rmat = Matrix4::new( // z
                             cz, -sz, 0.0, 0.0,
                             sz,  cz, 0.0, 0.0,
                             0.0, 0.0, 1.0, 0.0,
                             0.0, 0.0, 0.0, 1.0,
    ) * Matrix4::new( // y
                      cy, 0.0,  sy, 0.0,
                      0.0, 1.0, 0.0, 0.0,
                      -sy, 0.0,  cy, 0.0,
                      0.0, 0.0, 0.0, 1.0,
    ) * Matrix4::new( // x
                      1.0, 0.0, 0.0, 0.0,
                      0.0,  cx, -sx, 0.0,
                      0.0,  sx,  cx, 0.0,
                      0.0, 0.0, 0.0, 1.0,
    );

    let proj = Perspective3::new(aspect, fovy, znear, zfar);

    let view = Isometry3::look_at_lh(&Point3::new(0.0, 0.0, 0.0), &Point3::new(0.0, 0.0, -1.0), &Vector3::new(0.0, -1.0, 0.0));

    let crd = Point3::from([tx, ty, tz]).coords.data;
    let model = Translation3::new(crd[0], crd[1], crd[2]).to_homogeneous() * rmat;

    let proj_matrix = proj.as_matrix();
    let view_matrix = view.to_homogeneous();
    let model_matrix = model.try_inverse().unwrap();

    let proj_cooked: &[f32] = proj_matrix.as_slice();
    let view_cooked: &[f32] = view_matrix.as_slice();
    let model_cooked: &[f32] = model_matrix.as_slice();

    let proj_dt;
    let view_dt;
    let model_dt;

    unsafe {
        assert_eq!(proj_cooked.len(), 16);
        assert_eq!(view_cooked.len(), 16);
        assert_eq!(model_cooked.len(), 16);

        proj_dt = *(proj_cooked.as_ptr() as *const Matrix3D);
        view_dt = *(view_cooked.as_ptr() as *const Matrix3D);
        model_dt = *(model_cooked.as_ptr() as *const Matrix3D);
    }

    (proj_dt, view_dt, model_dt)
}

#[derive(Copy, Clone, Debug)]
struct MVPUniform {
    model: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}


fn main() {
    println!("START");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Matrixagon - Rust/Ash")
        .build(&event_loop).unwrap();

    let instance = InstanceMTXG::new(&window, "App Name", true, (1,2,0));

    let device = instance.find_device(vec![khr::Swapchain::name()], true);

    // let mut swapchain = instance.get_swapchain(&device, 2, None);
    let mut swapchain = SwapchainMTXG::new(&instance, &device, 2, true, None);

    let cmd_pool = CommandPoolMTXG::new(&device);

    let renderpass = GraphicsPipelineMTXG::create_renderpass_with_depth(&device, swapchain.image_format().format, device.find_depth_format());

    use std::fs::File;

    let decoder = png::Decoder::new(File::open("./grass_side.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut txtr_buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut txtr_buf).unwrap();

    let vk_txtr_format = match info.color_type {
        png::ColorType::Grayscale => { vk::Format::R8_SRGB },
        png::ColorType::RGB => { vk::Format::R8G8B8_SRGB },
        png::ColorType::Indexed => { vk::Format::R8G8B8_SRGB },  // dont know what is this, defaulting to RGB
        png::ColorType::GrayscaleAlpha => { vk::Format::R8G8_SRGB },  // R=Greyscale,G=Alpha
        png::ColorType::RGBA => { vk::Format::R8G8B8A8_SRGB },
    };

    // let stage_img = CPUAccessibleBufferMTXG::new(&instance,&device,vk::BufferUsageFlags::TRANSFER_SRC,txtr_buf);
    let stage_img = BufferMemoryMTXG::new::<u8>(&instance, &device, txtr_buf.len(), vk::BufferUsageFlags::TRANSFER_SRC,
                                                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
        .fill(txtr_buf);
    let texture_img = ImageMemoryMTXG::new(&instance, &device, info.width, info.height, vk_txtr_format,
                                           vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED, vk::MemoryPropertyFlags::DEVICE_LOCAL);

    let cmd_buf = CommandBufferMTXG::new(&device, &cmd_pool, 1);
    cmd_buf.start_recording(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    cmd_buf.transition_img_layout(&texture_img, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
    cmd_buf.copy_buffer_to_image(&stage_img, &texture_img);
    cmd_buf.transition_img_layout(&texture_img, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    cmd_buf.finish_recording();
    cmd_buf.build_submit_once(&device);

    unsafe { cmd_buf.cleanup(&device) };
    unsafe { stage_img.cleanup(&device) };

    let texture_img_view = create_image_view(&device, texture_img.image, vk::Format::R8G8B8A8_SRGB, vk::ImageAspectFlags::COLOR);

    let texture_sampler = create_sampler(
        &device,
        vk::Filter::NEAREST,vk::Filter::NEAREST,
        vk::SamplerAddressMode::REPEAT,vk::SamplerAddressMode::REPEAT,
        true,4.0);

    let mut uniform_buf_vec = Vec::with_capacity(swapchain.image_count());
    for _ in 0..swapchain.image_count() {
        uniform_buf_vec.push(UniformBufferMTXG::new::<MVPUniform>(&instance, &device));
    }

    let aspect_ratio = swapchain.current_extent().width as f32 / swapchain.current_extent().height as f32;
    // mvp from camera's position
    let mvp = gen_mvp(aspect_ratio, 1.3, 0.1, 100.0, 0.0, 0.0, -10.0, 0.0, 0.0, 0.0);
    let mvp_uniform = MVPUniform {
        model: mvp.0,
        view: mvp.1,
        proj: mvp.2,
    };

    for i in 0..swapchain.image_count() {
        uniform_buf_vec[i].update(&device, mvp_uniform.clone());
    }

    let mvp_binding = SetBindingMTXG::new(0, vk::DescriptorType::UNIFORM_BUFFER, vk::ShaderStageFlags::VERTEX);
    let txtr_binding = SetBindingMTXG::new(1, vk::DescriptorType::COMBINED_IMAGE_SAMPLER, vk::ShaderStageFlags::FRAGMENT);

    // descriptor lasts for the entire lifetime of the pipeline its used by
    let main_dscrp = DescriptorSetsMTXG::new(&device, &[mvp_binding, txtr_binding], swapchain.image_count() as u32);
    main_dscrp.bind_uniform(0, uniform_buf_vec.as_slice(), true);
    main_dscrp.bind_sampler(1, &[texture_img_view], texture_sampler, false);


    let mut main_gp = GraphicsPipelineMTXG::new::<CubeVertex>(
        &device,
        swapchain.current_extent(),
        renderpass,
        Some(&main_dscrp),
        "./vert.spv",
        "./frag.spv",
        vk::PolygonMode::FILL,
        vk::CullModeFlags::NONE,
        false,
        true,
    );

    let mut framebuffer = swapchain.get_framebuffers(&device, renderpass);

    let vertices = vec![
        // face 1
        CubeVertex {pos: [0.0+0.5,0.0-0.5,0.0+0.0], col: [1.0, 0.0, 0.0], crd: [0.0, 0.0]},
        CubeVertex {pos: [0.0-0.5,0.0-0.5,0.0+0.0], col: [0.0, 1.0, 0.0], crd: [0.0, 1.0]},
        CubeVertex {pos: [0.0-0.5,0.0+0.5,0.0+0.0], col: [0.0, 0.0, 1.0], crd: [1.0, 1.0]},
        CubeVertex {pos: [0.0+0.5,0.0+0.5,0.0+0.0], col: [0.0, 1.0, 1.0], crd: [1.0, 0.0]},
        // face 2
        CubeVertex {pos: [0.0+0.5,0.8-0.5,1.0+0.0], col: [1.0, 0.0, 0.0], crd: [0.0, 0.0]},
        CubeVertex {pos: [0.0-0.5,0.8-0.5,1.0+0.0], col: [0.0, 1.0, 0.0], crd: [0.0, 1.0]},
        CubeVertex {pos: [0.0-0.5,0.8+0.5,1.0+0.0], col: [0.0, 0.0, 1.0], crd: [1.0, 1.0]},
        CubeVertex {pos: [0.0+0.5,0.8+0.5,1.0+0.0], col: [0.0, 1.0, 1.0], crd: [1.0, 0.0]},
    ];

    let indices: Vec<u32> = vec![
        0, 1, 2,
        0, 2, 3,

        4, 5, 6,
        4, 6, 7,
    ];

    let vert_buf = BufferMemoryMTXG::new::<CubeVertex>(&instance, &device, vertices.len(), vk::BufferUsageFlags::VERTEX_BUFFER,
                                                       vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
        .fill(vertices);
    let indx_buf = BufferMemoryMTXG::new::<u32>(&instance, &device, indices.len(), vk::BufferUsageFlags::INDEX_BUFFER,
                                                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT)
        .fill(indices);

    // let vert_buf = CPUAccessibleBufferMTXG::new(&instance, &device, vk::BufferUsageFlags::VERTEX_BUFFER, vertices);
    // let indx_buf = CPUAccessibleBufferMTXG::new(&instance, &device, vk::BufferUsageFlags::INDEX_BUFFER, indices);

    let cmd_buf = CommandBufferMTXG::new(&device, &cmd_pool, framebuffer.len() as u32);
    cmd_buf.start_recording(vk::CommandBufferUsageFlags::empty());
    cmd_buf.begin_depth_pass(renderpass, &framebuffer, swapchain.current_extent(), (0.0, 0.0, 0.0));
    cmd_buf.draw_pipeline_indexed(&main_gp, 0, &vert_buf, &indx_buf, vk::IndexType::UINT32, Some(&main_dscrp));
    cmd_buf.end_pass();
    cmd_buf.finish_recording();
    let mut present = cmd_buf.build(&swapchain);  // first creation, so recreation is false


    // // creation of depth image
    // let format = device.find_depth_format();
    // let vk::Extent2D { width, height } = swapchain.current_extent();
    // let depth_image = ImageMemoryMTXG::new(&instance, &device, width, height, format,
    //                                        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, vk::MemoryPropertyFlags::DEVICE_LOCAL);
    // let depth_image_view = create_image_view(&device, depth_image.image, format, vk::ImageAspectFlags::DEPTH);

    // println!("vvvvvvvvvvvvvvvvvvvvvvv [@#]");
    // println!("^^^^^^^^^^^^^^^^^^^^^^^ [@~]");

    use winit::event::{Event, WindowEvent, VirtualKeyCode as K, ElementState, KeyboardInput};
    use winit::event_loop::ControlFlow;

    let mut window_resized = false;

    let mut t = 0.0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {window_id, event} => {
                match event {
                    WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit },
                    WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                        match input {
                            KeyboardInput { virtual_keycode: key, state: ElementState::Pressed, .. } => {
                                if let Some(k) = key {
                                    match k {
                                        K::Escape => { *control_flow = ControlFlow::Exit },
                                        _ => {},
                                    }
                                }
                            }
                            _ => {},
                        }
                    },
                    WindowEvent::Resized(size) => {
                        // println!("{:?}", size);
                        window_resized = true;
                    },
                    _ => {},
                }
            },
            // Called when the windows contents are invalidated (e.g.: dimensions changed)
            Event::RedrawRequested(window_id) => {
                unsafe {
                    if present.check_suboptimal_once() || window_resized {
                        // note: renderpass does not need to be recreate because its independent from window sizes
                        window_resized = false;

                        device.wait_idle();

                        // swapchain recreation / object cleanup
                        swapchain = swapchain.clone().recreate_swapchain(&instance, &device, vec![
                            &swapchain,
                            &main_gp,
                            &framebuffer,
                            &present,
                        ]);

                        // retrieving recreated object
                        framebuffer = swapchain.get_framebuffers(&device, renderpass);

                        main_gp.recreate_pipeline(&device, swapchain.current_extent());

                        let cmd_buf = CommandBufferMTXG::new(&device, &cmd_pool, framebuffer.len() as u32);
                        cmd_buf.start_recording(vk::CommandBufferUsageFlags::empty());
                        cmd_buf.begin_depth_pass(renderpass, &framebuffer, swapchain.current_extent(), (0.0, 0.0, 0.0));
                        cmd_buf.draw_pipeline_indexed(&main_gp, 0, &vert_buf, &indx_buf, vk::IndexType::UINT32, Some(&main_dscrp));
                        cmd_buf.end_pass();
                        cmd_buf.finish_recording();
                        present.replace_cmd_buffer(cmd_buf);

                        // uniform buffer update
                        // uniform buffer only needs to be recreated when swapchain has different image count
                        if swapchain.image_count() != uniform_buf_vec.len() {
                            for i in 0..uniform_buf_vec.len() {
                                uniform_buf_vec[i].cleanup(&device);
                                uniform_buf_vec.push(UniformBufferMTXG::new::<MVPUniform>(&instance, &device));
                            }
                            uniform_buf_vec.clear();
                            for _ in 0..swapchain.image_count() {
                                uniform_buf_vec.push(UniformBufferMTXG::new::<MVPUniform>(&instance, &device));
                            }

                            let aspect_ratio = swapchain.current_extent().width as f32 / swapchain.current_extent().height as f32;

                            // mvp from camera's position
                            let mvp = gen_mvp(aspect_ratio, 1.3, 0.1, 100.0, 0.0, 0.0, -10.0, 0.0, 0.0, t);
                            let mvp_uniform = MVPUniform {
                                model: mvp.0,
                                view: mvp.1,
                                proj: mvp.2,
                            };

                            for i in 0..swapchain.image_count() {
                                uniform_buf_vec[i].update(&device, mvp_uniform.clone());
                            }
                        }
                    }
                }
            },
            // Calls after RedrawRequested, else calls after MainEventsCleared
            Event::RedrawEventsCleared => {
                t += 0.001;

                let aspect_ratio = swapchain.current_extent().width as f32 / swapchain.current_extent().height as f32;

                // mvp from camera's position
                let mvp = gen_mvp(aspect_ratio, 1.3, 0.1, 100.0, 0.0, 0.0, -1.0, 0.0, 0.0, t);
                let mvp_uniform = MVPUniform {
                    model: mvp.0,
                    view: mvp.1,
                    proj: mvp.2,
                };

                for i in 0..swapchain.image_count() {
                    uniform_buf_vec[i].update(&device, mvp_uniform.clone());
                }

                present.submit_and_present(&swapchain);
            },
            Event::LoopDestroyed => {
                unsafe {
                    instance.clone().cleanup(&device, vec![
                        &swapchain, &main_gp, &renderpass, &framebuffer, &present, &cmd_pool, &texture_img, &texture_img_view, &texture_sampler, &uniform_buf_vec, &main_dscrp, &vert_buf, &indx_buf
                    ]);
                };
            },
            _ => {},
        }
    });
}

// TODO ==== BEGIN ORIGINAL CODE ====

// extern crate nalgebra as na;
//
// use vulkano::{
//     instance::{Instance, PhysicalDevice},
//     device::{Device},
// };
//
// use vulkano_win::VkSurfaceBuild;
// use winit::event_loop::EventLoop;
// use winit::window::WindowBuilder;
// use winit::dpi::{Position, PhysicalPosition, PhysicalSize};
//
// use crate::app::MainApp;
// use crate::datatype::{CamDirection, Dimension};
// use crate::event::{EventDispatcher, EventName};
// use crate::event::types;
//
// #[macro_use]
// mod event;
// mod threadpool;
// mod ui;
// mod world;
//
// mod app;
// mod datatype;
// mod math;
// mod util;
//
//
// // #[macro_export]
// // macro_rules! println {
// //     ($args:literal $(,)?) => {{
// //         std::print!("{:?} {:?} {:?} :", file!(), line!(), column!());
// //         std::println!($args);
// //     }};
// //     ($args:literal, $($param:tt)*) => {{
// //         std::print!("{:?} {:?} {:?} :", file!(), line!(), column!());
// //         std::println!($args, $($param)*);
// //     }};
// // }
//
//
// fn main() {
//     // main setup
//
//     println!("PROGRAM - BEGIN INITIALIZATION");
//     let instance= {
//         let extensions = vulkano_win::required_extensions();
//         Instance::new(None, &extensions, None).expect("failed to create instance")
//     };
//
//     let event_loop = EventLoop::new();
//     let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();
//
//     let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
//     let queue_family = physical.queue_families()
//         .find(|&q| q.supports_graphics())
//         .expect("couldn't find a graphical queue family");
//     let (device, mut queues) = {
//         let device_ext = vulkano::device::DeviceExtensions {
//             khr_swapchain: true,
//             .. vulkano::device::DeviceExtensions::none()
//         };
//         Device::new(physical, physical.supported_features(), &device_ext,
//                     [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
//     };
//     let queue = queues.next().unwrap();
//
//     let mut dimensions = surface.window().inner_size().into();
//
//     // setting up for the program
//     println!("PROGRAM - BEGIN MAIN PROGRAM");
//
//     let evd = EventDispatcher::new(types::global_enmtyp());
//
//     // let mut textr: Texture<'static> = Texture::new(queue.clone());
//     let mut app = MainApp::new(
//         device.clone(), queue.clone(), evd.clone(),
//         surface.clone(), physical, dimensions
//     );
//
//     use winit::event_loop::ControlFlow;
//     use winit::event::{Event, WindowEvent, DeviceEvent, VirtualKeyCode as K, KeyboardInput, ElementState};
//
//     let mut pressed: Vec<K> = Vec::new();
//     let mut cmd_mode = false;
//     let mut focused = true;
//     let mut minimized = false;
//
//     event_loop.run(move |event, _, control_flow| {
//         dimensions = surface.window().inner_size().into();
//         // println!("D {:?}", dimensions);
//
//         match event {
//             Event::WindowEvent { event, .. } => {
//                 match event {
//                     WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit },
//                     WindowEvent::Resized(size) => {
//                         // when screen gets resized
//
//                         let PhysicalSize { width, height } = size;
//                         if width == 0 && height == 0 {
//                             println!("Screen minimized");
//                             minimized = true;
//                         } else if minimized {
//                             println!("Screen un-minimized");
//                             minimized = false;
//                         } else {
//                             evd.clone().emit(EventName("MeshEvent/UpdateDimensions"), event_data![Dimension::new(height, width)]);
//                         }
//                     },
//                     WindowEvent::KeyboardInput { input, .. } => {
//                         if !minimized {
//                             match input {
//                                 KeyboardInput { virtual_keycode: key, state: ElementState::Pressed, .. } => {
//                                     if let Some(k) = key {
//                                         match k {
//                                             K::Escape => { *control_flow = ControlFlow::Exit },
//                                             K::T => { cmd_mode = !cmd_mode },
//                                             K::A => { if !pressed.contains(&K::A) { pressed.push(K::A); } },
//                                             K::D => { if !pressed.contains(&K::D) { pressed.push(K::D); } },
//                                             K::W => { if !pressed.contains(&K::W) { pressed.push(K::W); } },
//                                             K::S => { if !pressed.contains(&K::D) { pressed.push(K::S); } },
//                                             K::LShift => { if !pressed.contains(&K::LShift) { pressed.push(K::LShift); } },
//                                             K::Space => { if !pressed.contains(&K::Space) { pressed.push(K::Space); } },
//                                             K::LControl => {
//                                                 // TODO: Use the event system after App Event & World Event is added
//                                                 app.world.player.camera.trans_speed = 0.25;
//                                             },
//                                             _ => {}
//                                         }
//                                     } else {
//                                         println!("An invalid key registered. Please make sure you are only returning ASCII character");
//                                     }
//                                 },
//                                 KeyboardInput { virtual_keycode: key, state: ElementState::Released, .. } => {
//                                     if let Some(key) = key {
//                                         match key {
//                                             K::A => { if pressed.contains(&K::A) { pressed.retain(|i| i != &K::A); } },
//                                             K::D => { if pressed.contains(&K::D) { pressed.retain(|i| i != &K::D); } },
//                                             K::W => { if pressed.contains(&K::W) { pressed.retain(|i| i != &K::W); } },
//                                             K::S => { if pressed.contains(&K::S) { pressed.retain(|i| i != &K::S); } },
//                                             K::LShift => { if pressed.contains(&K::LShift) { pressed.retain(|i| i != &K::LShift); } },
//                                             K::Space => { if pressed.contains(&K::Space) { pressed.retain(|i| i != &K::Space); } },
//                                             K::LControl => {
//                                                 app.world.player.camera.trans_speed = 0.1;
//                                             },
//                                             _ => {}
//                                         }
//                                     }
//                                 }
//                             }
//                         }
//                     },
//                     WindowEvent::MouseInput { state, button, .. } => {
//                         if !cmd_mode {
//                             // TODO: will do again after the "do_block" event moved
//                             // if button == MouseButton::Left && state == ElementState::Pressed {
//                             //     app.world.do_block(true, false);
//                             // } else if button == MouseButton::Right && state == ElementState::Pressed {
//                             //     app.world.do_block(false, true);
//                             // }
//                         }
//                     },
//                     WindowEvent::Focused(win_focused) => {
//                         focused = win_focused;
//                     },
//                     _ => {}
//                 }
//             },
//             Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
//                 if !cmd_mode && focused {
//                     // println!("ROT X: {}, Y: {}", delta.1, delta.0);
//
//                     app.world.player.camera.rotate(delta.1 as f32, delta.0 as f32, 0.0);
//
//                     // app.world.player.camera.rotate(delta.1 as f32, 0.0, 0.0);
//                     // app.world.player.camera.rotate(0.0, delta.0 as f32, 0.0);
//
//                     let res = surface.window().set_cursor_position(
//                         Position::Physical(PhysicalPosition { x: dimensions.width as i32 / 2, y: dimensions.height as i32 / 2 })
//                     );
//                     if let Err(e) = res {
//                         println!("Setting mouse position of the windows caused an error of {}", e);
//                     }
//                 }
//             },
//             // this calls last after all the event finishes emitting
//             // and only calls once, which is great for updating mutable variables since it'll be uniform
//             Event::MainEventsCleared => {
//                 let mut directions = Vec::new();
//
//                 if pressed.contains(&K::A) { directions.push(CamDirection::Leftward) }
//                 if pressed.contains(&K::D) { directions.push(CamDirection::Rightward) }
//                 if pressed.contains(&K::W) { directions.push(CamDirection::Forward) }
//                 if pressed.contains(&K::S) { directions.push(CamDirection::Backward) }
//                 if pressed.contains(&K::LShift) { directions.push(CamDirection::Downward) }
//                 if pressed.contains(&K::Space) { directions.push(CamDirection::Upward) }
//
//                 app.world.player.camera.travel(directions);
//
//                 // event dispatcher to event_swap() after all the events has been finished
//                 evd.clone().event_swap();
//             },
//             Event::RedrawEventsCleared => {
//                 if !minimized {
//                     app.update(dimensions);
//                 }
//             },
//             _ => {},
//         };
//     });
// }
