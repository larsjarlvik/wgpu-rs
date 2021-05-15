use crate::{camera, settings, texture};
mod uniforms;
use cgmath::*;
use std::convert::TryInto;

pub struct Lights {
    pub texture_bind_group: wgpu::BindGroup,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub uniforms: uniforms::UniformBuffer,
    pub shadow_matrix: Vec<Matrix4<f32>>,
    pub shadow_texture_view: Vec<wgpu::TextureView>,
}

impl Lights {
    pub fn new(device: &wgpu::Device) -> Self {
        // Textures
        let shadow_sampler = texture::create_shadow_sampler(device);
        let shadow_texture_view: Vec<wgpu::TextureView> = (0..settings::SHADOW_CASCADE_SPLITS.len())
            .map(|_| {
                texture::create_view(
                    &device,
                    settings::SHADOW_RESOLUTION,
                    settings::SHADOW_RESOLUTION,
                    settings::DEPTH_TEXTURE_FORMAT,
                )
            })
            .collect();

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("deferred_textures_bind_group_layout"),
            entries: &[
                texture::create_array_bind_group_layout(0, wgpu::TextureSampleType::Depth, settings::SHADOW_CASCADE_SPLITS.len()),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: true,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        });

        let t: &[&wgpu::TextureView; settings::SHADOW_CASCADE_SPLITS.len()] =
            &shadow_texture_view.iter().collect::<Vec<_>>().try_into().unwrap();

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("deferred_textures"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(t),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
            ],
        });

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

        let uniforms = uniforms::UniformBuffer::new(
            &device,
            &uniform_bind_group_layout,
            uniforms::Uniforms {
                light_dir: settings::LIGHT_DIR.into(),
                light_color: settings::LIGHT_COLOR.into(),
                ambient_strength: settings::LIGHT_AMBIENT,
                light_intensity: settings::LIGHT_INTENSITY,
                shadow_matrix: [Matrix4::identity().into(); settings::SHADOW_CASCADE_SPLITS.len()],
                shadow_split_depth: [[0.0, 0.0, 0.0, 0.0]; settings::SHADOW_CASCADE_SPLITS.len()],
            },
        );

        Lights {
            texture_bind_group,
            texture_bind_group_layout,
            uniform_bind_group_layout,
            uniforms,
            shadow_texture_view,
            shadow_matrix: vec![Matrix4::identity(); settings::SHADOW_CASCADE_SPLITS.len()],
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, viewport: &camera::Viewport, view: Matrix4<f32>) {
        let inv_cam = (viewport.proj * view).inverse_transform().unwrap();
        let clip_range = (viewport.z_far * 0.65) - viewport.z_near;

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
}
