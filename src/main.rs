

use vulkano::{
    instance::{Instance, PhysicalDevice},
    device::{Device},
};

use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use crate::app::MainApp;
use crate::datatype::CamDirection;
use cgmath::Deg;
use winit::dpi::{Position, PhysicalPosition};

mod ui;
mod mesh;
mod terrain;
mod player;
mod block;

mod app;
mod datatype;
mod world;
mod shader;
mod chunk;
mod texture;


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

    // let mut textr: Texture<'static> = Texture::new(queue.clone());
    let mut app = MainApp::new(
        device.clone(), queue.clone(),
        surface.clone(), physical, dimensions);

    use winit::event_loop::ControlFlow;
    use winit::event::{Event, WindowEvent, DeviceEvent, VirtualKeyCode as K, KeyboardInput, ElementState};

    let mut pressed: Vec<K> = Vec::new();
    let mut cmd_mode = false;

    event_loop.run( move |event, _, control_flow| {
        dimensions = surface.window().inner_size().into();

        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {*control_flow = ControlFlow::Exit},
                    WindowEvent::KeyboardInput { input, ..} => {
                        match input {
                            KeyboardInput { virtual_keycode: key, state: ElementState::Pressed, ..} => {
                                match key.unwrap() {
                                    K::Escape => {*control_flow = ControlFlow::Exit},
                                    K::T => {cmd_mode = !cmd_mode},
                                    K::A => { if !pressed.contains(&K::A) {pressed.push(K::A);} },
                                    K::D => { if !pressed.contains(&K::D) {pressed.push(K::D);} },
                                    K::W => { if !pressed.contains(&K::W) {pressed.push(K::W);} },
                                    K::S => { if !pressed.contains(&K::D) {pressed.push(K::S);} },
                                    K::LShift => { if !pressed.contains(&K::LShift) {pressed.push(K::LShift);} },
                                    K::Space =>  { if !pressed.contains(&K::Space) {pressed.push(K::Space);} },
                                    _ => {}
                                }
                            },
                            KeyboardInput { virtual_keycode: key, state: ElementState::Released, ..} => {
                                match key.unwrap() {
                                    K::A => { if pressed.contains(&K::A) {pressed.retain(|i| i != &K::A);} },
                                    K::D => { if pressed.contains(&K::D) {pressed.retain(|i| i != &K::D);} },
                                    K::W => { if pressed.contains(&K::W) {pressed.retain(|i| i != &K::W);} },
                                    K::S => { if pressed.contains(&K::S) {pressed.retain(|i| i != &K::S);} },
                                    K::LShift => { if pressed.contains(&K::LShift) {pressed.retain(|i| i != &K::LShift);} },
                                    K::Space => { if pressed.contains(&K::Space) {pressed.retain(|i| i != &K::Space);} },
                                    _ => {}
                                }
                            }
                        }
                    },
                    _ => {}
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                if !cmd_mode {
                    println!("ROT X: {}, Y: {}", delta.1, delta.0);
                    app.world.player.camera.rotate(-Deg(delta.1 as f32/10.0), Deg(delta.0 as f32/10.0), Deg(0.0));

                    surface.window().set_cursor_position(
                        Position::Physical(PhysicalPosition{ x: dimensions.width as i32/2, y: dimensions.height as i32/2 })
                    ).unwrap();
                }
            },
            // this calls last after all the event finishes emitting
            // and only calls once, which is great for updating mutable variables since it'll be uniform
            Event::MainEventsCleared => {
                let mut directions = Vec::new();

                if pressed.contains(&K::A) {directions.push(CamDirection::Leftward)}
                else if pressed.contains(&K::D) {directions.push(CamDirection::Rightward)}
                else if pressed.contains(&K::W) {directions.push(CamDirection::Forward)}
                else if pressed.contains(&K::S) {directions.push(CamDirection::Backward)}
                else if pressed.contains(&K::LShift) {directions.push(CamDirection::Downward)}
                else if pressed.contains(&K::Space)  {directions.push(CamDirection::Upward)}
                else {directions.push(CamDirection::None)}

                app.world.player.camera.travel(directions);
            },
            Event::RedrawEventsCleared => {
                app.update(dimensions);
            },
            _ => {},
        }
    });
}