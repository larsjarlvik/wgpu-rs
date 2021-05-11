use crate::{camera, noise, plane, settings, texture};
use std::{collections::HashMap, time::Instant};
use wgpu::util::DeviceExt;
mod uniforms;

pub struct Water {
    pub lods: Vec<HashMap<plane::ConnectType, plane::LodBuffer>>,
    pub uniforms: uniforms::UniformBuffer,
    pub render_pipeline: wgpu::RenderPipeline,
    pub noise_bindings: noise::NoiseBindings,
    pub refraction_texture_view: wgpu::TextureView,
    pub refraction_depth_texture_view: wgpu::TextureView,
    pub reflection_texture_view: wgpu::TextureView,
    pub reflection_depth_texture_view: wgpu::TextureView,
    pub texture_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub node_uniform_bind_group_layout: wgpu::BindGroupLayout,
}

impl Water {
    pub fn new(
        device: &wgpu::Device,
        viewport: &camera::Viewport,
        noise: &noise::Noise,
        tile: &plane::Plane,
        light_uniforms: &wgpu::BindGroupLayout,
        light_textures: &wgpu::BindGroupLayout,
    ) -> Self {
        let noise_bindings = noise.create_bindings(device);
        let uniforms = uniforms::UniformBuffer::new(&device, uniforms::Uniforms { time: 0.0 });
        let mut lods = vec![];

        for lod in 0..=settings::LODS.len() {
            let indices_lod = tile.create_indices(&device, lod as u32 + 1);
            lods.push(indices_lod);
        }

        // Textures
        let sampler = texture::create_sampler(device, wgpu::AddressMode::ClampToEdge, wgpu::FilterMode::Nearest);
        let refraction_texture_view = texture::create_view(&device, viewport.width, viewport.height, settings::COLOR_TEXTURE_FORMAT);
        let refraction_depth_texture_view = texture::create_view(&device, viewport.width, viewport.height, settings::DEPTH_TEXTURE_FORMAT);
        let reflection_texture_view = texture::create_view(&device, viewport.width, viewport.height, settings::COLOR_TEXTURE_FORMAT);
        let reflection_depth_texture_view = texture::create_view(&device, viewport.width, viewport.height, settings::DEPTH_TEXTURE_FORMAT);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water_texture_bind_group_layout"),
            entries: &[
                texture::create_bind_group_layout(0, wgpu::TextureSampleType::Depth),
                texture::create_bind_group_layout(1, wgpu::TextureSampleType::Uint),
                texture::create_bind_group_layout(2, wgpu::TextureSampleType::Uint),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("deferred_textures"),
            layout: &texture_bind_group_layout,
            entries: &[
                texture::create_bind_group_entry(0, &refraction_depth_texture_view),
                texture::create_bind_group_entry(1, &refraction_texture_view),
                texture::create_bind_group_entry(2, &reflection_texture_view),
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let node_uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water_pipeline_layout"),
            bind_group_layouts: &[
                &viewport.bind_group_layout,
                &uniforms.bind_group_layout,
                &noise_bindings.bind_group_layout,
                &texture_bind_group_layout,
                &node_uniform_bind_group_layout,
                light_uniforms,
                light_textures,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/water.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/water.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("water_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[plane::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[settings::COLOR_TEXTURE_FORMAT.into()],
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

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("water_vertex_buffer"),
            contents: bytemuck::cast_slice(&tile.vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Self {
            lods,
            uniforms,
            render_pipeline,
            noise_bindings,
            reflection_texture_view,
            refraction_texture_view,
            refraction_depth_texture_view,
            texture_bind_group,
            reflection_depth_texture_view,
            vertex_buffer,
            node_uniform_bind_group_layout,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, time: Instant) {
        self.uniforms.data.time = time.elapsed().as_millis() as f32;
        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
    }
}
