mod compute;
use crate::{camera, noise, plane, settings, texture};
use image::GenericImageView;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::num::NonZeroU32;

pub struct Terrain {
    pub compute: compute::Compute,
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group: wgpu::BindGroup,
    pub noise_bindings: noise::NoiseBindings,
}

impl Terrain {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera, noise: &noise::Noise) -> Terrain {
        let noise_bindings = noise.create_bindings(device);
        let compute = compute::Compute::new(device, noise);

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
            bind_group_layouts: &[
                &camera.uniforms.bind_group_layout,
                &texture_bind_group_layout,
                &noise_bindings.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("../../shaders-compiled/terrain.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../../shaders-compiled/terrain.frag.spv"));
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
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[settings::COLOR_TEXTURE_FORMAT.into(), settings::COLOR_TEXTURE_FORMAT.into()],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: settings::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[plane::Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let texture_bind_group = build_textures(device, queue, &texture_bind_group_layout);

        Terrain {
            texture_bind_group,
            render_pipeline,
            compute,
            noise_bindings,
        }
    }
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
