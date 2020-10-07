use bytemuck::{Pod, Zeroable};
use obj::load_obj;
use std::io::BufRead;
use ultraviolet::Similarity3;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::*;

pub struct Mesh {
    pub data: MeshData,
    pub instances: Vec<Similarity3>,
}

impl Mesh {
    pub fn from_obj_file<T: BufRead>(device: &Device, file: T) -> Self {
        Self {
            data: MeshData::from_obj_file(device, file),
            instances: Vec::new(),
        }
    }

    pub fn create_bind_group(
        &self,
        device: &Device,
        instances_bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        let instance_data = self
            .instances
            .iter()
            .map(|transform| Mat4Raw {
                data: *transform.into_homogeneous_matrix().as_array(),
            })
            .collect::<Vec<Mat4Raw>>();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&instance_data),
            usage: BufferUsage::STORAGE,
        });
        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: instances_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(instance_buffer.slice(..)),
            }],
        })
    }
}

pub struct MeshData {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    index_count: u32,
}

impl MeshData {
    pub fn from_obj_file<T: BufRead>(device: &Device, file: T) -> Self {
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
        Self {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }

    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn index_count(&self) -> u32 {
        self.index_count
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Mat4Raw {
    data: [f32; 16],
}
