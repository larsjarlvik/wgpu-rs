use super::{node_uniforms, systems, WorldData};
use crate::{camera, settings, world::node_assets};
use cgmath::*;
use std::collections::HashMap;

pub struct NodeData {
    pub asset_instances: HashMap<String, Vec<systems::assets::Instance>>,
    pub uniforms: node_uniforms::UniformBuffer,
}

enum NodeTree {
    Branch(Box<Vec<Node>>),
    Leaf(NodeData),
    None,
}

pub struct Node {
    pub x: f32,
    pub z: f32,
    pub bounding_box: camera::BoundingBox,
    tree: NodeTree,
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

        Self {
            x,
            z,
            bounding_box,
            tree: NodeTree::None,
            depth,
            size,
            radius,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, world: &mut WorldData, viewport: &camera::Viewport) {
        let distance = self.get_distance(viewport.eye.x, viewport.eye.z);
        let z_far_range = num_traits::Float::sqrt(viewport.z_far.powf(2.0) + viewport.z_far.powf(2.0));

        if distance > z_far_range {
            self.tree = NodeTree::None;
            return;
        }

        match self.tree {
            NodeTree::None => {
                if self.depth == 0 {
                    let distance = vec2(self.x, self.z).distance(vec2(viewport.eye.x, viewport.eye.z)) - self.radius;
                    if distance < z_far_range {
                        self.build_leaf_node(device, world);
                    }
                } else {
                    self.add_children(device, world, viewport);
                }
            }
            _ => {}
        }
    }

    pub fn add_children(&mut self, device: &wgpu::Device, world: &mut WorldData, viewport: &camera::Viewport) {
        let mut children = vec![];
        let child_size = self.size / 2.0;

        for cz in -1..1 {
            for cx in -1..1 {
                let mut child = Node::new(
                    self.x + ((cx as f32 + 0.5) * child_size),
                    self.z + ((cz as f32 + 0.5) * child_size),
                    self.depth - 1,
                );
                child.update(device, world, viewport);
                self.bounding_box = self.bounding_box.grow(&child.bounding_box);
                children.push(child);
            }
        }

        self.tree = NodeTree::Branch(Box::new(children));
    }

    fn build_leaf_node(&mut self, device: &wgpu::Device, world: &mut WorldData) {
        let mut asset_instances: HashMap<String, Vec<systems::assets::Instance>> = HashMap::new();
        let (y_min, y_max) = world.map.min_max_elevation(self.x, self.z, settings::TILE_SIZE);
        self.bounding_box.min.y = y_min;
        self.bounding_box.max.y = y_max.max(0.0);

        for (name, asset) in &world.assets.assets {
            let node_assets = node_assets::create_assets(self.x, self.z, self.size, world, &asset, &format!("{}", name.clone()));
            for node_asset in node_assets {
                let asset_bb = asset.bounding_box.transform(node_asset.transform);
                self.bounding_box = self.bounding_box.grow(&asset_bb);

                let instances = asset_instances.entry(name.clone()).or_insert(vec![]);
                instances.push(node_asset);
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

        self.tree = NodeTree::Leaf(NodeData { asset_instances, uniforms });
    }

    pub fn get_nodes<'a>(&'a self, frustum: &camera::FrustumCuller) -> Vec<&'a Self> {
        optick::event!();
        if frustum.test_bounding_box(&self.bounding_box) {
            match &self.tree {
                NodeTree::Branch(children) => children.iter().flat_map(|child| child.get_nodes(frustum)).collect(),
                NodeTree::Leaf(_) => vec![&self],
                NodeTree::None => vec![],
            }
        } else {
            vec![]
        }
    }

    pub fn get_data<'a>(&'a self) -> Option<&'a NodeData> {
        match &self.tree {
            NodeTree::Branch(_) => None,
            NodeTree::Leaf(data) => Some(&data),
            NodeTree::None => None,
        }
    }

    pub fn get_distance(&self, x: f32, z: f32) -> f32 {
        vec2(self.x, self.z).distance(vec2(x, z)) - self.radius
    }
}
