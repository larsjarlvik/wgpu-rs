use wgpu::util::DeviceExt;
use cgmath::SquareMatrix;
use crate::settings;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normals: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct Quad {
    render_bundle: wgpu::RenderBundle,
}

pub struct Terrain {
    pub uniforms: UniformBuffer,
    pub render_pipeline: wgpu::RenderPipeline,
    pub quads: Vec<Quad>,
}

impl Terrain {
    pub fn new(device: &wgpu::Device) -> Terrain {
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain_pipeline_layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("../shaders-compiled/terrain.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../shaders-compiled/terrain.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain_pipeline"),
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
            color_states: &[settings::COLOR_TEXTURE_FORMAT.into(), settings::COLOR_TEXTURE_FORMAT.into(), settings::COLOR_TEXTURE_FORMAT.into()],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: settings::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: true,
        });

        let uniforms = UniformBuffer::new(
            &device,
            &uniform_bind_group_layout,
            Uniforms {
                view_proj: cgmath::Matrix4::identity().into(),
            },
        );

        Terrain {
            render_pipeline,
            uniforms,
            quads: Vec::new(),
        }
    }

    pub fn add_quad(&mut self, device: &wgpu::Device, tx: f32, ty: f32) {
        let mut vertices = vec![];
        let size = 50;

        for y in (-size..size - 1).map(|i| (i as f32)) {
            for x in (-size..size - 1).map(|i| (i as f32)) {
                vertices.push(gen_vertex(tx + x, ty + y + 1.0));
                vertices.push(gen_vertex(tx + x, ty + y));
                vertices.push(gen_vertex(tx + x + 1.0, ty + y));
                vertices.push(gen_vertex(tx + x + 1.0, ty + y + 1.0));
                vertices.push(gen_vertex(tx + x, ty + y + 1.0));
                vertices.push(gen_vertex(tx + x, ty + y));
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices.as_slice()),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let num_elements = vertices.len() as u32;

        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: None,
            color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });
        encoder.set_pipeline(&self.render_pipeline);
        encoder.set_bind_group(0, &self.uniforms.bind_group, &[]);
        encoder.set_vertex_buffer(0, vertex_buffer.slice(..));
        encoder.draw(0..num_elements, 0..1);
        let render_bundle = encoder.finish(&wgpu::RenderBundleDescriptor {
            label: Some("terrain"),
        });

        &self.quads.push(Quad {
            render_bundle,
        });
    }

    pub fn get_render_bundles(&self) -> Vec<&wgpu::RenderBundle> {
        let mut ret = Vec::new();
        for i in &self.quads{
            ret.push(&i.render_bundle);
        }
        ret
    }
}

fn gen_vertex(x: f32, z: f32) -> Vertex {
    Vertex {
        position: [x, 0.0, z],
        normals: [0.0, 1.0, 0.0],
    }
}
