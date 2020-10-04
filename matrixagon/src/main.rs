extern crate nalgebra as na;

use vulkano::{
    instance::{Instance, PhysicalDevice},
    device::{Device},
};

use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit::dpi::{Position, PhysicalPosition, PhysicalSize};

use crate::app::MainApp;
use crate::datatype::CamDirection;
use crate::event::EventDispatcher;
use crate::event::types;

#[macro_use]
mod event;
mod ui;
mod world;

mod app;
mod datatype;
mod math;
mod util;


fn main() {
    // main setup

    println!("PROGRAM - BEGIN INITIALIZATION");
    let instance= {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("failed to create instance")
    };

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();

    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");
    let (device, mut queues) = {
        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };
        Device::new(physical, physical.supported_features(), &device_ext,
                    [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };
    let queue = queues.next().unwrap();

    let mut dimensions = surface.window().inner_size().into();

    // setting up for the program
    println!("PROGRAM - BEGIN MAIN PROGRAM");

    let evd = EventDispatcher::new(types::global_enmtyp());

    // let mut textr: Texture<'static> = Texture::new(queue.clone());
    let mut app = MainApp::new(
        device.clone(), queue.clone(), evd.clone(),
        surface.clone(), physical, dimensions
    );

    use winit::event_loop::ControlFlow;
    use winit::event::{Event, WindowEvent, DeviceEvent, VirtualKeyCode as K, KeyboardInput, ElementState};

    let mut pressed: Vec<K> = Vec::new();
    let mut cmd_mode = false;
    let mut focused = true;
    let mut minimized = false;

    event_loop.run(move |event, _, control_flow| {
        dimensions = surface.window().inner_size().into();
        // println!("D {:?}", dimensions);

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit },
                    WindowEvent::Resized(size) => {
                        let PhysicalSize { width, height } = size;
                        if width == 0 && height == 0 {
                            println!("Screen minimized");
                            minimized = true;
                        } else {
                            if minimized {
                                println!("Screen un-minimized");
                            }
                            minimized = false;
                        }
                    },
                    WindowEvent::KeyboardInput { input, .. } => {
                        if !minimized {
                            match input {
                                KeyboardInput { virtual_keycode: key, state: ElementState::Pressed, .. } => {
                                    if let Some(k) = key {
                                        match k {
                                            K::Escape => { *control_flow = ControlFlow::Exit },
                                            K::T => { cmd_mode = !cmd_mode },
                                            K::A => { if !pressed.contains(&K::A) { pressed.push(K::A); } },
                                            K::D => { if !pressed.contains(&K::D) { pressed.push(K::D); } },
                                            K::W => { if !pressed.contains(&K::W) { pressed.push(K::W); } },
                                            K::S => { if !pressed.contains(&K::D) { pressed.push(K::S); } },
                                            K::LShift => { if !pressed.contains(&K::LShift) { pressed.push(K::LShift); } },
                                            K::Space => { if !pressed.contains(&K::Space) { pressed.push(K::Space); } },
                                            K::LControl => {
                                                // TODO: Use the event system after App Event & World Event is added
                                                app.world.player.camera.trans_speed = 0.25;
                                            },
                                            _ => {}
                                        }
                                    } else {
                                        println!("An invalid key registered. Please make sure you are only returning ASCII character");
                                    }
                                },
                                KeyboardInput { virtual_keycode: key, state: ElementState::Released, .. } => {
                                    if let Some(key) = key {
                                        match key {
                                            K::A => { if pressed.contains(&K::A) { pressed.retain(|i| i != &K::A); } },
                                            K::D => { if pressed.contains(&K::D) { pressed.retain(|i| i != &K::D); } },
                                            K::W => { if pressed.contains(&K::W) { pressed.retain(|i| i != &K::W); } },
                                            K::S => { if pressed.contains(&K::S) { pressed.retain(|i| i != &K::S); } },
                                            K::LShift => { if pressed.contains(&K::LShift) { pressed.retain(|i| i != &K::LShift); } },
                                            K::Space => { if pressed.contains(&K::Space) { pressed.retain(|i| i != &K::Space); } },
                                            K::LControl => {
                                                app.world.player.camera.trans_speed = 0.1;
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    },
                    WindowEvent::MouseInput { state, button, .. } => {
                        if !cmd_mode {
                            // TODO: will do again after the "do_block" event moved
                            // if button == MouseButton::Left && state == ElementState::Pressed {
                            //     app.world.do_block(true, false);
                            // } else if button == MouseButton::Right && state == ElementState::Pressed {
                            //     app.world.do_block(false, true);
                            // }
                        }
                    },
                    WindowEvent::Focused(win_focused) => {
                        focused = win_focused;
                    },
                    _ => {}
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                if !cmd_mode && focused {
                    // println!("ROT X: {}, Y: {}", delta.1, delta.0);

                    app.world.player.camera.rotate(delta.1 as f32, delta.0 as f32, 0.0);

                    // app.world.player.camera.rotate(delta.1 as f32, 0.0, 0.0);
                    // app.world.player.camera.rotate(0.0, delta.0 as f32, 0.0);

                    let res = surface.window().set_cursor_position(
                        Position::Physical(PhysicalPosition { x: dimensions.width as i32 / 2, y: dimensions.height as i32 / 2 })
                    );
                    if let Err(e) = res {
                        println!("Setting mouse position of the windows caused an error of {}", e);
                    }
                }
            },
            // this calls last after all the event finishes emitting
            // and only calls once, which is great for updating mutable variables since it'll be uniform
            Event::MainEventsCleared => {
                let mut directions = Vec::new();

                if pressed.contains(&K::A) { directions.push(CamDirection::Leftward) }
                if pressed.contains(&K::D) { directions.push(CamDirection::Rightward) }
                if pressed.contains(&K::W) { directions.push(CamDirection::Forward) }
                if pressed.contains(&K::S) { directions.push(CamDirection::Backward) }
                if pressed.contains(&K::LShift) { directions.push(CamDirection::Downward) }
                if pressed.contains(&K::Space) { directions.push(CamDirection::Upward) }

                app.world.player.camera.travel(directions);

                // event dispatcher to event_swap() after all the events has been finished
                evd.clone().event_swap();
            },
            Event::RedrawEventsCleared => {
                if !minimized {
                    app.update(dimensions);
                }
            },
            _ => {},
        };
    });
}