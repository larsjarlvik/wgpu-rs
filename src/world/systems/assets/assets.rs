use std::{collections::HashMap, fs};

use crate::{camera, model, texture};
use wgpu::util::DeviceExt;

pub struct Buffers {
    pub texture_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub uniforms: super::uniforms::UniformBuffer,
}

pub struct Asset {
    pub primitives: Vec<Buffers>,
    pub bounding_box: camera::BoundingBox,
    pub density: f32,
    pub size_range: [f32; 2],
    pub slope_range: [f32; 2],
    pub temp_range: [f32; 2],
    pub temp_preferred: f32,
    pub moist_range: [f32; 2],
    pub moist_preferred: f32,
    pub render_distance: f32,
    pub rotation: [(f32, f32); 3],
    pub align: bool,
    pub radius: f32,
}

pub type AssetMap = HashMap<String, Asset>;

pub fn create(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
) -> AssetMap {
    let mut assets = HashMap::new();
    let paths = fs::read_dir("./res/assets").expect("Could not find ./res/assets!");

    for asset in paths {
        let model = model::Model::new(device, queue, &asset.unwrap().path());

        for mesh in model.meshes.iter() {
            let render_distance = mesh.get_extra_f32_or("render_distance", 1.0);
            let primitives = mesh
                .primitives
                .iter()
                .map(|p| {
                    to_buffers(
                        device,
                        sampler,
                        p,
                        &model.materials,
                        uniform_bind_group_layout,
                        texture_bind_group_layout,
                        render_distance,
                    )
                })
                .collect();

            let rx = mesh.get_extra_f32("rotation_x");
            let ry = mesh.get_extra_f32("rotation_y");
            let rz = mesh.get_extra_f32("rotation_z");
            let rotation = [(-(rx * 0.5), rx * 0.5), (-(ry * 0.5), ry * 0.5), (-(rz * 0.5), rz * 0.5)];

            assets.insert(
                mesh.name.clone(),
                Asset {
                    density: mesh.get_extra_f32("density"),
                    size_range: mesh.get_extra_range("size"),
                    slope_range: mesh.get_extra_range("slope"),
                    temp_range: mesh.get_extra_range("temp"),
                    temp_preferred: mesh.get_extra_f32("temp_preferred"),
                    moist_range: mesh.get_extra_range("moist"),
                    moist_preferred: mesh.get_extra_f32("moist_preferred"),
                    radius: mesh.get_extra_f32("radius"),
                    align: mesh.get_extra_bool("align"),
                    render_distance,
                    rotation,
                    primitives,
                    bounding_box: mesh.bounding_box.clone(),
                },
            );
        }
    }

    assets
}

fn to_buffers(
    device: &wgpu::Device,
    sampler: &wgpu::Sampler,
    primitive: &model::Primitive,
    materials: &Vec<model::Material>,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    texture_bind_group_layout: &wgpu::BindGroupLayout,
    render_distance: f32,
) -> Buffers {
    let mut vertices = vec![];
    for i in 0..primitive.positions.len() {
        vertices.push(super::data::Vertex {
            position: primitive.positions[i],
            normals: primitive.normals[i],
            tangents: primitive.tangents[i],
            tex_coords: primitive.tex_coords[i],
        });
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("asset_vertex_buffer"),
        contents: bytemuck::cast_slice(&vertices.as_slice()),
        usage: wgpu::BufferUsage::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("asset_index_buffer"),
        contents: bytemuck::cast_slice(&primitive.indices.as_slice()),
        usage: wgpu::BufferUsage::INDEX,
    });
    let num_elements = primitive.indices.len() as u32;

    let mut entries = vec![];
    let material = materials.iter().filter(|m| m.index == primitive.material_index).nth(0).unwrap();

    match &material.base_color_texture {
        Some(base_color) => {
            entries.push(texture::create_bind_group_entry(0, &base_color));
        }
        None => (),
    }
    match &material.normal_texture {
        Some(normal) => {
            entries.push(texture::create_bind_group_entry(1, &normal));
        }
        None => (),
    }

    entries.push(wgpu::BindGroupEntry {
        binding: 2,
        resource: wgpu::BindingResource::Sampler(&sampler),
    });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("asset_texture_layout"),
        layout: texture_bind_group_layout,
        entries: &entries,
    });

    let wind_factor = *material.extras.get("wind_factor").unwrap_or(&0.0);
    let uniforms = super::uniforms::UniformBuffer::new(
        device,
        uniform_bind_group_layout,
        super::uniforms::Uniforms {
            wind_factor,
            render_distance,
        },
    );

    Buffers {
        texture_bind_group,
        vertex_buffer,
        index_buffer,
        num_elements,
        uniforms,
    }
}
