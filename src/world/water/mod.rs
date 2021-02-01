use crate::{camera, noise, plane, settings, texture};
use std::{collections::HashMap, time::Instant};
use wgpu::util::DeviceExt;
mod uniforms;

pub struct Water {
    pub plane: plane::Plane,
    pub lods: Vec<HashMap<plane::ConnectType, plane::LodBuffer>>,
    pub uniforms: uniforms::UniformBuffer,
    pub render_pipeline: wgpu::RenderPipeline,
    pub noise_bindings: noise::NoiseBindings,
    pub refraction_texture_view: wgpu::TextureView,
    pub refraction_depth_texture_view: wgpu::TextureView,
    pub reflection_texture_view: wgpu::TextureView,
    pub reflection_depth_texture_view: wgpu::TextureView,
    pub texture_bind_group: wgpu::BindGroup,
}

impl Water {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor, camera: &camera::Camera, noise: &noise::Noise) -> Self {
        let noise_bindings = noise.create_bindings(device);
        let uniforms = uniforms::UniformBuffer::new(&device, uniforms::Uniforms { time: 0.0 });
        let plane = plane::Plane::new(settings::TILE_SIZE);
        let mut lods = vec![];

        for lod in 0..=settings::LODS.len() {
            let indices_lod = plane.create_indices(&device, lod as u32 + 1);
            lods.push(indices_lod);
        }

        // Textures
        let sampler =  texture::create_sampler(device, wgpu::AddressMode::ClampToEdge, wgpu::FilterMode::Nearest);
        let refraction_texture_view = texture::create_view(&device, &swap_chain_desc, settings::COLOR_TEXTURE_FORMAT);
        let refraction_depth_texture_view = texture::create_view(&device, &swap_chain_desc, settings::DEPTH_TEXTURE_FORMAT);
        let reflection_texture_view = texture::create_view(&device, &swap_chain_desc, settings::COLOR_TEXTURE_FORMAT);
        let reflection_depth_texture_view = texture::create_view(&device, &swap_chain_desc, settings::DEPTH_TEXTURE_FORMAT);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water_texture_bind_group_layout"),
            entries: &[
                texture::create_bind_group_layout(0, wgpu::TextureComponentType::Float),
                texture::create_bind_group_layout(1, wgpu::TextureComponentType::Uint),
                texture::create_bind_group_layout(2, wgpu::TextureComponentType::Uint),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
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

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water_pipeline_layout"),
            bind_group_layouts: &[
                &camera.uniforms.bind_group_layout,
                &uniforms.bind_group_layout,
                &noise_bindings.bind_group_layout,
                &texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("../../shaders-compiled/water.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../../shaders-compiled/water.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("water_pipeline"),
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
            color_states: &[settings::COLOR_TEXTURE_FORMAT.into()],
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

        Self {
            plane,
            lods,
            uniforms,
            render_pipeline,
            noise_bindings,
            reflection_texture_view,
            refraction_texture_view,
            refraction_depth_texture_view,
            texture_bind_group,
            reflection_depth_texture_view,
        }
    }

    pub fn create_buffer(&self, device: &wgpu::Device, x: f32, z: f32) -> wgpu::Buffer {
        let mut vertices = self.plane.vertices.clone();
        for v in vertices.iter_mut() {
            v.position[0] += x;
            v.position[2] += z;
        }

        let contents: &[u8] = bytemuck::cast_slice(&vertices);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("water_vertex_buffer"),
            contents,
            usage: wgpu::BufferUsage::VERTEX,
        })
    }

    pub fn update(&mut self, queue: &wgpu::Queue, time: Instant) {
        self.uniforms.data.time = time.elapsed().as_millis() as f32;
        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
    }
}
