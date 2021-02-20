use crate::{camera, settings};
pub mod textures;
mod uniforms;

pub struct DeferredRender {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group: wgpu::BindGroup,
    pub target: textures::Textures,
    pub uniforms: uniforms::UniformBuffer,
}

impl DeferredRender {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor, cameras: &camera::Cameras) -> Self {
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
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout, &cameras.bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../shaders-compiled/deferred.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../shaders-compiled/deferred.frag.spv"));
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
            },
        );

        DeferredRender {
            render_pipeline,
            texture_bind_group,
            target,
            uniforms,
        }
    }

    pub fn get_render_bundle(&self, device: &wgpu::Device, camera: &camera::Camera) -> wgpu::RenderBundle {
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
        encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("deferred") })
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
}
