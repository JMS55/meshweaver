use bytemuck::{Pod, Zeroable};
use obj::load_obj;
use std::io::BufRead;
use ultraviolet::Similarity3;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsage, Device};

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub model_matrix: Similarity3,
}

impl Mesh {
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
            model_matrix: Similarity3::identity(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}
