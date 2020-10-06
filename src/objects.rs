use bytemuck::{Pod, Zeroable};
use obj::load_obj;
use std::io::BufRead;
use ultraviolet::Similarity3;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,

    transform: Similarity3,
    transform_uniform_buffer: Buffer,
    pub transform_bind_group: BindGroup,
}

impl Mesh {
    pub fn from_obj_file<T: BufRead>(
        device: &Device,
        transform_bind_group_layout: &BindGroupLayout,
        file: T,
    ) -> Self {
        let obj = load_obj::<_, _, u16>(file).unwrap();
        let vertices = obj
            .vertices
            .into_iter()
            .map(|vertex: obj::Vertex| Vertex {
                position: vertex.position,
                normal: vertex.normal,
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

        let transform = Similarity3::identity();
        let transform_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(transform.into_homogeneous_matrix().as_slice()),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });
        let transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &transform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(transform_uniform_buffer.slice(..)),
            }],
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count,

            transform,
            transform_uniform_buffer,
            transform_bind_group,
        }
    }

    pub fn update_transform<F: FnOnce(&mut Similarity3)>(
        &mut self,
        queue: &Queue,
        device: &Device,
        transform_bind_group_layout: &BindGroupLayout,
        func: F,
    ) {
        func(&mut self.transform);
        queue.write_buffer(
            &self.transform_uniform_buffer,
            0,
            bytemuck::cast_slice(self.transform.into_homogeneous_matrix().as_slice()),
        );
        self.transform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: transform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(self.transform_uniform_buffer.slice(..)),
            }],
        });
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}
