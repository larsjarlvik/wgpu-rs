use crate::{camera, noise, plane, settings, texture};
use image::GenericImageView;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::convert::TryInto;

pub struct Terrain {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group: wgpu::BindGroup,
    pub noise_bindings: noise::NoiseBindings,
}

impl Terrain {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, viewport: &camera::Viewport, noise: &noise::Noise) -> Terrain {
        let noise_bindings = noise.create_bindings(device);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                texture::create_array_bind_group_layout(0, wgpu::TextureSampleType::Uint, 5),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain_pipeline_layout"),
            bind_group_layouts: &[
                &viewport.bind_group_layout,
                &texture_bind_group_layout,
                &noise_bindings.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/terrain.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/terrain.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[plane::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[settings::COLOR_TEXTURE_FORMAT.into(), settings::COLOR_TEXTURE_FORMAT.into()],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: settings::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState::default(),
        });

        let texture_bind_group = build_textures(device, queue, &texture_bind_group_layout);

        Terrain {
            texture_bind_group,
            render_pipeline,
            noise_bindings,
        }
    }
}

fn load_texture(device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> wgpu::TextureView {
    let i1 = image::open(format!("res/textures/{}.png", path)).unwrap();
    texture::create_mipmapped_view(
        device,
        queue,
        &i1.as_rgba8().unwrap().to_vec(),
        i1.dimensions().0,
        i1.dimensions().1,
    )
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

    // TODO: Use 3D texture
    let paths = vec!["grass", "grass_normals", "cliffwall", "cliffwall_normals", "sand"];
    let textures = paths
        .par_iter()
        .map(|&t| load_texture(device, queue, t))
        .collect::<Vec<wgpu::TextureView>>();
    let t: &[&wgpu::TextureView; 5] = &textures.iter().collect::<Vec<_>>().try_into().unwrap();

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("terrain_textures"),
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(t),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}
