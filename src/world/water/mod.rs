use crate::{camera, plane, settings};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct Water {
    pub plane: plane::Plane,
    pub lods: Vec<HashMap<plane::ConnectType, plane::LodBuffer>>,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Water {
    pub fn new(device: &wgpu::Device, camera: &camera::Camera) -> Self {
        let plane = plane::Plane::new(settings::TILE_SIZE);
        let mut lods = vec![];

        for lod in 0..=settings::LODS.len() {
            let indices_lod = plane.create_indices(&device, lod as u32 + 1);
            lods.push(indices_lod);
        }

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water_pipeline_layout"),
            bind_group_layouts: &[&camera.uniforms.bind_group_layout],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(wgpu::include_spirv!("../../shaders-compiled/water.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("../../shaders-compiled/water.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("water_pipeline"),
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
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[settings::COLOR_TEXTURE_FORMAT.into(), settings::COLOR_TEXTURE_FORMAT.into()],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: settings::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[plane::Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Self {
            plane,
            lods,
            render_pipeline,
        }
    }

    pub fn create_buffer(&self, device: &wgpu::Device, x: f32, z: f32) -> wgpu::Buffer {
        let mut vertices = self.plane.vertices.clone();
        for v in vertices.iter_mut() {
            v.position[0] += x;
            v.position[2] += z;
        }

        let contents: &[u8] = bytemuck::cast_slice(&vertices);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("water_vertex_buffer"),
            contents,
            usage: wgpu::BufferUsage::VERTEX,
        })
    }
}
