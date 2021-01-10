use crate::{camera, deferred, settings};
use cgmath::*;
mod uniforms;

pub struct Water {
    pub render_bundle: wgpu::RenderBundle,
    pub textures: deferred::textures::Textures,
    pub uniforms: uniforms::UniformBuffer,
}

impl Water {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor, camera: &camera::Camera) -> Self {
        // Uniforms
        let uniforms = uniforms::UniformBuffer::new(
            device,
            uniforms::Uniforms {
                inverse_view_proj: Matrix4::identity().into(),
            },
        );

        // Textures
        let textures = deferred::textures::Textures::new(device, swap_chain_desc);
        let texture_bind_group_layout = textures.create_bind_group_layout(device);
        let texture_bind_group = textures.create_bind_group(device, &texture_bind_group_layout);

        // Sampler
        let sampler = deferred::sampler::Sampler::new(device);
        let sampler_bind_group_layout = sampler.create_bind_group_layout(device);
        let sampler_bind_group = sampler.create_bind_group(device, &sampler_bind_group_layout);

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &sampler_bind_group_layout,
                &camera.uniforms.bind_group_layout,
                &uniforms.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("../shaders-compiled/water.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../shaders-compiled/water.frag.spv"));
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
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

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
        encoder.set_bind_group(0, &texture_bind_group, &[]);
        encoder.set_bind_group(1, &sampler_bind_group, &[]);
        encoder.set_bind_group(2, &camera.uniforms.bind_group, &[]);
        encoder.set_bind_group(3, &uniforms.bind_group, &[]);
        encoder.draw(0..6, 0..1);
        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("water") });

        Self {
            render_bundle,
            textures,
            uniforms,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, camera: &camera::Camera) {
        match (camera.proj * camera.view).invert() {
            Some(i) => {
                self.uniforms.data.inverse_view_proj = (settings::OPENGL_TO_WGPU_MATRIX * i).into();
                queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
            }
            None => {}
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &deferred::textures::Textures) {
        let bundles = vec![&self.render_bundle];
        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            store: true,
        };

        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &target.position_texture_view,
                        resolve_target: None,
                        ops,
                    },
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &target.normals_texture_view,
                        resolve_target: None,
                        ops,
                    },
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &target.base_color_texture_view,
                        resolve_target: None,
                        ops,
                    },
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &target.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            })
            .execute_bundles(bundles.into_iter());
    }
}
