use crate::*;
mod assets;
mod data;
pub use {self::data::Instance, self::data::Vertex};

pub struct Assets {
    pub sampler: wgpu::Sampler,
    pub render_pipeline: wgpu::RenderPipeline,
    pub shadow_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub assets: assets::Assets,
}

impl Assets {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: &camera::Viewport,
        light_uniforms: &wgpu::BindGroupLayout,
        light_textures: &wgpu::BindGroupLayout,
    ) -> Self {
        let sampler = texture::create_sampler(device, wgpu::AddressMode::Repeat, wgpu::FilterMode::Linear);
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                texture::create_bind_group_layout(0, wgpu::TextureSampleType::Uint),
                texture::create_bind_group_layout(1, wgpu::TextureSampleType::Uint),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("model_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &viewport.bind_group_layout,
                light_uniforms,
                light_textures,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/assets.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/assets.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("model_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[data::Vertex::desc(), data::Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: settings::COLOR_TEXTURE_FORMAT,
                    alpha_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    color_blend: wgpu::BlendState {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                topology: wgpu::PrimitiveTopology::TriangleList,
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

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/assets-shadows.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/assets-shadows.frag.spv"));
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("model_shadow_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[data::Vertex::desc(), data::Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: settings::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
                clamp_depth: true,
            }),
            multisample: wgpu::MultisampleState::default(),
        });

        let now = Instant::now();
        let models = assets::Assets::new(device, queue, &texture_bind_group_layout, &sampler);
        println!("Assets: {} ms", now.elapsed().as_millis());

        Self {
            sampler,
            render_pipeline,
            shadow_pipeline,
            texture_bind_group_layout,
            assets: models,
        }
    }
}
