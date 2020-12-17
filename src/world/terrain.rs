use super::elevation;
use crate::{camera, settings};
use cgmath::{InnerSpace, Vector3};
use rayon::prelude::*;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normals: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}
pub struct Tile {
    vertex_buffer: wgpu::Buffer,
    num_elements: u32,
}

pub struct Terrain {
    pub render_bundle: wgpu::RenderBundle,
    render_pipeline: wgpu::RenderPipeline,
    seed: i32,
    tiles: Vec<Tile>,
}

impl Terrain {
    pub fn new(device: &wgpu::Device, camera: &camera::Camera) -> Terrain {
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain_pipeline_layout"),
            bind_group_layouts: &[&camera.uniforms.bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("../shaders-compiled/terrain.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../shaders-compiled/terrain.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                settings::COLOR_TEXTURE_FORMAT.into(),
                settings::COLOR_TEXTURE_FORMAT.into(),
                settings::COLOR_TEXTURE_FORMAT.into(),
            ],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: settings::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let render_bundle = build_render_bundle(&device, &render_pipeline, &camera, &Vec::new());
        Terrain {
            render_pipeline,
            render_bundle,
            seed: 100,
            tiles: Vec::new(),
        }
    }

    pub fn add_tile(&mut self, device: &wgpu::Device, tx: f32, ty: f32, tile_size: f32) {
        let mut to_generate = Vec::new();
        let half_tile_size = (tile_size / 2.0) as i32;

        for y in (-half_tile_size..half_tile_size).map(|i| (i as f32)) {
            for x in (-half_tile_size..half_tile_size).map(|i| (i as f32)) {
                to_generate.push(cgmath::Point2::new(tx + x, ty + y + 1.0));
                to_generate.push(cgmath::Point2::new(tx + x + 1.0, ty + y));
                to_generate.push(cgmath::Point2::new(tx + x, ty + y));
                to_generate.push(cgmath::Point2::new(tx + x + 1.0, ty + y + 1.0));
                to_generate.push(cgmath::Point2::new(tx + x + 1.0, ty + y));
                to_generate.push(cgmath::Point2::new(tx + x, ty + y + 1.0));
            }
        }

        let vertices = to_generate
            .par_iter()
            .map(|point| gen_vertex(self.seed, point.x, point.y))
            .collect::<Vec<Vertex>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices.as_slice()),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let num_elements = vertices.len() as u32;

        &self.tiles.push(Tile {
            vertex_buffer,
            num_elements,
        });
    }

    pub fn build(&mut self, device: &wgpu::Device, camera: &camera::Camera) {
        self.render_bundle = build_render_bundle(&device, &self.render_pipeline, &camera, &self.tiles);
    }

    pub fn get_elevation(&self, x: f32, z: f32) -> f32 {
        get_elevation(self.seed, x, z)
    }
}

fn build_render_bundle(
    device: &wgpu::Device,
    render_pipeline: &wgpu::RenderPipeline,
    camera: &camera::Camera,
    tiles: &Vec<Tile>,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[
            settings::COLOR_TEXTURE_FORMAT,
            settings::COLOR_TEXTURE_FORMAT,
            settings::COLOR_TEXTURE_FORMAT,
        ],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });
    encoder.set_pipeline(&render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);

    for tile in tiles {
        encoder.set_vertex_buffer(0, tile.vertex_buffer.slice(..));
        encoder.draw(0..tile.num_elements, 0..1);
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
}

fn get_elevation(seed: i32, x: f32, z: f32) -> f32 {
    elevation::perlin(x * 0.02, 0.0, z * 0.02, 4, 0.5, seed) * 30.0
}

fn gen_vertex(seed: i32, x: f32, z: f32) -> Vertex {
    let h_l = get_elevation(seed, x - 1.0, z);
    let h_r = get_elevation(seed, x + 1.0, z);
    let h_d = get_elevation(seed, x, x - 1.0);
    let h_u = get_elevation(seed, x, x + 1.0);

    let normals = Vector3::new(h_l - h_r, 2.0, h_d - h_u);

    Vertex {
        position: [x, get_elevation(seed, x, z), z],
        normals: normals.normalize().into(),
    }
}
