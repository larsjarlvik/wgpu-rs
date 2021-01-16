use crate::{camera, deferred, models, noise, settings};
use cgmath::{vec2, Vector2};
mod assets;
mod node;
mod terrain;

pub struct WorldData {
    terrain: terrain::Terrain,
    noise: noise::Noise,
    pub models: models::Models,
}

pub struct World {
    root_node: node::Node,
    pub data: WorldData,
    pub terrain_bundle: wgpu::RenderBundle,
}

impl World {
    pub async fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera) -> Self {
        let noise = noise::Noise::new(&device, &queue).await;
        let mut models = models::Models::new(&device, &camera);
        for asset in assets::ASSETS {
            models.load_model(&device, &queue, asset.name, format!("{}.glb", asset.name).as_str());
        }

        let mut root_node = node::Node::new(0.0, 0.0, settings::TILE_DEPTH);
        let mut terrain = terrain::Terrain::new(device, queue, camera, &noise);
        let terrain_bundle = get_terrain_bundle(device, camera, &mut terrain, &mut root_node);

        let mut world = Self {
            data: WorldData { terrain, noise, models },
            terrain_bundle,
            root_node,
        };

        world.terrain_bundle = get_terrain_bundle(device, camera, &mut world.data.terrain, &mut world.root_node);
        world
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera) {
        self.root_node.update(device, queue, &mut self.data, camera);
        self.data.models.refresh_render_bundle(device, camera);
        self.terrain_bundle = get_terrain_bundle(device, camera, &mut self.data.terrain, &mut self.root_node);
    }

    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, target: &deferred::textures::Textures) {
        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            store: true,
        };

        let render_bundles = vec![&self.terrain_bundle, &self.data.models.render_bundle];
        self.render_to_texture_group(device, queue, target, ops, render_bundles);
    }

    fn render_to_texture_group(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &deferred::textures::Textures,
        ops: wgpu::Operations<wgpu::Color>,
        bundles: Vec<&wgpu::RenderBundle>,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &target.normals_texture_view,
                            resolve_target: None,
                            ops,
                        },
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &target.base_color_texture_view,
                            resolve_target: None,
                            ops,
                        },
                    ],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                        attachment: &target.depth_texture_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                })
                .execute_bundles(bundles.into_iter());
            queue.submit(std::iter::once(encoder.finish()));
        }
    }
}

impl WorldData {
    pub fn get_elevation(&self, p: Vector2<f32>) -> f32 {
        let xz = p * settings::TERRAIN_SCALE;
        let q = vec2(
            self.noise.fbm(xz, settings::TERRAIN_OCTAVES),
            self.noise.fbm(xz + vec2(1.0, 1.0), settings::TERRAIN_OCTAVES),
        );

        let r = vec2(
            self.noise.fbm(xz + q + vec2(1.7 + 0.15, 9.2 + 0.15), settings::TERRAIN_OCTAVES),
            self.noise.fbm(xz + q + vec2(8.3 + 0.126, 2.8 + 0.126), settings::TERRAIN_OCTAVES),
        );

        (self.noise.fbm(xz + r, settings::TERRAIN_OCTAVES) - 0.3) / settings::TERRAIN_SCALE / 2.0
    }
}

fn get_terrain_bundle(
    device: &wgpu::Device,
    camera: &camera::Camera,
    terrain: &mut terrain::Terrain,
    root_node: &mut node::Node,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[settings::COLOR_TEXTURE_FORMAT, settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });
    encoder.set_pipeline(&terrain.render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
    encoder.set_bind_group(1, &terrain.texture_bind_group, &[]);
    encoder.set_bind_group(2, &terrain.noise_bindings.bind_group, &[]);

    for lod in 0..=settings::LODS.len() {
        let lod_buffer = terrain.compute.lods.get(lod).expect("Could not get LOD!");
        encoder.set_index_buffer(lod_buffer.index_buffer.slice(..));

        for node in root_node.get_terrain_nodes(camera, lod as u32) {
            encoder.set_vertex_buffer(0, node.buffer.slice(..));
            encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
}
