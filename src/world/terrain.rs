use crate::{camera, settings};
use cgmath::{InnerSpace, Vector3};
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    OpenSimplex,
};
use wgpu::util::DeviceExt;

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
    vertex_buffer: wgpu::Buffer,
    num_elements: u32,
}

pub struct Terrain {
    pub render_bundle: wgpu::RenderBundle,
    render_pipeline: wgpu::RenderPipeline,
    noise: NoiseMap,
    quads: Vec<Quad>,
}

impl Terrain {
    pub fn new(device: &wgpu::Device, camera: &camera::Camera) -> Terrain {
        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain_pipeline_layout"),
            bind_group_layouts: &[&camera.uniforms.bind_group_layout],
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
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[
                settings::COLOR_TEXTURE_FORMAT.into(),
                settings::COLOR_TEXTURE_FORMAT.into(),
                settings::COLOR_TEXTURE_FORMAT.into(),
            ],
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
            alpha_to_coverage_enabled: false,
        });

        let render_bundle = build_render_bundle(&device, &render_pipeline, &camera, &Vec::new());
        let open_simplex = OpenSimplex::new();
        let noise = PlaneMapBuilder::new(&open_simplex).build();

        Terrain {
            render_pipeline,
            render_bundle,
            noise,
            quads: Vec::new(),
        }
    }

    pub fn add_quad(&mut self, device: &wgpu::Device, camera: &camera::Camera, tx: f32, ty: f32) {
        let mut vertices = vec![];
        let size = 50;

        for y in (-size..size).map(|i| (i as f32)) {
            for x in (-size..size).map(|i| (i as f32)) {
                vertices.push(gen_vertex(&self.noise, tx + x, ty + y + 1.0));
                vertices.push(gen_vertex(&self.noise, tx + x, ty + y));
                vertices.push(gen_vertex(&self.noise, tx + x + 1.0, ty + y));
                vertices.push(gen_vertex(&self.noise, tx + x + 1.0, ty + y + 1.0));
                vertices.push(gen_vertex(&self.noise, tx + x, ty + y + 1.0));
                vertices.push(gen_vertex(&self.noise, tx + x + 1.0, ty + y));
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices.as_slice()),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let num_elements = vertices.len() as u32;

        &self.quads.push(Quad {
            vertex_buffer,
            num_elements,
        });

        self.render_bundle = build_render_bundle(&device, &self.render_pipeline, &camera, &self.quads);
    }

    pub fn get_elevation(&self, x: f32, z: f32) -> f32 {
        get_elevation(&self.noise, x, z)
    }
}

fn build_render_bundle(
    device: &wgpu::Device,
    render_pipeline: &wgpu::RenderPipeline,
    camera: &camera::Camera,
    quads: &Vec<Quad>,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[
            settings::COLOR_TEXTURE_FORMAT,
            settings::COLOR_TEXTURE_FORMAT,
            settings::COLOR_TEXTURE_FORMAT,
        ],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });
    encoder.set_pipeline(&render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);

    for quad in quads {
        encoder.set_vertex_buffer(0, quad.vertex_buffer.slice(..));
        encoder.draw(0..quad.num_elements, 0..1);
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
}

fn get_elevation(noise: &NoiseMap, x: f32, z: f32) -> f32 {
    noise.get_value(x as usize, z as usize) as f32 * 60.0
}

fn gen_vertex(noise: &NoiseMap, x: f32, z: f32) -> Vertex {
    let h_l = get_elevation(&noise, x - 1.0, z);
    let h_r = get_elevation(&noise, x + 1.0, z);
    let h_d = get_elevation(&noise, x, x - 1.0);
    let h_u = get_elevation(&noise, x, x + 1.0);

    let normals = Vector3::new(h_l - h_r, 2.0, h_d - h_u);

    Vertex {
        position: [x, get_elevation(&noise, x, z), z],
        normals: normals.normalize().into(),
    }
}
