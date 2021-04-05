use crate::{camera, settings};
mod textures;
mod uniforms;
use cgmath::*;

pub struct DeferredRender {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group: wgpu::BindGroup,
    pub target: textures::Textures,
    pub uniforms: uniforms::UniformBuffer,
    pub shadow_matrix: Matrix4<f32>,
}

impl DeferredRender {
    pub fn new(device: &wgpu::Device, viewport: &camera::Viewport) -> Self {
        // Textures
        let target = textures::Textures::new(device, viewport.width, viewport.height);
        let texture_bind_group_layout = target.create_bind_group_layout(device);
        let texture_bind_group = target.create_bind_group(device, &texture_bind_group_layout);

        // Uniforms
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

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("deferred_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout, &viewport.bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/deferred.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/deferred.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("deferred_pipeline"),
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

        let uniforms = uniforms::UniformBuffer::new(
            &device,
            &uniform_bind_group_layout,
            uniforms::Uniforms {
                light_dir: [0.5, -1.0, 0.0],
                light_color: [1.0, 0.9, 0.5],
                ambient_strength: 0.3,
                light_intensity: 2.0,
                shadow_matrix: Matrix4::identity().into(),
            },
        );

        DeferredRender {
            render_pipeline,
            texture_bind_group,
            target,
            uniforms,
            shadow_matrix: Matrix4::identity(),
        }
    }

    pub fn render_to(&self, encoder: &mut wgpu::CommandEncoder, bundles: Vec<&wgpu::RenderBundle>) {
        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            store: true,
        };

        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("deferred_render_pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.target.normals_texture_view,
                        resolve_target: None,
                        ops,
                    },
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &self.target.base_color_texture_view,
                        resolve_target: None,
                        ops,
                    },
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.target.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            })
            .execute_bundles(bundles.into_iter());
    }

    pub fn update(&mut self, queue: &wgpu::Queue, eye: &camera::Instance) {
        let corners = eye.frustum.get_frustum_corners();
        let mut min = vec3(0.0, 0.0, 0.0);
        let mut max = vec3(0.0, 0.0, 0.0);
        let mut first = true;

        for corner in corners.iter() {
            if first {
                min.x = corner.x;
                max.x = corner.x;
                min.y = corner.y;
                max.y = corner.y;
                min.z = corner.z;
                max.z = corner.z;
                first = false;
                continue;
            }

            min.x = min.x.min(corner.x);
            max.x = max.x.max(corner.x);
            min.y = min.y.min(corner.y);
            max.y = max.y.max(corner.y);
            min.z = min.z.min(corner.z);
            max.z = max.z.max(corner.z);
        }

        let view = Matrix4::look_at_rh(Point3::origin(), Point3::from_vec(settings::LIGHT_DIR), Vector3::unit_y());
        let proj = ortho(min.x, max.x, min.y, max.y, min.z, max.z);
        self.shadow_matrix = proj * view;
        self.uniforms.data.shadow_matrix = self.shadow_matrix.into();
        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_target: &wgpu::TextureView,
        depth_target: &wgpu::TextureView,
        render_bundle: &wgpu::RenderBundle,
    ) {
        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: color_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: depth_target,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            })
            .execute_bundles(std::iter::once(render_bundle));
    }

    pub fn get_render_bundle(&self, device: &wgpu::Device, camera: &camera::Instance, bundle_name: &str) -> wgpu::RenderBundle {
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });

        encoder.set_pipeline(&self.render_pipeline);
        encoder.set_bind_group(0, &self.texture_bind_group, &[]);
        encoder.set_bind_group(1, &self.uniforms.bind_group, &[]);
        encoder.set_bind_group(2, &camera.uniforms.bind_group, &[]);
        encoder.draw(0..6, 0..1);
        encoder.finish(&wgpu::RenderBundleDescriptor {
            label: Some(format!("deferred_{}", bundle_name).as_str()),
        })
    }
}
