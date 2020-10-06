use crate::objects::{Mesh, Vertex};
use std::mem;
use std::time::Duration;
use ultraviolet::projection::rh_yup::perspective_wgpu_dx;
use ultraviolet::{Mat4, Rotor3, Vec3};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

pub struct Renderer {
    depth_texture: TextureView,
    msaa_texture: TextureView,
    pub transform_bind_group_layout: BindGroupLayout,
    render_pipeline: RenderPipeline,

    view_matrix: Mat4,
    projection_matrix: Mat4,
    camera_uniform_buffer: Buffer,
    camera_bind_group_layout: BindGroupLayout,
    camera_bind_group: BindGroup,

    light_position: Vec3,
    light_uniform_buffer: Buffer,
    light_bind_group_layout: BindGroupLayout,
    light_bind_group: BindGroup,
}

impl Renderer {
    pub fn new(device: &Device, screen_width: f32, screen_height: f32) -> Self {
        let view_matrix = Mat4::look_at(
            Vec3::new(0.0, 1.0, 2.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::unit_y(),
        );
        let projection_matrix = perspective_wgpu_dx(45.0, screen_width / screen_height, 0.1, 100.0);
        let camera_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice((projection_matrix * view_matrix).as_slice()),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(camera_uniform_buffer.slice(..)),
            }],
        });

        let light_position = Vec3::new(-15.0, 15.0, 0.0);
        let light_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(light_position.as_slice()),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });
        let light_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStage::FRAGMENT,
                ty: BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let light_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &light_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(light_uniform_buffer.slice(..)),
            }],
        });

        let depth_texture = device
            .create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: screen_width as u32,
                    height: screen_height as u32,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 8,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_view(&TextureViewDescriptor::default());
        let msaa_texture = device
            .create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: screen_width as u32,
                    height: screen_height as u32,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 8,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_view(&TextureViewDescriptor::default());
        let transform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &transform_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex_stage: ProgrammableStageDescriptor {
                module: &device.create_shader_module(include_spirv!("../shaders/vert.spv")),
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &device.create_shader_module(include_spirv!("../shaders/frag.spv")),
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Ccw,
                cull_mode: CullMode::Back,
                clamp_depth: false,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format: TextureFormat::Bgra8UnormSrgb,
                alpha_blend: BlendDescriptor::REPLACE,
                color_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: Some(DepthStencilStateDescriptor {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilStateDescriptor::default(),
            }),
            vertex_state: VertexStateDescriptor {
                index_format: IndexFormat::Uint16,
                vertex_buffers: &[VertexBufferDescriptor {
                    stride: mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float3, 1 => Float3],
                }],
            },
            sample_count: 8,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            render_pipeline,
            depth_texture,
            transform_bind_group_layout,
            msaa_texture,

            view_matrix,
            projection_matrix,
            camera_uniform_buffer,
            camera_bind_group_layout,
            camera_bind_group,

            light_position,
            light_uniform_buffer,
            light_bind_group_layout,
            light_bind_group,
        }
    }

    pub fn update_light_position(
        &mut self,
        queue: &Queue,
        device: &Device,
        time_elapsed: Duration,
    ) {
        // One full rotation every 5 seconds
        let angle = (time_elapsed.as_secs_f32() * 365.0) / 5.0;
        self.light_position
            .rotate_by(Rotor3::from_rotation_xz(angle.to_radians()));
        queue.write_buffer(
            &self.light_uniform_buffer,
            0,
            bytemuck::cast_slice(self.light_position.as_slice()),
        );
        self.light_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.light_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(self.light_uniform_buffer.slice(..)),
            }],
        });
    }

    pub fn render(&self, mesh: &Mesh, encoder: &mut CommandEncoder, render_target: &TextureView) {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: &self.msaa_texture,
                resolve_target: Some(render_target),
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..));
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &mesh.transform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.light_bind_group, &[]);
        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }

    pub fn set_screen_size(&mut self, queue: &Queue, device: &Device, width: f32, height: f32) {
        self.depth_texture = device
            .create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 8,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_view(&TextureViewDescriptor::default());
        self.msaa_texture = device
            .create_texture(&TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth: 1,
                },
                mip_level_count: 1,
                sample_count: 8,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                usage: TextureUsage::OUTPUT_ATTACHMENT,
            })
            .create_view(&TextureViewDescriptor::default());

        self.projection_matrix = perspective_wgpu_dx(45.0, width / height, 0.1, 100.0);
        queue.write_buffer(
            &self.camera_uniform_buffer,
            0,
            bytemuck::cast_slice((self.projection_matrix * self.view_matrix).as_slice()),
        );
        self.camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(self.camera_uniform_buffer.slice(..)),
            }],
        });
    }
}
