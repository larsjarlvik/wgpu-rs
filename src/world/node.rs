use super::{assets, World};
use crate::{
    camera::{self, frustum::*},
    models, settings,
};
use cgmath::*;
use models::data::Instance;
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

struct Terrain {
    buffer: wgpu::Buffer,
    instances: HashMap<String, models::InstanceBuffer>,
}

pub struct Node {
    children: Vec<Node>,
    x: f32,
    z: f32,
    depth: i32,
    terrain: Option<Terrain>,
}

impl Node {
    pub fn new(x: f32, z: f32, depth: i32, device: &wgpu::Device, queue: &wgpu::Queue, world: &mut World) -> Self {
        let mut node = Self {
            children: vec![],
            depth,
            x,
            z,
            terrain: None,
        };

        if node.is_leaf() {
            node.build_leaf_node(device, queue, world);
        } else {
            node.add_children(device, queue, world);
        }

        node
    }

    pub fn is_leaf(&self) -> bool {
        self.depth == 0
    }

    pub fn get_size(&self) -> f32 {
        2.0f32.powf(self.depth as f32) * settings::TILE_SIZE
    }

    pub fn add_children(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world: &mut World) {
        let child_size = self.get_size() / 2.0;
        for cz in -1..1 {
            for cx in -1..1 {
                self.children.push(Node::new(
                    self.x + ((cx as f32 + 0.5) * child_size),
                    self.z + ((cz as f32 + 0.5) * child_size),
                    self.depth - 1,
                    device,
                    queue,
                    world,
                ));
            }
        }
    }

    fn build_leaf_node(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world: &mut World) {
        let mut instances = HashMap::new();
        for a in assets::ASSETS {
            let assets = self.create_assets(world, a);
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("instance_buffer"),
                contents: bytemuck::cast_slice(assets.as_slice()),
                usage: wgpu::BufferUsage::VERTEX,
            });

            instances.insert(
                a.name.to_string(),
                models::InstanceBuffer {
                    buffer,
                    length: assets.len() as u32,
                },
            );
        }

        self.terrain = Some(Terrain {
            buffer: world.terrain.compute.compute(device, queue, self.x, self.z),
            instances,
        });
    }

    fn create_assets(&self, world: &mut World, asset: &assets::Asset) -> Vec<Instance> {
        let node_size = self.get_size();
        let count = (node_size * asset.density) as u32;
        (0..count)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::thread_rng();
                let mx = self.x + (&rng.gen::<f32>() - 0.5) * node_size;
                let mz = self.z + (&rng.gen::<f32>() - 0.5) * node_size;
                let my = world.get_elevation(vec2(mx, mz)) - 0.25;
                (mx, my, mz, rng.gen::<f32>(), rng.gen_range(asset.min_size, asset.max_size))
            })
            .filter(|asset| asset.1 > 0.0)
            .map(|asset| models::data::Instance {
                transform: {
                    Matrix4::from_translation(vec3(asset.0, asset.1, asset.2))
                        * Matrix4::from_angle_y(Deg(asset.3 * 360.0))
                        * Matrix4::from_scale(asset.4)
                }
                .into(),
            })
            .collect::<Vec<models::data::Instance>>()
    }

    pub fn get_terrain_buffer_slices(&self, camera: &camera::Camera) -> Vec<wgpu::BufferSlice> {
        if self.check_frustum(camera) {
            match &self.terrain {
                Some(t) => vec![t.buffer.slice(..)],
                None => self
                    .children
                    .iter()
                    .flat_map(|child| child.get_terrain_buffer_slices(camera))
                    .collect(),
            }
        } else {
            vec![]
        }
    }

    pub fn get_model_bundles(&self, device: &wgpu::Device, camera: &camera::Camera, models: &models::Models) -> Vec<wgpu::RenderBundle> {
        if self.check_frustum(camera) {
            match &self.terrain {
                Some(t) => vec![models.get_render_bundle(device, camera, &t.instances)],
                None => self
                    .children
                    .iter()
                    .flat_map(|child| child.get_model_bundles(device, camera, models))
                    .collect(),
            }
        } else {
            vec![]
        }
    }

    fn check_frustum(&self, camera: &camera::Camera) -> bool {
        let half_size = self.get_size() / 2.0;
        let min = vec3(self.x - half_size, -100.0, self.z - half_size);
        let max = vec3(self.x + half_size, 100.0, self.z + half_size);

        match camera.frustum.test_bounding_box(BoundingBox { min, max }) {
            Intersection::Inside | Intersection::Partial => true,
            Intersection::Outside => false,
        }
    }
}
