use std::{collections::HashMap, mem};
use strum_macros::EnumIter;
use strum::IntoEnumIterator;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

#[derive(EnumIter, PartialEq, Eq, Hash, Clone)]
pub enum ConnectType {
    None,
    ZPos,
    XPos,
    ZNeg,
    XNeg,
    ZPosXPos,
    XPosZNeg,
    ZNegXNeg,
    XNegZPos,
}

pub struct LodBuffer {
    pub index_buffer: wgpu::Buffer,
    pub length: u32,
}

pub struct Plane {
    pub size: u32,
    pub length: u32,
    pub vertices: Vec<Vertex>,
}

impl Plane {
    pub fn new(size: u32) -> Self {
        let mut vertices = Vec::new();
        let half_size = size as f32 / 2.0;

        for z in 0..size + 1 {
            for x in 0..size + 1 {
                vertices.push(Vertex {
                    position: [(x as f32) - half_size, 0.0, (z as f32) - half_size],
                    normal: [0.0, 0.0, 0.0],
                });
            }
        }

        let length = vertices.len() as u32;
        Self {
            vertices,
            length,
            size,
        }
    }

    pub fn create_indices(&self, device: &wgpu::Device, lod: u32) -> HashMap<ConnectType, LodBuffer> {
        let lod = 2u32.pow(lod as u32) / 2;
        let mut lod_buffers: HashMap<ConnectType, LodBuffer> = HashMap::new();

        for connect_type in ConnectType::iter() {
            let mut indices = Vec::new();

            match connect_type {
                ConnectType::ZNeg => {
                    self.z_neg(&mut indices, lod);
                    self.fill_tile(&mut indices, lod, &connect_type);
                }
                ConnectType::XPos => {
                    self.fill_tile(&mut indices, lod, &connect_type);
                    self.x_pos(&mut indices, lod);
                }
                ConnectType::ZPos => {
                    self.fill_tile(&mut indices, lod, &connect_type);
                    self.z_pos(&mut indices, lod);
                }
                ConnectType::XNeg => {
                    self.x_neg(&mut indices, lod);
                    self.fill_tile(&mut indices, lod, &connect_type);
                }
                ConnectType::XNegZPos => {
                    self.x_neg(&mut indices, lod);
                    self.fill_tile(&mut indices, lod, &connect_type);
                    self.z_pos(&mut indices, lod);
                }
                ConnectType::XPosZNeg => {
                    self.z_neg(&mut indices, lod);
                    self.fill_tile(&mut indices, lod, &connect_type);
                    self.x_pos(&mut indices, lod);
                }
                ConnectType::ZNegXNeg => {
                    self.x_neg(&mut indices, lod);
                    self.z_neg(&mut indices, lod);
                    self.fill_tile(&mut indices, lod, &connect_type);
                }
                ConnectType::ZPosXPos => {
                    self.fill_tile(&mut indices, lod, &connect_type);
                    self.z_pos(&mut indices, lod);
                    indices.push(self.get_index(0, self.size));
                    indices.push(self.get_index(self.size, self.size));
                    self.x_pos(&mut indices, lod);
                }
                ConnectType::None => {
                    self.fill_tile(&mut indices, lod, &connect_type);
                }
            }

            lod_buffers.insert(connect_type, LodBuffer {
                length: indices.len() as u32,
                index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&indices.as_slice()),
                    usage: wgpu::BufferUsage::INDEX,
                }),
            });
        }

        lod_buffers
    }

    fn fill_tile(&self, indices: &mut Vec<u32>, lod: u32, connect_type: &ConnectType) {
        for z in (lod_z_neg(lod, connect_type)..(self.size - lod_z_pos(lod, connect_type))).step_by(lod as usize) {
            if z > 0 {
                indices.push(self.get_index(lod_x_neg(lod, connect_type), z));
            }

            for x in (lod_x_neg(lod, connect_type)..(self.size + lod - lod_x_pos(lod, connect_type))).step_by(lod as usize) {
                indices.push(self.get_index(x, z));
                indices.push(self.get_index(x, z + lod));
            }

            indices.push(self.get_index(self.size - lod_x_pos(lod, connect_type), z + lod));
        }
    }

    fn get_index(&self, x: u32, z: u32) -> u32 {
        z * (self.size + 1) + x
    }

    fn z_neg(&self, indices: &mut Vec<u32>, lod: u32) {
        for x in 0..self.size + 1 {
            indices.push(self.get_index(x, 0));
            indices.push(self.get_index(x + (x % lod), lod));
        }
        indices.push(self.get_index(self.size + 1 - lod, lod));
    }

    fn x_pos(&self, indices: &mut Vec<u32>, lod: u32) {
        for z in (0..self.size + 1).rev() {
            indices.push(self.get_index(self.size, z));
            indices.push(self.get_index(self.size - lod, z - (z % lod)));
        }
    }

    fn z_pos(&self, indices: &mut Vec<u32>, lod: u32) {
        for x in (0..self.size + 1).rev() {
            indices.push(self.get_index(x + (x % lod), self.size - lod));
            indices.push(self.get_index(x, self.size));
        }
    }

    fn x_neg(&self, indices: &mut Vec<u32>, lod: u32) {
        for z in (0..self.size + 1).rev() {
            indices.push(self.get_index(0, z));
            indices.push(self.get_index(lod, z - (z % lod)));
        }
    }
}

fn lod_x_pos(lod: u32, connect_type: &ConnectType) -> u32 {
    match connect_type {
        ConnectType::XPos | ConnectType::XPosZNeg | ConnectType::ZPosXPos => lod,
        _ => 0,
    }
}
fn lod_x_neg(lod: u32, connect_type: &ConnectType) -> u32 {
    match connect_type {
        ConnectType::XNeg | ConnectType::XNegZPos | ConnectType::ZNegXNeg => lod,
        _ => 0,
    }
}
fn lod_z_neg(lod: u32, connect_type: &ConnectType) -> u32 {
    match connect_type {
        ConnectType::ZNeg | ConnectType::XPosZNeg | ConnectType::ZNegXNeg => lod,
        _ => 0,
    }
}
fn lod_z_pos(lod: u32, connect_type: &ConnectType) -> u32 {
    match connect_type {
        ConnectType::ZPos | ConnectType::XNegZPos | ConnectType::ZPosXPos => lod,
        _ => 0,
    }
}
