use super::{assets, WorldData};
use crate::{
    camera::{self, frustum::*},
    models, settings,
};
use cgmath::*;
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;

struct Terrain {
    buffer: wgpu::Buffer,
    instance_keys: HashMap<String, Vec<String>>,
}

pub struct Node {
    children: Vec<Node>,
    x: f32,
    z: f32,
    size: f32,
    depth: i32,
    terrain: Option<Terrain>,
}

impl Node {
    pub fn new(x: f32, z: f32, depth: i32) -> Self {
        let mut node = Self {
            children: vec![],
            depth,
            x,
            z,
            size: 2.0f32.powf(depth as f32) * settings::TILE_SIZE,
            terrain: None,
        };

        if !node.is_leaf() {
            node.add_children();
        }

        node
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world: &mut WorldData, camera: &camera::Camera) {
        let distance = vec2(self.x, self.z).distance(vec2(camera.eye.x, camera.eye.z)) - self.size;
        match &self.terrain {
            Some(_) => {
                if distance > camera.z_far {
                    self.delete_leaf_node(world);
                }
            }
            None => {
                if self.is_leaf() {
                    if distance < camera.z_far {
                        self.build_leaf_node(device, queue, world);
                    }
                } else {
                    for child in self.children.iter_mut() {
                        child.update(device, queue, world, camera);
                    }
                }
            }
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.depth == 0
    }

    pub fn add_children(&mut self) {
        let child_size = self.size / 2.0;
        for cz in -1..1 {
            for cx in -1..1 {
                self.children.push(Node::new(
                    self.x + ((cx as f32 + 0.5) * child_size),
                    self.z + ((cz as f32 + 0.5) * child_size),
                    self.depth - 1,
                ));
            }
        }
    }

    fn build_leaf_node(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world: &mut WorldData) {
        let mut instance_keys = HashMap::new();

        for asset in assets::ASSETS {
            let instances = self.create_assets(world, asset);
            let keys = instances.keys().cloned().collect::<Vec<String>>();
            let model = world.models.models.get_mut(&asset.name.to_string()).expect("Model not found!");

            model.add_instances(instances);
            instance_keys.insert(asset.name.to_string(), keys);
        }

        self.terrain = Some(Terrain {
            buffer: world.terrain.compute.compute(device, queue, self.x, self.z),
            instance_keys,
        });
    }

    fn delete_leaf_node(&mut self, world: &mut WorldData) {
        match &self.terrain {
            Some(terrain) => {
                for (key, model) in world.models.models.iter_mut() {
                    model.remove_instances(terrain.instance_keys.get(key).expect("Model not found!"));
                }
            }
            None => {}
        }

        self.terrain = None;
    }

    fn create_assets(&self, world: &WorldData, asset: &assets::Asset) -> HashMap<String, models::data::Instance> {
        let count = (self.size * asset.density) as u32;
        let model = world.models.models.get(&asset.name.to_string()).expect("Model not found!");

        (0..count)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::thread_rng();
                let mx = self.x + (&rng.gen::<f32>() - 0.5) * self.size;
                let mz = self.z + (&rng.gen::<f32>() - 0.5) * self.size;
                let my = world.get_elevation(vec2(mx, mz)) - 0.25;
                (mx, my, mz, rng.gen::<f32>(), rng.gen_range(asset.min_size, asset.max_size))
            })
            .filter(|asset| asset.1 > 0.0)
            .map(|(mx, my, mz, rot, scale)| {
                let t = Matrix4::from_translation(vec3(mx, my, mz)) * Matrix4::from_angle_y(Deg(rot * 360.0)) * Matrix4::from_scale(scale);
                (
                    nanoid::simple(),
                    (
                        models::data::InstanceData { transform: t.into() },
                        model.transformed_bounding_box(t),
                    ),
                )
            })
            .collect::<HashMap<String, models::data::Instance>>()
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

    fn check_frustum(&self, camera: &camera::Camera) -> bool {
        let half_size = self.size / 2.0;
        let min = Point3::new(self.x - half_size, -100.0, self.z - half_size);
        let max = Point3::new(self.x + half_size, 100.0, self.z + half_size);

        match camera.frustum.test_bounding_box(&BoundingBox { min, max }) {
            Intersection::Inside | Intersection::Partial => true,
            Intersection::Outside => false,
        }
    }
}
