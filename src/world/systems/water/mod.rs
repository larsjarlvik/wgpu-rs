use crate::{
    camera, noise, plane, settings, texture,
    world::{self, node::Node},
};
use cgmath::*;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct Water {
    pub lods: Vec<HashMap<plane::ConnectType, plane::LodBuffer>>,
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
        env: &world::enivornment::Environment,
    ) -> Self {
        let noise_bindings = noise.create_bindings(device);
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
                &texture_bind_group_layout,
                &noise_bindings.bind_group_layout,
                &node_uniform_bind_group_layout,
                &env.uniform_bind_group_layout,
                &env.texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/water.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/water.frag.spv"));
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

    pub fn get_bundle(
        &self,
        device: &wgpu::Device,
        camera: &camera::Instance,
        world_data: &world::WorldData,
        nodes: &Vec<&Node>,
    ) -> wgpu::RenderBundle {
        optick::event!();
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: Some("water_bundle"),
            color_formats: &[settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });
        encoder.set_pipeline(&self.render_pipeline);
        encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
        encoder.set_bind_group(1, &self.texture_bind_group, &[]);
        encoder.set_bind_group(2, &self.noise_bindings.bind_group, &[]);
        encoder.set_bind_group(4, &world_data.environment.uniforms.bind_group, &[]);
        encoder.set_bind_group(5, &world_data.environment.texture_bind_group, &[]);
        encoder.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        let plane = camera.uniforms.data.clip[3];
        let eye = vec3(
            camera.uniforms.data.eye_pos[0],
            camera.uniforms.data.eye_pos[1],
            camera.uniforms.data.eye_pos[2],
        );

        for node in nodes {
            for lod in 0..=settings::LODS.len() {
                let water_lod = world_data.lods.get(lod).expect("Could not get LOD!");

                if check_clip(plane, node.bounding_box.min.y)
                    && plane::get_lod(eye, vec3(node.x, 0.0, node.z), camera.uniforms.data.z_far) == lod as u32
                {
                    if let Some(data) = &node.get_data() {
                        let ct = plane::get_connect_type(eye, vec3(node.x, 0.0, node.z), lod as u32, camera.uniforms.data.z_far);
                        let lod_buffer = water_lod.get(&ct).unwrap();
                        encoder.set_bind_group(3, &data.uniforms.bind_group, &[]);
                        encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
                    }
                }
            }
        }

        encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("water") })
    }
}

fn check_clip(plane: f32, y_min: f32) -> bool {
    y_min <= plane
}
