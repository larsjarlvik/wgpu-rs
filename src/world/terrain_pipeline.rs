use super::{noise, terrain_tile};
use crate::{camera, settings, texture};
use image::GenericImageView;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::num::NonZeroU32;

pub struct Terrain {
    pub render_bundle: wgpu::RenderBundle,
    pub compute: terrain_tile::Compute,
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
}

impl Terrain {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera, tile_size: u32) -> Terrain {
        let compute = terrain_tile::Compute::new(device, tile_size as f32);

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
                    count: Some(NonZeroU32::new(5).unwrap()),
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
                vertex_buffers: &[terrain_tile::Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let texture_bind_group = build_textures(device, queue, &texture_bind_group_layout);
        let render_bundle = build_render_bundle(
            &device,
            &render_pipeline,
            &texture_bind_group,
            &camera,
            &Vec::new(),
            compute.num_elements,
        );

        Terrain {
            texture_bind_group,
            render_pipeline,
            render_bundle,
            compute,
        }
    }

    pub fn create_tile(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        noise: &noise::Noise,
        x: i32,
        z: i32,
    ) -> terrain_tile::TerrainTile {
        let render_buffer = self.compute.compute(device, queue, x as f32, z as f32, noise);
        terrain_tile::TerrainTile { render_buffer }
    }

    pub fn refresh(&mut self, device: &wgpu::Device, camera: &camera::Camera, tiles: Vec<&terrain_tile::TerrainTile>) {
        self.render_bundle = build_render_bundle(
            &device,
            &self.render_pipeline,
            &self.texture_bind_group,
            &camera,
            &tiles,
            self.compute.num_elements,
        );
    }
}

fn build_render_bundle(
    device: &wgpu::Device,
    render_pipeline: &wgpu::RenderPipeline,
    texture_bind_group: &wgpu::BindGroup,
    camera: &camera::Camera,
    tiles: &Vec<&terrain_tile::TerrainTile>,
    num_elements: u32,
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
        encoder.set_vertex_buffer(0, tile.render_buffer.slice(..));
        encoder.draw(0..num_elements, 0..1);
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
}

fn load_textures(device: &wgpu::Device, queue: &wgpu::Queue, textures: Vec<&str>) -> Vec<wgpu::TextureView> {
    textures
        .par_iter()
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

    let textures = load_textures(
        device,
        queue,
        vec!["grass", "grass_normals", "cliffwall", "cliffwall_normals", "sand"],
    );
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
