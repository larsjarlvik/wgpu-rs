use super::noise;
use std::mem;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub bitangent: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: mem::size_of::<[f32; 9]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float3,
                },
            ],
        }
    }
}

pub struct TerrainTile {
    pub render_buffer: wgpu::Buffer,
}

pub struct Compute {
    pub num_elements: u32,
    vertices: Vec<Vertex>,
    compute_pipeline: wgpu::ComputePipeline,
    vertex_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    src_vertex_buffer: wgpu::Buffer,
    tile_size: f32,
    noise_bindings: noise::NoiseBindings,
}

impl Compute {
    pub fn new(device: &wgpu::Device, noise: &noise::Noise, tile_size: f32) -> Self {
        let vertices = create_vertices(tile_size);
        let num_elements = vertices.len() as u32;
        let noise_bindings = noise.create_bindings(device);

        let vertex_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("input_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: true,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::StorageBuffer {
                        dynamic: false,
                        min_binding_size: None,
                        readonly: false,
                    },
                    count: None,
                },
            ],
        });

        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::COMPUTE,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain_compute_layout"),
            bind_group_layouts: &[
                &vertex_bind_group_layout,
                &uniform_bind_group_layout,
                &noise_bindings.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let module = device.create_shader_module(wgpu::include_spirv!("../shaders-compiled/terrain.comp.spv"));
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("terrain_compute_pipeline"),
            layout: Some(&layout),
            compute_stage: wgpu::ProgrammableStageDescriptor {
                module: &module,
                entry_point: "main",
            },
        });

        let src_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("input_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::STORAGE,
        });

        Self {
            compute_pipeline,
            num_elements,
            vertex_bind_group_layout,
            uniform_bind_group_layout,
            src_vertex_buffer,
            vertices,
            tile_size,
            noise_bindings,
        }
    }

    pub fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, x: f32, z: f32) -> wgpu::Buffer {
        let contents: &[u8] = bytemuck::cast_slice(&self.vertices);
        let dst_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("output_vertex_buffer"),
            contents,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::MAP_READ,
        });

        let vertex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.vertex_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(self.src_vertex_buffer.slice(..)),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(dst_vertex_buffer.slice(..)),
                },
            ],
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::cast_slice(&[x * self.tile_size, z * self.tile_size]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buffer.slice(..)),
            }],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("heightmap_calc"),
        });
        {
            let mut pass = encoder.begin_compute_pass();
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &vertex_bind_group, &[]);
            pass.set_bind_group(1, &uniform_bind_group, &[]);
            pass.set_bind_group(2, &self.noise_bindings.bind_group, &[]);
            pass.dispatch(self.num_elements, 1, 1);
        }
        queue.submit(std::iter::once(encoder.finish()));
        dst_vertex_buffer
    }
}

fn create_vertex(x: f32, z: f32) -> Vertex {
    Vertex {
        position: [x, 0.0, z],
        normal: [0.0, 0.0, 0.0],
        tangent: [0.0, 0.0, 0.0],
        bitangent: [0.0, 0.0, 0.0],
    }
}

fn create_vertices(tile_size: f32) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let half_tile_size = (tile_size / 2.0) as i32;
    for z in (-half_tile_size..half_tile_size).map(|i| (i as f32)) {
        for x in (-half_tile_size..half_tile_size).map(|i| (i as f32)) {
            vertices.push(create_vertex(x, z + 1.0));
            vertices.push(create_vertex(x + 1.0, z));
            vertices.push(create_vertex(x, z));
            vertices.push(create_vertex(x + 1.0, z + 1.0));
            vertices.push(create_vertex(x + 1.0, z));
            vertices.push(create_vertex(x, z + 1.0));
        }
    }
    vertices
}
