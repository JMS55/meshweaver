use std::iter;
use wgpu::*;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Meshweaver")
        .build(&event_loop)
        .unwrap();

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

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(new_inner_size) => {
                swapchain_descriptor.width = new_inner_size.width;
                swapchain_descriptor.height = new_inner_size.height;
                swapchain = device.create_swap_chain(&surface, &swapchain_descriptor);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                swapchain_descriptor.width = new_inner_size.width;
                swapchain_descriptor.height = new_inner_size.height;
                swapchain = device.create_swap_chain(&surface, &swapchain_descriptor);
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            let frame = swapchain.get_current_frame().unwrap().output;
            let mut encoder =
                device.create_command_encoder(&CommandEncoderDescriptor { label: None });
            {
                let _ = encoder.begin_render_pass(&RenderPassDescriptor {
                    color_attachments: &[RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color::BLACK),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
            }
            queue.submit(iter::once(encoder.finish()));
        }
        _ => {}
    });
}
