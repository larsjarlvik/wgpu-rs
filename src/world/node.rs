use super::{node_uniforms, WorldData};
use crate::{
    assets, camera,
    pipelines::{self, model},
    settings,
    world::node_assets,
};
use cgmath::*;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::collections::HashMap;

pub struct NodeData {
    pub model_instances: HashMap<String, Vec<model::Instance>>,
    pub uniforms: node_uniforms::UniformBuffer,
}

pub struct Node {
    pub x: f32,
    pub z: f32,
    pub bounding_box: camera::BoundingBox,
    pub data: Option<NodeData>,
    children: Vec<Node>,
    size: f32,
    radius: f32,
    depth: u32,
}

impl Node {
    pub fn new(x: f32, z: f32, depth: u32) -> Self {
        let size = 2.0f32.powf(depth as f32) * settings::TILE_SIZE as f32;
        let radius = (size * size + size * size).sqrt() / 2.0;
        let bounding_box = camera::BoundingBox {
            min: Point3::new(x - size / 2.0, -10000.0, z - size / 2.0),
            max: Point3::new(x + size / 2.0, 10000.0, z + size / 2.0),
        };

        let mut node = Self {
            x,
            z,
            bounding_box,
            data: None,
            children: vec![],
            depth,
            size,
            radius,
        };

        if !node.is_leaf() {
            node.add_children();
        }

        node
    }

    pub fn update(&mut self, device: &wgpu::Device, world: &mut WorldData, viewport: &camera::Viewport) {
        let distance = self.get_distance(viewport.eye.x, viewport.eye.z);
        let z_far_range = num_traits::Float::sqrt(viewport.z_far.powf(2.0) + viewport.z_far.powf(2.0));

        if distance > z_far_range {
            self.data = None;
            self.children = Vec::new();
            return;
        }

        if self.data.is_none() {
            if self.is_leaf() {
                let distance = vec2(self.x, self.z).distance(vec2(viewport.eye.x, viewport.eye.z)) - self.radius;
                if distance < z_far_range && self.data.is_none() {
                    self.build_leaf_node(device, world);
                }
            } else {
                self.add_children();
                for child in self.children.iter_mut() {
                    child.update(device, world, viewport);
                    self.bounding_box = self.bounding_box.grow(&child.bounding_box);
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
                    let child = Node::new(
                        self.x + ((cx as f32 + 0.5) * child_size),
                        self.z + ((cz as f32 + 0.5) * child_size),
                        self.depth - 1,
                    );
                    self.bounding_box = self.bounding_box.grow(&child.bounding_box);
                    self.children.push(child);
                }
            }
        }
    }

    fn build_leaf_node(&mut self, device: &wgpu::Device, world: &mut WorldData) {
        let mut model_instances: HashMap<String, Vec<pipelines::model::Instance>> = HashMap::new();
        let (y_min, y_max) = world.map.min_max_elevation(self.x, self.z, settings::TILE_SIZE);
        self.bounding_box.min.y = y_min;
        self.bounding_box.max.y = y_max.max(0.0);

        let seed = format!("{}_NODE_{}_{}", settings::MAP_SEED, self.x, self.z);
        let mut rng: Pcg64 = Seeder::from(seed).make_rng();

        for (i, asset) in assets::ASSETS.iter().enumerate() {
            for (n, mesh) in asset.meshes.iter().enumerate() {
                let assets = node_assets::create_assets(self.x, self.z, self.size, world, mesh, &format!("{}_{}", i, n));
                for asset in assets {
                    let name = mesh.variants.get(rng.gen_range(0..mesh.variants.len())).unwrap();

                    let model = world
                        .models
                        .meshes
                        .get(&name.to_string())
                        .expect(format!("Mesh {} not found!", name).as_str());

                    let asset_bb = model.bounding_box.transform(asset.transform);
                    self.bounding_box = self.bounding_box.grow(&asset_bb);

                    let instances = model_instances.entry(name.to_string()).or_insert(vec![]);
                    instances.push(asset);
                }
            }
        }

        let size = (settings::TILE_SIZE * 2u32.pow(settings::TILE_DEPTH)) as f32;
        let uniforms = node_uniforms::UniformBuffer::new(
            device,
            &world.terrain.node_uniform_bind_group_layout,
            node_uniforms::Uniforms {
                translation: [self.x, self.z],
                size,
            },
        );

        self.data = Some(NodeData { model_instances, uniforms });
    }

    pub fn get_nodes(&self, frustum: &camera::FrustumCuller) -> Vec<(&Self, &NodeData)> {
        if frustum.test_bounding_box(&self.bounding_box) {
            match &self.data {
                Some(t) => vec![(&self, &t)],
                None => self.children.iter().flat_map(|child| child.get_nodes(frustum)).collect(),
            }
        } else {
            vec![]
        }
    }

    pub fn get_distance(&self, x: f32, z: f32) -> f32 {
        vec2(self.x, self.z).distance(vec2(x, z)) - self.radius
    }
}
