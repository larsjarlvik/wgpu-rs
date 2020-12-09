use crate::texture;
use crate::vertex;
use wgpu::util::DeviceExt;
use cgmath::SquareMatrix;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn set(&mut self, view_proj: [[f32; 4]; 4]) {
        self.view_proj = view_proj;
    }
}

pub struct RenderPipeline {
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniforms: Uniforms,
    pub uniform_buffer: wgpu::Buffer,
    pub texture_target: Target,
    pub uniform_target: Target,
}

pub struct Target {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl RenderPipeline {
    pub fn new(device: &wgpu::Device, diffuse_texture: &texture::Texture) -> Self {
        let uniforms = Uniforms::new();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let texture_target = create_texture_target(&device, &diffuse_texture);
        let uniform_target = create_uniform_target(&device, &uniform_buffer);
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_target.bind_group_layout, &uniform_target.bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("shaders-compiled/default.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shaders-compiled/default.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
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
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[vertex::Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        RenderPipeline {
            render_pipeline,
            texture_target,
            uniform_target,
            uniforms,
            uniform_buffer,
        }
    }
}

fn create_uniform_target(device: &wgpu::Device, uniform_buffer: &wgpu::Buffer) -> Target {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStage::VERTEX,
            ty: wgpu::BindingType::UniformBuffer {
                dynamic: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("uniform_bind_group_layout"),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
        }],
        label: Some("uniform_bind_group"),
    });

    Target {
        bind_group_layout,
        bind_group,
    }
}

fn create_texture_target(device: &wgpu::Device, diffuse_texture: &texture::Texture) -> Target {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    dimension: wgpu::TextureViewDimension::D2,
                    component_type: wgpu::TextureComponentType::Uint,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    });

    Target {
        bind_group_layout,
        bind_group,
    }
}
