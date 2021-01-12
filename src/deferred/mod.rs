use crate::{camera, settings};
use std::time;
pub mod textures;
mod uniforms;
mod water;

pub struct DeferredRender {
    pub render_bundle: wgpu::RenderBundle,
    pub target: textures::Textures,
    pub uniforms: uniforms::UniformBuffer,
    pub time: time::Instant,
}

impl DeferredRender {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, swap_chain_desc: &wgpu::SwapChainDescriptor, camera: &camera::Camera) -> Self {
        // Water
        let water = water::Water::new(device, queue);
        let water_texture_bind_group_layout = water.create_bind_group_layout(device);
        let water_texture_bind_group = water.create_bind_group(device, &water_texture_bind_group_layout);

        // Textures
        let target = textures::Textures::new(device, swap_chain_desc);
        let texture_bind_group_layout = target.create_bind_group_layout(device);
        let texture_bind_group = target.create_bind_group(device, &texture_bind_group_layout);

        // Uniforms
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("deferred_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &water_texture_bind_group_layout,
                &uniform_bind_group_layout,
                &camera.uniforms.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("../shaders-compiled/deferred.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../shaders-compiled/deferred.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("deferred_pipeline"),
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
            color_states: &[settings::COLOR_TEXTURE_FORMAT.into()],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let uniforms = uniforms::UniformBuffer::new(
            &device,
            &uniform_bind_group_layout,
            uniforms::Uniforms {
                light_dir: [0.5, -1.0, 0.0],
                light_color: [1.0, 0.9, 0.5],
                ambient_strength: 0.3,
                light_intensity: 2.0,
                time: 0.0,
            },
        );

        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: None,
            sample_count: 1,
        });

        encoder.set_pipeline(&render_pipeline);
        encoder.set_bind_group(0, &texture_bind_group, &[]);
        encoder.set_bind_group(1, &water_texture_bind_group, &[]);
        encoder.set_bind_group(2, &uniforms.bind_group, &[]);
        encoder.set_bind_group(3, &camera.uniforms.bind_group, &[]);
        encoder.draw(0..6, 0..1);
        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("models") });

        DeferredRender {
            render_bundle,
            target,
            uniforms,
            time: time::Instant::now(),
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        self.uniforms.data.time = self.time.elapsed().as_millis() as f32;
        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
    }
}
