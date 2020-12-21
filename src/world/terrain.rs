use super::elevation;
use crate::{camera, settings, texture};
use cgmath::{InnerSpace, Vector3};
use image::GenericImageView;
use rayon::prelude::*;
use std::num::NonZeroU32;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
    pub normal: [f32; 3],
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
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}
pub struct TerrainTile {
    vertex_buffer: wgpu::Buffer,
    num_elements: u32,
}

pub struct Terrain {
    pub render_bundle: wgpu::RenderBundle,
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
}

impl Terrain {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera) -> Terrain {
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2Array,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                    count: Some(NonZeroU32::new(2).unwrap()),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain_pipeline_layout"),
            bind_group_layouts: &[&camera.uniforms.bind_group_layout, &texture_bind_group_layout],
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

        let texture_bind_group = build_textures(device, queue, &texture_bind_group_layout);
        let render_bundle = build_render_bundle(&device, &render_pipeline, &texture_bind_group, &camera, &Vec::new());
        Terrain {
            texture_bind_group,
            render_pipeline,
            render_bundle,
        }
    }

    pub fn create_tile(&mut self, device: &wgpu::Device, noise: &noise::OpenSimplex, x: i32, z: i32, tile_size: f32) -> TerrainTile {
        let mut to_generate = Vec::new();
        let half_tile_size = (tile_size / 2.0) as i32;

        let tx = x as f32 * tile_size;
        let tz = z as f32 * tile_size;

        for z in (-half_tile_size..half_tile_size).map(|i| (i as f32)) {
            for x in (-half_tile_size..half_tile_size).map(|i| (i as f32)) {
                to_generate.push(cgmath::Point2::new(tx + x, tz + z + 1.0));
                to_generate.push(cgmath::Point2::new(tx + x + 1.0, tz + z));
                to_generate.push(cgmath::Point2::new(tx + x, tz + z));
                to_generate.push(cgmath::Point2::new(tx + x + 1.0, tz + z + 1.0));
                to_generate.push(cgmath::Point2::new(tx + x + 1.0, tz + z));
                to_generate.push(cgmath::Point2::new(tx + x, tz + z + 1.0));
            }
        }

        let vertices = to_generate
            .par_iter()
            .map(|point| gen_vertex(noise, point.x, point.y))
            .collect::<Vec<Vertex>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices.as_slice()),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let num_elements = vertices.len() as u32;

        TerrainTile {
            vertex_buffer,
            num_elements,
        }
    }

    pub fn build(&mut self, device: &wgpu::Device, camera: &camera::Camera, tiles: Vec<&TerrainTile>) {
        self.render_bundle = build_render_bundle(&device, &self.render_pipeline, &self.texture_bind_group, &camera, &tiles);
    }
}

fn gen_vertex(noise: &noise::OpenSimplex, x: f32, z: f32) -> Vertex {
    let normal = Vector3::new(
        elevation::get(noise, x - 1.0, z) - elevation::get(noise, x + 1.0, z),
        2.0,
        elevation::get(noise, x, x - 1.0) - elevation::get(noise, x, x + 1.0),
    )
    .normalize();
    let bitangent = Vector3::cross(Vector3::new(0.0, 0.0, 1.0), normal).normalize();
    let tangent = Vector3::cross(normal, bitangent).normalize();

    Vertex {
        position: [x, elevation::get(noise, x, z), z],
        tangent: tangent.into(),
        bitangent: bitangent.into(),
        normal: normal.into(),
    }
}

fn build_render_bundle(
    device: &wgpu::Device,
    render_pipeline: &wgpu::RenderPipeline,
    texture_bind_group: &wgpu::BindGroup,
    camera: &camera::Camera,
    tiles: &Vec<&TerrainTile>,
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
    encoder.set_bind_group(1, &texture_bind_group, &[]);

    for tile in tiles {
        encoder.set_vertex_buffer(0, tile.vertex_buffer.slice(..));
        encoder.draw(0..tile.num_elements, 0..1);
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
}

fn load_textures(device: &wgpu::Device, queue: &wgpu::Queue, textures: Vec<&str>) -> Vec<wgpu::TextureView> {
    textures
        .iter()
        .map(|t| {
            let i1 = image::open(format!("res/textures/{}.png", t)).unwrap();
            texture::Texture::new(
                device,
                queue,
                &i1.as_rgba8().unwrap().to_vec(),
                i1.dimensions().0,
                i1.dimensions().1,
            )
            .view
        })
        .collect::<Vec<wgpu::TextureView>>()
}

fn build_textures(device: &wgpu::Device, queue: &wgpu::Queue, texture_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
    let sampler = &device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let textures = load_textures(device, queue, vec!["grass-basecolor", "grass-normal"]);
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("terrain_textures"),
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(&textures),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}
