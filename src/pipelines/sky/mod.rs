use super::render_targets;
use crate::{camera, settings, texture};
mod data;

pub struct Sky {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group: wgpu::BindGroup,
    pub uniforms: data::UniformBuffer,
    pub texture_view: String,
    pub depth_texture_view: String,
}

impl Sky {
    pub fn new(device: &wgpu::Device, viewport: &camera::Viewport, render_targets: &mut render_targets::RenderTargets) -> Self {
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                texture::create_bind_group_layout(0, wgpu::TextureSampleType::Depth),
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

        let texture_view = render_targets.create(&device, viewport.width, viewport.height, settings::COLOR_TEXTURE_FORMAT);
        let depth_texture_view = render_targets.create(&device, viewport.width, viewport.height, settings::DEPTH_TEXTURE_FORMAT);
        let sampler = texture::create_sampler(device, wgpu::AddressMode::ClampToEdge, wgpu::FilterMode::Linear);
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                texture::create_bind_group_entry(0, render_targets.get(&depth_texture_view)),
                texture::create_bind_group_entry(1, render_targets.get(&texture_view)),
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout, &viewport.bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/sky.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/sky.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[settings::COLOR_TEXTURE_FORMAT.into()],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        let uniforms = data::UniformBuffer::new(
            &device,
            &uniform_bind_group_layout,
            data::Uniforms {
                light_dir: settings::LIGHT_DIR.into(),
                not_used: 0.0,
                sky_color: settings::SKY_COLOR.into(),
            },
        );

        Self {
            render_pipeline,
            texture_bind_group,
            uniforms,
            texture_view,
            depth_texture_view,
        }
    }
}
