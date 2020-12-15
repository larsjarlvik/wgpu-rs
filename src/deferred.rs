use crate::settings;

pub struct DeferredRender {
    pub render_bundle: wgpu::RenderBundle,
    pub position_texture_view: wgpu::TextureView,
    pub normals_texture_view: wgpu::TextureView,
    pub base_color_texture_view: wgpu::TextureView,
    pub depth_texture_view: wgpu::TextureView,
}

impl DeferredRender {
    pub fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor) -> Self {
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                create_bind_group_layout(0, wgpu::TextureComponentType::Float),
                create_bind_group_layout(1, wgpu::TextureComponentType::Uint),
                create_bind_group_layout(2, wgpu::TextureComponentType::Uint),
                create_bind_group_layout(3, wgpu::TextureComponentType::Uint),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
        });

        let sampler = &device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let position_texture_view = create_texture_view(&device, &swap_chain_desc, settings::COLOR_TEXTURE_FORMAT);
        let normals_texture_view = create_texture_view(&device, &swap_chain_desc, settings::COLOR_TEXTURE_FORMAT);
        let base_color_texture_view = create_texture_view(&device, &swap_chain_desc, settings::COLOR_TEXTURE_FORMAT);
        let depth_texture_view = create_texture_view(&device, &swap_chain_desc, settings::DEPTH_TEXTURE_FORMAT);

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_array"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&depth_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&position_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normals_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&base_color_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("deferred_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("./shaders-compiled/deferred.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("./shaders-compiled/deferred.frag.spv"));
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

        let render_bundle = create_bundle(&device, &render_pipeline, &texture_bind_group);

        DeferredRender {
            render_bundle,
            position_texture_view,
            normals_texture_view,
            base_color_texture_view,
            depth_texture_view,
        }
    }
}

fn create_texture_view(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor, format: wgpu::TextureFormat) -> wgpu::TextureView {
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
        format: format,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::COPY_DST,
    };
    let texture = device.create_texture(frame_descriptor);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn create_bind_group_layout(binding: u32, component_type: wgpu::TextureComponentType) -> wgpu::BindGroupLayoutEntry  {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStage::FRAGMENT,
        ty: wgpu::BindingType::SampledTexture {
            multisampled: false,
            dimension: wgpu::TextureViewDimension::D2,
            component_type,
        },
        count: None,
    }
}

pub fn create_bundle(
    device: &wgpu::Device,
    render_pipeline: &wgpu::RenderPipeline,
    texture_bind_group: &wgpu::BindGroup,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: None,
        sample_count: 1,
    });

    encoder.set_pipeline(&render_pipeline);
    encoder.set_bind_group(0, &texture_bind_group, &[]);
    encoder.draw(0..6, 0..1);
    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("models") })
}
