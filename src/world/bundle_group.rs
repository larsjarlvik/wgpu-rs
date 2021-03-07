use super::{node, terrain, water, WorldData};
use crate::{camera, deferred, settings};

pub struct BundleGroup {
    pub terrain: Option<wgpu::RenderBundle>,
    pub water: Option<wgpu::RenderBundle>,
    pub models: Option<wgpu::RenderBundle>,
    pub sky: Option<wgpu::RenderBundle>,
}

impl BundleGroup {
    pub fn new() -> Self {
        Self {
            terrain: None,
            water: None,
            models: None,
            sky: None,
        }
    }

    pub fn with_terrain_bundle(
        mut self,
        device: &wgpu::Device,
        world_data: &mut WorldData,
        camera: &camera::camera::Camera,
        root_node: &node::Node,
    ) -> Self {
        self.terrain = Some(get_terrain_bundle(device, &camera, &mut world_data.terrain, &root_node));
        self
    }

    pub fn with_water_bundle(
        mut self,
        device: &wgpu::Device,
        world_data: &mut WorldData,
        camera: &camera::camera::Camera,
        root_node: &node::Node,
    ) -> Self {
        self.water = Some(get_water_bundle(device, &camera, &mut world_data.water, &root_node));
        self
    }

    pub fn with_models_bundle(mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera) -> Self {
        self.models = Some(world_data.models.get_render_bundle(device, &camera));
        self
    }

    pub fn with_sky_bundle(mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera) -> Self {
        self.sky = Some(world_data.sky.get_render_bundle(device, &camera));
        self
    }

    pub fn update(&mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera, root_node: &node::Node) {
        if self.terrain.is_some() {
            self.terrain = Some(get_terrain_bundle(device, &camera, &mut world_data.terrain, &root_node));
        }
        if self.water.is_some() {
            self.water = Some(get_water_bundle(device, &camera, &mut world_data.water, &root_node));
        }
        if self.models.is_some() {
            self.models = Some(world_data.models.get_render_bundle(device, &camera));
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, world_data: &mut WorldData, camera: &camera::camera::Camera) {
        if self.sky.is_some() {
            self.sky = Some(world_data.sky.get_render_bundle(device, &camera));
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, target: &deferred::textures::Textures) {
        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            store: true,
        };

        let mut bundles = vec![];
        if self.terrain.is_some() {
            bundles.push(self.terrain.as_ref().unwrap());
        }
        if self.models.is_some() {
            bundles.push(self.models.as_ref().unwrap());
        }

        encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("terrain_models_render_pass"),
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
        if self.water.is_some() {
            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("water_render_pass"),
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
                .execute_bundles(std::iter::once(self.water.as_ref().unwrap()));
        }
    }

    pub fn render_sky(&self, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
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
            .execute_bundles(std::iter::once(self.sky.as_ref().unwrap()));
    }
}

fn get_terrain_bundle(
    device: &wgpu::Device,
    camera: &camera::camera::Camera,
    terrain: &mut terrain::Terrain,
    root_node: &node::Node,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: Some("terrain_bundle"),
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
    camera: &camera::camera::Camera,
    water: &mut water::Water,
    root_node: &node::Node,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: Some("water_bundle"),
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
