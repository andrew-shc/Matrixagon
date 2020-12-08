extern crate nalgebra as na;

use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use ash::vk;

use std::mem;
use std::fs::File;

use renderer::CleanupVkObj;

use na::{
    Point3,
    Vector3,
    Matrix4,
    Perspective3,
    Isometry3,
    Translation3,
};

use crate::render::buffers::{SamplerFilter, SamplerAddressMode};
use crate::render::{TextureFormat, ShaderStages, Commands, DataType, VertexRates};

#[macro_use] mod render;


#[derive(Copy, Clone, Debug)]
struct CubeVertex {
    pos: [f32; 3],
    col: [f32; 3],
    crd: [f32; 2],
}

impl_vertex!(CubeVertex, pos => DataType::Vec3, col => DataType::Vec3, crd => DataType::Vec2,);


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
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Matrixagon - Rust/Ash")
        .build(&event_loop).unwrap();

    let (mut gfx, device) = render::Renderer::new(&window, "App Name", true);

    // TEXTURE BUFFER
    let decoder = png::Decoder::new(File::open("./grass_side.png").unwrap());
    let (info, mut reader) = decoder.read_info().unwrap();
    let mut txtr_buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut txtr_buf).unwrap();

    let mut sampler = gfx.create_sampler();
    sampler.filter(SamplerFilter::Nearest, SamplerFilter::Nearest);
    sampler.address_mode(SamplerAddressMode::ClampToBorder, SamplerAddressMode::ClampToBorder);
    sampler.anisotropy(4.0);
    sampler.build(&device);

    let image = gfx.create_sampled_image(txtr_buf, info.width, info.height,TextureFormat::RGBA, sampler.clone());

    // MVP BUFFER
    let aspect_ratio = gfx.current_extent().width as f32 / gfx.current_extent().height as f32;
    // mvp from camera's position
    let mvp = gen_mvp(aspect_ratio, 1.3, 0.1, 100.0, 0.0, 0.0, -10.0, 0.0, 0.0, 0.0);
    let mvp_uniform = MVPUniform {
        model: mvp.0,
        view: mvp.1,
        proj: mvp.2,
    };

    let mut mvp_ub = gfx.create_uniform_buffer::<MVPUniform>();
    mvp_ub.update(&device, mvp_uniform);

    let mut main_descriptors = gfx.create_descriptors();
    main_descriptors.bind_uniform(0,ShaderStages::Vertex,mvp_ub.clone());
    main_descriptors.bind_sampler(1, ShaderStages::Fragment, image.clone());
    main_descriptors.build(&device);

    let main_renderpass = gfx.get_renderpass();

    let mut main_gp = gfx.create_graphics_pipeline::<CubeVertex>(
        &main_renderpass,
        &main_descriptors,
        "./vert.spv",
        "./frag.spv",
        VertexRates::Vertex,
        true,false,
    );

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

    let vert_buf = gfx.create_vertex_buffer(vertices);
    let indx_buf = gfx.create_index_buffer(indices);

    let mut cmd_buf = gfx.create_command_buffers(
        main_renderpass,
        (0.0, 0.0, 0.5),
        vec![
            Commands::DrawIndexed(main_gp.clone(), 0, vert_buf.clone(), indx_buf.clone(), Some(main_descriptors.clone())),
        ],
    );


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
                if cmd_buf.suboptimal() || window_resized {
                    window_resized = false;

                    gfx.recreate(vec![&mut main_gp], vec![&cmd_buf, &main_descriptors]);

                    unsafe { mvp_ub.cleanup(&device); }
                    mvp_ub = gfx.create_uniform_buffer::<MVPUniform>();

                    main_descriptors = gfx.create_descriptors();
                    main_descriptors.bind_uniform(0,ShaderStages::Vertex,mvp_ub.clone());
                    main_descriptors.bind_sampler(1, ShaderStages::Fragment, image.clone());
                    main_descriptors.build(&device);

                    gfx.replace_command_buffers(&mut cmd_buf,
                                                main_renderpass,
                                                (0.0, 0.0, 0.5),
                                                vec![
                                                    Commands::DrawIndexed(main_gp.clone(), 0, vert_buf.clone(), indx_buf.clone(), Some(main_descriptors.clone())),
                                                ]
                    );

                    let aspect_ratio = gfx.current_extent().width as f32 / gfx.current_extent().height as f32;

                    // mvp from camera's position
                    let mvp = gen_mvp(aspect_ratio, 1.3, 0.1, 100.0, 0.0, 0.0, -1.0, 0.0, 0.0, t);
                    let mvp_uniform = MVPUniform {
                        model: mvp.0,
                        view: mvp.1,
                        proj: mvp.2,
                    };

                    mvp_ub.update(&device, mvp_uniform);
                }
            },
            // Calls after RedrawRequested, else calls after MainEventsCleared
            Event::RedrawEventsCleared => {
                t += 0.001;

                let aspect_ratio = gfx.current_extent().width as f32 / gfx.current_extent().height as f32;

                // mvp from camera's position
                let mvp = gen_mvp(aspect_ratio, 1.3, 0.1, 100.0, 0.0, 0.0, -1.0, 0.0, 0.0, t);
                let mvp_uniform = MVPUniform {
                    model: mvp.0,
                    view: mvp.1,
                    proj: mvp.2,
                };

                mvp_ub.update(&device, mvp_uniform);

                cmd_buf.present(&gfx);
            },
            Event::LoopDestroyed => {
                gfx.end(vec![&main_gp], vec![&main_renderpass], &cmd_buf, vec![&image, &sampler, &mvp_ub, &main_descriptors, &vert_buf, &indx_buf]);
            },
            _ => {},
        }
    });
}

// TODO ==== BEGIN ORIGINAL CODE ====
//
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
// #[macro_use] mod event;
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
