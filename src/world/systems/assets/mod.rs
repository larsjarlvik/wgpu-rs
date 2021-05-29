use crate::{
    camera, noise, settings, texture,
    world::{self, node::Node},
};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::{collections::HashMap, time::Instant};
use wgpu::util::DeviceExt;
mod assets;
mod data;
mod uniforms;
pub use {self::assets::Asset, self::assets::AssetMap, self::data::Instance, self::data::Vertex};

pub struct InstanceBuffer {
    pub buffer: wgpu::Buffer,
    pub length: u32,
}

pub type InstanceBufferMap = HashMap<String, InstanceBuffer>;

pub struct Assets {
    pub sampler: wgpu::Sampler,
    pub render_pipeline: wgpu::RenderPipeline,
    pub shadow_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub assets: assets::AssetMap,
    noise_bindings: noise::NoiseBindings,
}

impl Assets {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: &camera::Viewport,
        noise: &noise::Noise,
        env: &world::enivornment::Environment,
    ) -> Self {
        let noise_bindings = noise.create_bindings(device);

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

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
            label: Some("asset_pipeline_layout"),
            bind_group_layouts: &[
                &uniform_bind_group_layout,
                &texture_bind_group_layout,
                &viewport.bind_group_layout,
                &noise_bindings.bind_group_layout,
                &env.uniform_bind_group_layout,
                &env.texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/assets.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/assets.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("asset_pipeline"),
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

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/assets-shadows.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/assets-shadows.frag.spv"));
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("asset_shadow_pipeline"),
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
        let assets = assets::create(device, queue, &uniform_bind_group_layout, &texture_bind_group_layout, &sampler);
        println!("Assets: {} ms", now.elapsed().as_millis());

        Self {
            sampler,
            render_pipeline,
            shadow_pipeline,
            texture_bind_group_layout,
            noise_bindings,
            assets,
        }
    }

    pub fn get_bundle(
        &self,
        device: &wgpu::Device,
        camera: &camera::Instance,
        world_data: &world::WorldData,
        asset_instances: &mut InstanceBufferMap,
        nodes: &Vec<&Node>,
    ) -> wgpu::RenderBundle {
        optick::event!();
        update_instance_buffer(device, asset_instances, nodes);

        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });

        encoder.set_pipeline(&self.render_pipeline);
        encoder.set_bind_group(2, &camera.uniforms.bind_group, &[]);
        encoder.set_bind_group(3, &self.noise_bindings.bind_group, &[]);
        encoder.set_bind_group(4, &world_data.environment.uniforms.bind_group, &[]);
        encoder.set_bind_group(5, &world_data.environment.texture_bind_group, &[]);

        for (key, asset_instances) in asset_instances.iter_mut().filter(|(_, val)| val.length > 0) {
            let asset = self.assets.get(key).unwrap();
            encoder.set_vertex_buffer(1, asset_instances.buffer.slice(..));

            for mesh in &asset.primitives {
                encoder.set_bind_group(0, &mesh.uniforms.bind_group, &[]);
                encoder.set_bind_group(1, &mesh.texture_bind_group, &[]);
                encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                encoder.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                encoder.draw_indexed(0..mesh.num_elements, 0, 0..asset_instances.length as _);
            }
        }

        encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("assets") })
    }

    pub fn get_shadow_bundle(
        &self,
        device: &wgpu::Device,
        camera: &camera::Instance,
        world_data: &world::WorldData,
        asset_instances: &mut InstanceBufferMap,
        nodes: &Vec<&Node>,
    ) -> wgpu::RenderBundle {
        optick::event!();
        update_instance_buffer(device, asset_instances, nodes);

        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });

        encoder.set_pipeline(&self.shadow_pipeline);
        encoder.set_bind_group(2, &camera.uniforms.bind_group, &[]);
        encoder.set_bind_group(3, &self.noise_bindings.bind_group, &[]);
        encoder.set_bind_group(4, &world_data.environment.uniforms.bind_group, &[]);

        for (key, asset_instances) in asset_instances.iter_mut().filter(|(_, val)| val.length > 0) {
            let asset = self.assets.get(key).unwrap();
            encoder.set_vertex_buffer(1, asset_instances.buffer.slice(..));

            for mesh in &asset.primitives {
                encoder.set_bind_group(0, &mesh.uniforms.bind_group, &[]);
                encoder.set_bind_group(1, &mesh.texture_bind_group, &[]);
                encoder.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                encoder.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                encoder.draw_indexed(0..mesh.num_elements, 0, 0..asset_instances.length as _);
            }
        }

        encoder.finish(&wgpu::RenderBundleDescriptor {
            label: Some("assets_shadow"),
        })
    }

    pub fn get_instances(&self, device: &wgpu::Device) -> InstanceBufferMap {
        let mut asset_instances = HashMap::new();

        for (key, _) in self.assets.iter() {
            let instances: Vec<Instance> = vec![];
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance_buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsage::VERTEX,
            });
            asset_instances.insert(key.clone(), InstanceBuffer { buffer, length: 0 });
        }

        asset_instances
    }
}

fn update_instance_buffer(device: &wgpu::Device, asset_instances: &mut InstanceBufferMap, nodes: &Vec<&Node>) {
    asset_instances.par_iter_mut().for_each(|(key, instance)| {
        let instances = nodes
            .iter()
            .filter_map(|n| n.get_data())
            .filter_map(|d| d.asset_instances.get(key))
            .flat_map(|mi| mi.clone())
            .collect::<Vec<Instance>>();

        instance.length = instances.len() as u32;
        if instance.length > 0 {
            instance.buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance_buffer"),
                contents: bytemuck::cast_slice(&instances.as_slice()),
                usage: wgpu::BufferUsage::VERTEX,
            });
        }
    });
}
