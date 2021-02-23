use crate::{camera, settings, texture};
mod data;

pub struct Sky {
    pub render_bundle: wgpu::RenderBundle,
    pub texture_view: wgpu::TextureView,
    pub depth_texture_view: wgpu::TextureView,
}

impl Sky {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor, cameras: &camera::Cameras) -> Self {
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

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                texture::create_bind_group_layout(0, wgpu::TextureSampleType::Depth),
                texture::create_bind_group_layout(1, wgpu::TextureSampleType::Uint),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false, filtering: false },
                    count: None,
                },
            ],
        });

        let texture_extent = wgpu::Extent3d {
            width: swap_chain_desc.width,
            height: swap_chain_desc.height,
            depth: 1,
        };
        let frame_descriptor = &wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: settings::COLOR_TEXTURE_FORMAT,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::RENDER_ATTACHMENT | wgpu::TextureUsage::COPY_DST,
        };
        let texture = device.create_texture(frame_descriptor);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_texture_view = texture::create_view(&device, &swap_chain_desc, settings::DEPTH_TEXTURE_FORMAT);
        let sampler = texture::create_sampler(device, wgpu::AddressMode::ClampToEdge, wgpu::FilterMode::Linear);
        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                texture::create_bind_group_entry(0, &depth_texture_view),
                texture::create_bind_group_entry(1, &texture_view),
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout, &cameras.bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/sky.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../shaders-compiled/sky.frag.spv"));
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
                light_dir: [0.5, -1.0, 0.0],
            },
        );

        let render_bundle = create_bundle(&device, &render_pipeline, &texture_bind_group, &uniforms.bind_group, &cameras.eye_cam);

        Self {
            render_bundle,
            texture_view,
            depth_texture_view,
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            })
            .execute_bundles(std::iter::once(&self.render_bundle));
    }
}

pub fn create_bundle(
    device: &wgpu::Device,
    render_pipeline: &wgpu::RenderPipeline,
    texture_bind_group: &wgpu::BindGroup,
    uniform_bind_group: &wgpu::BindGroup,
    camera: &camera::camera::Camera,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: None,
        sample_count: 1,
    });

    encoder.set_pipeline(&render_pipeline);
    encoder.set_bind_group(0, &texture_bind_group, &[]);
    encoder.set_bind_group(1, &uniform_bind_group, &[]);
    encoder.set_bind_group(2, &camera.uniforms.bind_group, &[]);
    encoder.draw(0..6, 0..1);
    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("sky") })
}
