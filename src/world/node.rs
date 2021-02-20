use super::{assets, WorldData};
use crate::{
    camera::{self, frustum::*},
    models,
    plane::{self, ConnectType},
    settings,
};
use cgmath::*;
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;

pub struct NodeData {
    pub terrain_buffer: wgpu::Buffer,
    pub water_buffer: wgpu::Buffer,
    instance_keys: HashMap<String, Vec<String>>,
}

pub struct Node {
    children: Vec<Node>,
    x: f32,
    z: f32,
    size: f32,
    radius: f32,
    depth: i32,
    data: Option<NodeData>,
}

impl Node {
    pub fn new(x: f32, z: f32, depth: i32) -> Self {
        let size = 2.0f32.powf(depth as f32) * settings::TILE_SIZE as f32;
        let radius = (size * size + size * size).sqrt() / 2.0;
        let mut node = Self {
            children: vec![],
            depth,
            x,
            z,
            size,
            radius,
            data: None,
        };

        if !node.is_leaf() {
            node.add_children();
        }

        node
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, world: &mut WorldData, camera: &camera::camera::Camera) {
        let distance = vec2(self.x, self.z).distance(vec2(camera.uniforms.data.look_at[0], camera.uniforms.data.look_at[2])) - self.radius;
        if distance > camera.z_far_range {
            self.delete_node(world);
            return;
        }

        if self.data.is_none() {
            if self.is_leaf() {
                let distance = vec2(self.x, self.z).distance(vec2(camera.uniforms.data.eye_pos[0], camera.uniforms.data.eye_pos[2])) - self.radius;
                if distance < camera.z_far_range && self.data.is_none() {
                    self.build_leaf_node(device, queue, world);
                }
            } else {
                self.add_children();
                for child in self.children.iter_mut() {
                    child.update(device, queue, world, camera);
                }
            }
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.depth == 0
    }

    pub fn add_children(&mut self) {
        if self.children.len() == 0 {
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

        self.data = Some(NodeData {
            terrain_buffer: world.terrain.compute.compute(device, queue, self.x, self.z),
            water_buffer: world.water.create_buffer(&device, self.x, self.z),
            instance_keys,
        });
    }

    fn delete_node(&mut self, world: &mut WorldData) {
        match &self.data {
            Some(terrain) => {
                for (key, model) in world.models.models.iter_mut() {
                    model.remove_instances(terrain.instance_keys.get(key).expect("Model not found!"));
                }
            }
            None => {
                for child in self.children.iter_mut() {
                    child.delete_node(world);
                }
            }
        }

        self.children = Vec::new();
        self.data = None;
    }

    fn create_assets(&self, world: &WorldData, asset: &assets::Asset) -> HashMap<String, models::data::Instance> {
        let count = (self.size * self.size * asset.density) as u32;
        let model = world.models.models.get(&asset.name.to_string()).expect("Model not found!");

        (0..count)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::thread_rng();
                let mx = self.x + (rng.gen::<f32>() - 0.5) * self.size;
                let mz = self.z + (rng.gen::<f32>() - 0.5) * self.size;
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

    pub fn get_nodes(&self, camera: &camera::camera::Camera, lod: u32) -> Vec<(&NodeData, ConnectType)> {
        if self.check_frustum(camera) {
            match &self.data {
                Some(t) => {
                    let eye = vec3(camera.uniforms.data.eye_pos[0], camera.uniforms.data.eye_pos[1], camera.uniforms.data.eye_pos[2]);
                    if plane::get_lod(eye, vec3(self.x, 0.0, self.z), camera.uniforms.data.z_far) == lod {
                        let ct = plane::get_connect_type(eye, vec3(self.x, 0.0, self.z), lod, camera.uniforms.data.z_far);
                        return vec![(&t, ct)];
                    }
                    vec![]
                }
                None => self.children.iter().flat_map(|child| child.get_nodes(camera, lod)).collect(),
            }
        } else {
            vec![]
        }
    }

    fn check_frustum(&self, camera: &camera::camera::Camera) -> bool {
        let half_size = self.size / 2.0;
        let min = Point3::new(self.x - half_size, -100.0, self.z - half_size);
        let max = Point3::new(self.x + half_size, 100.0, self.z + half_size);

        match camera.frustum.test_bounding_box(&BoundingBox { min, max }) {
            Intersection::Inside | Intersection::Partial => true,
            Intersection::Outside => false,
        }
    }
}
