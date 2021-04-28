use cgmath::*;
use std::{collections::HashMap, mem};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use wgpu::util::DeviceExt;

use crate::settings;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float2,
            }],
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
    XPosZPos,
    XPosZNeg,
    XNegZNeg,
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
                    position: [(x as f32) - half_size, (z as f32) - half_size],
                });
            }
        }

        let length = vertices.len() as u32;
        Self { vertices, length, size }
    }

    pub fn create_indices(&self, device: &wgpu::Device, lod: u32) -> HashMap<ConnectType, LodBuffer> {
        let mut lod_buffers: HashMap<ConnectType, LodBuffer> = HashMap::new();
        let n_lod = 2u32.pow(lod as u32 - 1) / 2;
        let lod = 2u32.pow(lod as u32) / 2;

        for connect_type in ConnectType::iter() {
            let mut indices = Vec::new();

            if lod > 1 {
                match connect_type {
                    ConnectType::ZNeg => {
                        self.z_neg(&mut indices, lod, n_lod);
                        indices.push(self.get_index(0, 0));
                        self.fill_tile(&mut indices, lod, &connect_type);
                    }
                    ConnectType::XPos => {
                        self.fill_tile(&mut indices, lod, &connect_type);
                        self.x_pos(&mut indices, lod, n_lod);
                    }
                    ConnectType::ZPos => {
                        self.fill_tile(&mut indices, lod, &connect_type);
                        self.z_pos(&mut indices, lod, n_lod);
                    }
                    ConnectType::XNeg => {
                        self.x_neg(&mut indices, lod, n_lod);
                        self.fill_tile(&mut indices, lod, &connect_type);
                    }
                    ConnectType::XNegZPos => {
                        self.x_neg(&mut indices, lod, n_lod);
                        self.fill_tile(&mut indices, lod, &connect_type);
                        self.z_pos(&mut indices, lod, n_lod);
                    }
                    ConnectType::XPosZNeg => {
                        self.z_neg(&mut indices, lod, n_lod);
                        indices.push(self.get_index(0, 0));
                        self.fill_tile(&mut indices, lod, &connect_type);
                        self.x_pos(&mut indices, lod, n_lod);
                    }
                    ConnectType::XNegZNeg => {
                        self.z_neg(&mut indices, lod, n_lod);
                        indices.push(self.get_index(0, 0));
                        indices.push(self.get_index(0, self.size));
                        self.x_neg(&mut indices, lod, n_lod);
                        indices.push(self.get_index(lod, 0));
                        self.fill_tile(&mut indices, lod, &connect_type);
                    }
                    ConnectType::XPosZPos => {
                        self.fill_tile(&mut indices, lod, &connect_type);
                        indices.push(self.get_index(self.size, self.size - lod));
                        indices.push(self.get_index(self.size, self.size - lod));
                        self.z_pos(&mut indices, lod, n_lod);
                        indices.push(self.get_index(0, self.size));
                        indices.push(self.get_index(self.size, self.size));
                        self.x_pos(&mut indices, lod, n_lod);
                    }
                    ConnectType::None => {
                        self.fill_tile(&mut indices, lod, &connect_type);
                    }
                }
            } else {
                self.fill_tile(&mut indices, lod, &connect_type);
            }

            lod_buffers.insert(
                connect_type,
                LodBuffer {
                    length: indices.len() as u32,
                    index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsage::INDEX,
                    }),
                },
            );
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

    pub fn get_index(&self, x: u32, z: u32) -> u32 {
        let index = z * (self.size + 1) + x;
        if index >= self.vertices.len() as u32 {
            println!("x: {} z: {} index: {} max: {}", x, z, index, self.vertices.len());
            panic!("Index buffer out of range!");
        }
        index
    }

    fn z_neg(&self, indices: &mut Vec<u32>, lod: u32, n_lod: u32) {
        for x in (0..self.size + 1).rev().step_by(n_lod as usize) {
            indices.push(self.get_index(x + (x % lod), lod));
            indices.push(self.get_index(x, 0));
        }
    }

    fn x_pos(&self, indices: &mut Vec<u32>, lod: u32, n_lod: u32) {
        for z in (0..self.size + 1).rev().step_by(n_lod as usize) {
            indices.push(self.get_index(self.size, z));
            indices.push(self.get_index(self.size - lod, z - (z % lod)));
        }
    }

    fn z_pos(&self, indices: &mut Vec<u32>, lod: u32, n_lod: u32) {
        for x in (0..self.size + 1).rev().step_by(n_lod as usize) {
            indices.push(self.get_index(x + (x % lod), self.size - lod));
            indices.push(self.get_index(x, self.size));
        }
    }

    fn x_neg(&self, indices: &mut Vec<u32>, lod: u32, n_lod: u32) {
        for z in (0..self.size + 1).rev().step_by(n_lod as usize) {
            indices.push(self.get_index(0, z));
            indices.push(self.get_index(lod, z - (z % lod)));
        }
    }
}

fn lod_x_pos(lod: u32, connect_type: &ConnectType) -> u32 {
    match connect_type {
        ConnectType::XPos | ConnectType::XPosZNeg | ConnectType::XPosZPos => lod,
        _ => 0,
    }
}
fn lod_x_neg(lod: u32, connect_type: &ConnectType) -> u32 {
    match connect_type {
        ConnectType::XNeg | ConnectType::XNegZPos | ConnectType::XNegZNeg => lod,
        _ => 0,
    }
}
fn lod_z_neg(lod: u32, connect_type: &ConnectType) -> u32 {
    match connect_type {
        ConnectType::ZNeg | ConnectType::XPosZNeg | ConnectType::XNegZNeg => lod,
        _ => 0,
    }
}
fn lod_z_pos(lod: u32, connect_type: &ConnectType) -> u32 {
    match connect_type {
        ConnectType::ZPos | ConnectType::XNegZPos | ConnectType::XPosZPos => lod,
        _ => 0,
    }
}

pub fn get_connect_type(a: Vector3<f32>, b: Vector3<f32>, lod: u32, z_far: f32) -> ConnectType {
    let ts = settings::TILE_SIZE as f32;

    if get_lod(a, vec3(b.x + ts, 0.0, b.z), z_far) < lod {
        if get_lod(a, vec3(b.x, 0.0, b.z + ts), z_far) < lod {
            return ConnectType::XPosZPos;
        }
        if get_lod(a, vec3(b.x, 0.0, b.z - ts), z_far) < lod {
            return ConnectType::XPosZNeg;
        }

        return ConnectType::XPos;
    }
    if get_lod(a, vec3(b.x - ts, 0.0, b.z), z_far) < lod {
        if get_lod(a, vec3(b.x, 0.0, b.z + ts), z_far) < lod {
            return ConnectType::XNegZPos;
        }
        if get_lod(a, vec3(b.x, 0.0, b.z - ts), z_far) < lod {
            return ConnectType::XNegZNeg;
        }
        return ConnectType::XNeg;
    }
    if get_lod(a, vec3(b.x, 0.0, b.z + ts), z_far) < lod {
        return ConnectType::ZPos;
    }
    if get_lod(a, vec3(b.x, 0.0, b.z - ts), z_far) < lod {
        return ConnectType::ZNeg;
    }

    ConnectType::None
}

pub fn get_lod(a: Vector3<f32>, b: Vector3<f32>, z_far: f32) -> u32 {
    let distance = a.distance(b) / z_far;
    for i in 0..settings::LODS.len() {
        let lod = settings::LODS.get(i).expect("Failed to get LOD!");
        if lod > &distance {
            return i as u32;
        }
    }

    settings::LODS.len() as u32
}
