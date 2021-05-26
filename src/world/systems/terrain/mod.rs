use crate::{
    camera, noise, plane, settings, texture,
    world::{self, node::Node},
};
use cgmath::*;
use image::GenericImageView;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::convert::TryInto;
use wgpu::util::DeviceExt;

pub struct Terrain {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group: wgpu::BindGroup,
    pub noise_bindings: noise::NoiseBindings,
    pub vertex_buffer: wgpu::Buffer,
    pub node_uniform_bind_group_layout: wgpu::BindGroupLayout,
}

impl Terrain {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport: &camera::Viewport,
        noise: &noise::Noise,
        tile: &plane::Plane,
        map: &world::map::Map,
        lights: &world::lights::Lights,
    ) -> Terrain {
        let noise_bindings = noise.create_bindings(device);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                texture::create_array_bind_group_layout(0, wgpu::TextureSampleType::Uint, 26),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: false,
                    },
                    count: None,
                },
            ],
        });

        let node_uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain_pipeline_layout"),
            bind_group_layouts: &[
                &viewport.bind_group_layout,
                &texture_bind_group_layout,
                &noise_bindings.bind_group_layout,
                &node_uniform_bind_group_layout,
                &map.textures.bind_group_layout,
                &lights.uniform_bind_group_layout,
                &lights.texture_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/terrain.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("../../../shaders/compiled/terrain.frag.spv"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[plane::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[settings::COLOR_TEXTURE_FORMAT.into()],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                topology: wgpu::PrimitiveTopology::TriangleStrip,
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

        let texture_bind_group = build_textures(device, queue, &texture_bind_group_layout);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain_vertex_buffer"),
            contents: bytemuck::cast_slice(&tile.vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Terrain {
            texture_bind_group,
            render_pipeline,
            noise_bindings,
            vertex_buffer,
            node_uniform_bind_group_layout,
        }
    }

    pub fn get_bundle(
        &self,
        device: &wgpu::Device,
        camera: &camera::Instance,
        world_data: &world::WorldData,
        nodes: &Vec<&Node>,
    ) -> wgpu::RenderBundle {
        optick::event!();
        let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: Some("terrain_bundle"),
            color_formats: &[settings::COLOR_TEXTURE_FORMAT],
            depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
            sample_count: 1,
        });
        encoder.set_pipeline(&self.render_pipeline);
        encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
        encoder.set_bind_group(1, &self.texture_bind_group, &[]);
        encoder.set_bind_group(2, &self.noise_bindings.bind_group, &[]);
        encoder.set_bind_group(4, &world_data.map.textures.bind_group, &[]);
        encoder.set_bind_group(5, &world_data.lights.uniforms.bind_group, &[]);
        encoder.set_bind_group(6, &world_data.lights.texture_bind_group, &[]);
        encoder.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        let direction = camera.uniforms.data.clip[1];
        let plane = camera.uniforms.data.clip[3];
        let eye = vec3(
            camera.uniforms.data.eye_pos[0],
            camera.uniforms.data.eye_pos[1],
            camera.uniforms.data.eye_pos[2],
        );

        for node in nodes {
            for lod in 0..=settings::LODS.len() {
                let terrain_lod = world_data.lods.get(lod).expect("Could not get LOD!");

                if check_clip(direction, plane, &node.bounding_box)
                    && plane::get_lod(eye, vec3(node.x, 0.0, node.z), camera.uniforms.data.z_far) == lod as u32
                {
                    if let Some(data) = &node.get_data() {
                        let ct = plane::get_connect_type(eye, vec3(node.x, 0.0, node.z), lod as u32, camera.uniforms.data.z_far);
                        let lod_buffer = terrain_lod.get(&ct).unwrap();

                        encoder.set_bind_group(3, &data.uniforms.bind_group, &[]);
                        encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
                    }
                }
            }
        }

        encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
    }
}

fn check_clip(direction: f32, plane: f32, bounding_box: &camera::BoundingBox) -> bool {
    direction == 0.0 || (direction > 0.0 && bounding_box.max.y >= plane) || (direction <= 0.0 && bounding_box.min.y < plane)
}

fn load_texture(device: &wgpu::Device, queue: &wgpu::Queue, path: &str) -> wgpu::TextureView {
    let i1 = image::open(format!("res/textures/{}.png", path)).unwrap();
    texture::create_mipmapped_view(
        device,
        queue,
        &i1.as_rgba8().unwrap().to_vec(),
        i1.dimensions().0,
        i1.dimensions().1,
    )
}

fn build_textures(device: &wgpu::Device, queue: &wgpu::Queue, texture_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
    let sampler = texture::create_sampler(device, wgpu::AddressMode::Repeat, wgpu::FilterMode::Linear);

    // TODO: Use 3D texture
    let paths = vec![
        "desert",
        "grassland",
        "snow",
        "tropical",
        "tundra",
        "barren",
        "rock",
        "cliff",
        "sand",
        "flowers",
        "mud",
        "pebbles_grass",
        "pebbles_desert",
    ];
    let textures = paths
        .par_iter()
        .flat_map(|&t| {
            vec![
                load_texture(device, queue, t),
                load_texture(device, queue, &format!("{}_normals", t)),
            ]
        })
        .collect::<Vec<wgpu::TextureView>>();
    let t: &[&wgpu::TextureView; 26] = &textures.iter().collect::<Vec<_>>().try_into().unwrap();

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("terrain_textures"),
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(t),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}
