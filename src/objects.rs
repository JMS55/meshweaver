use bytemuck::{Pod, Zeroable};
use ddsfile::Dds;
use obj::{load_obj, TexturedVertex};
use std::io::{BufRead, Read};
use ultraviolet::{Mat4, Similarity3};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}

pub struct Mesh {
    data: MeshData,
    pub instances: Vec<Similarity3>,
}

impl Mesh {
    pub fn from_obj_and_texture<F: BufRead, T: Read>(
        queue: &Queue,
        device: &Device,
        file: F,
        dxt5_texture: &mut T,
    ) -> Self {
        Self {
            data: MeshData::from_obj_and_texture(queue, device, file, dxt5_texture),
            instances: Vec::new(),
        }
    }

    pub fn create_instances_bind_group(
        &self,
        device: &Device,
        instances_bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        let instance_data = self
            .instances
            .iter()
            .map(|transform| transform.into_homogeneous_matrix())
            .collect::<Vec<Mat4>>();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&instance_data),
            usage: BufferUsage::STORAGE,
        });
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: instances_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(instance_buffer.slice(..)),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&self.data.texture),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&self.data.sampler),
                },
            ],
        })
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.data.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.data.index_buffer
    }

    pub fn index_count(&self) -> u32 {
        self.data.index_count
    }
}

struct MeshData {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,

    texture: TextureView,
    sampler: Sampler,
}

impl MeshData {
    fn from_obj_and_texture<F: BufRead, T: Read>(
        queue: &Queue,
        device: &Device,
        file: F,
        dxt5_texture: &mut T,
    ) -> Self {
        let obj = load_obj::<TexturedVertex, _, u16>(file).unwrap();
        let vertices = obj
            .vertices
            .into_iter()
            .map(|vertex| Vertex {
                position: vertex.position,
                normal: vertex.normal,
                uv: [vertex.texture[0], vertex.texture[1]],
            })
            .collect::<Vec<Vertex>>();
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&obj.indices),
            usage: BufferUsage::INDEX,
        });
        let index_count = obj.indices.len() as u32;

        let texture_data = Dds::read(dxt5_texture).unwrap();
        let texture_size = Extent3d {
            width: texture_data.get_width(),
            height: texture_data.get_height(),
            depth: texture_data.get_depth(),
        };
        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bc3RgbaUnormSrgb,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        });
        queue.write_texture(
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            texture_data.get_data(0).unwrap(),
            TextureDataLayout {
                offset: 0,
                bytes_per_row: texture_data.get_width() * 4,
                rows_per_image: 0,
            },
            texture_size,
        );
        let texture = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: None,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count,

            texture,
            sampler,
        }
    }
}
