use crate::{camera, settings};
mod data;

pub struct Sky {
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniforms: data::UniformBuffer,
}

impl Sky {
    pub fn new(device: &wgpu::Device, viewport: &camera::Viewport) -> Self {
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
            label: Some("sky_pipeline_layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &viewport.bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/sky.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/sky.frag.spv"));
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

        Self { render_pipeline, uniforms }
    }

    pub fn get_bundle(&self, device: &wgpu::Device, camera: &camera::Instance) -> wgpu::RenderBundle {
        optick::event!();
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: None,
            sample_count: 1,
        });

        encoder.set_pipeline(&self.render_pipeline);
        encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
        encoder.set_bind_group(1, &self.uniforms.bind_group, &[]);
        encoder.draw(0..6, 0..1);

        encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("sky") })
    }
}
