mod objects;
mod renderer;

use crate::objects::Mesh;
use crate::renderer::Renderer;
use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::iter;
use std::time::{Duration, Instant};
use ultraviolet::{Rotor3, Vec3};
use wgpu::*;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, WindowBuilder};

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Meshweaver")
        .build(&event_loop)
        .unwrap();
    let mut last_frame = Instant::now();
    let mut time_accumulator = Duration::from_secs(0);

    let instance = Instance::new(BackendBit::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };
    let (device, queue) = pollster::block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    limits: Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap()
    });
    let mut swapchain_descriptor = SwapChainDescriptor {
        usage: TextureUsage::OUTPUT_ATTACHMENT,
        format: TextureFormat::Bgra8UnormSrgb,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: PresentMode::Mailbox,
    };
    let mut swapchain = device.create_swap_chain(&surface, &swapchain_descriptor);

    let mut renderer = Renderer::new(
        &device,
        swapchain_descriptor.width as f32,
        swapchain_descriptor.height as f32,
    );
    let mut meshes = vec![
        &include_bytes!("../meshes/monkey.obj")[..],
        &include_bytes!("../meshes/uvsphere.obj")[..],
    ]
    .into_par_iter()
    .map(|obj| Mesh::from_obj_file(&device, &renderer.transform_bind_group_layout, obj))
    .collect::<Vec<Mesh>>();
    meshes[0].update_transform(
        &queue,
        &device,
        &renderer.transform_bind_group_layout,
        |transform| {
            transform.translation = Vec3::new(-1.0, 0.0, 0.0);
            transform.scale = 0.5;
        },
    );
    meshes[1].update_transform(
        &queue,
        &device,
        &renderer.transform_bind_group_layout,
        |transform| {
            transform.translation = Vec3::new(1.0, 0.0, 0.0);
            transform.scale = 0.5;
        },
    );

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(_) => {
            let now = Instant::now();
            time_accumulator += last_frame.elapsed();
            last_frame = now;
        }

        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

            WindowEvent::Resized(new_inner_size) => {
                swapchain_descriptor.width = new_inner_size.width;
                swapchain_descriptor.height = new_inner_size.height;
                swapchain = device.create_swap_chain(&surface, &swapchain_descriptor);
                renderer.set_screen_size(
                    &queue,
                    &device,
                    new_inner_size.width as f32,
                    new_inner_size.height as f32,
                );
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                swapchain_descriptor.width = new_inner_size.width;
                swapchain_descriptor.height = new_inner_size.height;
                swapchain = device.create_swap_chain(&surface, &swapchain_descriptor);
                renderer.set_screen_size(
                    &queue,
                    &device,
                    new_inner_size.width as f32,
                    new_inner_size.height as f32,
                );
            }

            WindowEvent::KeyboardInput { input, .. } => {
                if input.state == ElementState::Pressed {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                        Some(VirtualKeyCode::Return) => {
                            let fullscreen = match window.fullscreen() {
                                Some(_) => None,
                                None => Some(Fullscreen::Borderless(window.current_monitor())),
                            };
                            window.set_fullscreen(fullscreen);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        },

        Event::MainEventsCleared => {
            const TARGET_TIME: Duration = Duration::from_nanos(16666670);
            while time_accumulator >= TARGET_TIME {
                meshes.par_iter_mut().for_each(|mesh| {
                    mesh.update_transform(
                        &queue,
                        &device,
                        &renderer.transform_bind_group_layout,
                        |transform| {
                            transform.rotation =
                                Rotor3::from_rotation_xz(0.5f32.to_radians()) * transform.rotation;
                        },
                    );
                });
                time_accumulator -= TARGET_TIME;
            }
            window.request_redraw();
        }

        Event::RedrawRequested(_) => {
            let frame = swapchain.get_current_frame().unwrap().output;
            let mut encoder =
                device.create_command_encoder(&CommandEncoderDescriptor { label: None });
            renderer.clear(&mut encoder, &frame.view);
            for mesh in &meshes {
                renderer.render(mesh, &mut encoder, &frame.view);
            }
            queue.submit(iter::once(encoder.finish()));
        }
        _ => {}
    });
}
