use crate::{camera, settings};
mod textures;
mod uniforms;
use cgmath::*;

pub struct DeferredRender {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group: wgpu::BindGroup,
    pub target: textures::Textures,
    pub uniforms: uniforms::UniformBuffer,
    pub shadow_matrix: Vec<Matrix4<f32>>,
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

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/deferred.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders/compiled/deferred.frag.spv"));
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
                shadow_matrix: [Matrix4::identity().into(); settings::SHADOW_CASCADE_SPLITS.len()],
                shadow_split_depth: [[0.0, 0.0, 0.0, 0.0]; settings::SHADOW_CASCADE_SPLITS.len()],
            },
        );

        DeferredRender {
            render_pipeline,
            texture_bind_group,
            target,
            uniforms,
            shadow_matrix: vec![Matrix4::identity(); settings::SHADOW_CASCADE_SPLITS.len()],
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, viewport: &camera::Viewport, view: Matrix4<f32>) {
        let inv_cam = (viewport.proj * view).inverse_transform().unwrap();
        let clip_range = viewport.z_far - viewport.z_near;

        let mut last_split_dist = 0.0;
        for (i, split_dist) in settings::SHADOW_CASCADE_SPLITS.iter().enumerate() {
            #[rustfmt::skip]
            let mut frustum_corners = [
                vec3(-1.0, 1.0,-1.0),
                vec3( 1.0, 1.0,-1.0),
                vec3( 1.0,-1.0,-1.0),
                vec3(-1.0,-1.0,-1.0),
                vec3(-1.0, 1.0, 1.0),
                vec3( 1.0, 1.0, 1.0),
                vec3( 1.0,-1.0, 1.0),
                vec3(-1.0,-1.0, 1.0),
            ];

            for i in 0..8 {
                let inverted = inv_cam * frustum_corners[i].extend(1.0);
                frustum_corners[i] = inverted.truncate() / inverted.w;
            }
            for i in 0..4 {
                let dist = frustum_corners[i + 4] - frustum_corners[i];
                frustum_corners[i + 4] = frustum_corners[i] + (dist * *split_dist);
                frustum_corners[i] = frustum_corners[i] + (dist * last_split_dist);
            }

            let mut center = Vector3::zero();
            for corner in frustum_corners.iter() {
                center += *corner;
            }
            center /= 8.0;

            let mut radius = 0.0f32;
            for corner in frustum_corners.iter() {
                radius = radius.max(corner.distance(center));
            }
            radius = (radius * 16.0).ceil() / 16.0;

            let light_dir = settings::LIGHT_DIR;
            let light_view_matrix = Matrix4::look_at_rh(
                Point3::from_vec(center - light_dir * radius),
                Point3::from_vec(center),
                Vector3::unit_y(),
            );

            let r2 = radius * 2.0;
            let mut light_ortho_matrix = Matrix4::zero();
            light_ortho_matrix[0][0] = 2.0 / r2;
            light_ortho_matrix[1][1] = 2.0 / r2;
            light_ortho_matrix[2][2] = -1.0 / r2;
            light_ortho_matrix[3][3] = 1.0;

            let shadow_matrix = light_ortho_matrix * light_view_matrix;
            self.shadow_matrix[i] = shadow_matrix;
            self.uniforms.data.shadow_matrix[i] = shadow_matrix.into();
            self.uniforms.data.shadow_split_depth[i] = [viewport.z_near + *split_dist * clip_range, 0.0, 0.0, 0.0];
            last_split_dist = *split_dist;
        }

        queue.write_buffer(&self.uniforms.buffer, 0, bytemuck::cast_slice(&[self.uniforms.data]));
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
