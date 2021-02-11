use std::time::Instant;

use crate::{camera, deferred, models, noise, settings};
use cgmath::{vec2, Vector2};
mod assets;
mod node;
mod terrain;
mod water;

pub struct WorldData {
    terrain: terrain::Terrain,
    pub water: water::Water,
    noise: noise::Noise,
    pub models: models::Models,
}

pub struct World {
    root_node: node::Node,
    pub data: WorldData,
    pub terrain_bundle: wgpu::RenderBundle,
    pub water_bundle: wgpu::RenderBundle,
    pub models_bundle: wgpu::RenderBundle,
}

impl World {
    pub async fn new(device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor, queue: &wgpu::Queue, cameras: &camera::Cameras) -> Self {
        let noise = noise::Noise::new(&device, &queue).await;
        let mut models = models::Models::new(&device, &cameras);
        for asset in assets::ASSETS {
            models.load_model(&device, &queue, asset.name, format!("{}.glb", asset.name).as_str());
        }

        let mut root_node = node::Node::new(0.0, 0.0, settings::TILE_DEPTH);
        let mut terrain = terrain::Terrain::new(device, queue, &cameras, &noise);
        let mut water = water::Water::new(device, swap_chain_desc, &cameras, &noise);

        let terrain_bundle = get_terrain_bundle(device, &cameras.eye_cam, &mut terrain, &mut root_node);
        let water_bundle = get_water_bundle(device, &cameras.eye_cam, &mut water, &mut root_node);
        let models_bundle = models.get_render_bundle(device, &cameras.eye_cam);

        let mut world = Self {
            data: WorldData {
                terrain,
                water,
                noise,
                models,
            },
            terrain_bundle,
            water_bundle,
            models_bundle,
            root_node,
        };

        world.terrain_bundle = get_terrain_bundle(device, &cameras.eye_cam, &mut world.data.terrain, &mut world.root_node);
        world
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, cameras: &camera::Cameras, time: Instant) {
        self.root_node.update(device, queue, &mut self.data, &cameras.eye_cam);
        self.data.water.update(queue, time);
        self.water_bundle = get_water_bundle(device, &cameras.eye_cam, &mut self.data.water, &mut self.root_node);
    }

    pub fn update_bundle(&mut self, device: &wgpu::Device, camera: &camera::Camera) {
        self.terrain_bundle = get_terrain_bundle(device, camera, &mut self.data.terrain, &mut self.root_node);
        self.data.models.get_render_bundle(device, camera);
    }

    pub fn resize(&mut self, device: &wgpu::Device, swap_chain_desc: &wgpu::SwapChainDescriptor, cameras: &camera::Cameras) {
        self.data.water = water::Water::new(device, swap_chain_desc, cameras, &self.data.noise);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &deferred::textures::Textures, above_water: bool) {
        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            store: true,
        };

        let mut bundles = vec![&self.terrain_bundle];
        if above_water { bundles.push(&self.models_bundle); }

        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
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
    }

    pub fn render_water(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, depth_target: &wgpu::TextureView) {
        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &depth_target,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            })
            .execute_bundles(std::iter::once(&self.water_bundle));
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
        let terrain_lod = terrain.compute.lods.get(lod).expect("Could not get LOD!");

        for (terrain, connect_type) in root_node.get_nodes(camera, lod as u32) {
            let lod_buffer = terrain_lod.get(&connect_type).unwrap();
            encoder.set_vertex_buffer(0, terrain.terrain_buffer.slice(..));
            encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("terrain") })
}

fn get_water_bundle(
    device: &wgpu::Device,
    camera: &camera::Camera,
    water: &mut water::Water,
    root_node: &mut node::Node,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: None,
        color_formats: &[settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });
    encoder.set_pipeline(&water.render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
    encoder.set_bind_group(1, &water.uniforms.bind_group, &[]);
    encoder.set_bind_group(2, &water.noise_bindings.bind_group, &[]);
    encoder.set_bind_group(3, &water.texture_bind_group, &[]);

    for lod in 0..=settings::LODS.len() {
        let water_lod = water.lods.get(lod).expect("Could not get LOD!");

        for (water, connect_type) in root_node.get_nodes(camera, lod as u32) {
            let lod_buffer = water_lod.get(&connect_type).unwrap();
            encoder.set_vertex_buffer(0, water.water_buffer.slice(..));
            encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("water") })
}
