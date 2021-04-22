use super::{assets, WorldData};
use crate::{
    camera,
    pipelines::{self, model},
    plane, settings,
};
use cgmath::*;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub struct NodeData {
    pub terrain_buffer: wgpu::Buffer,
    pub water_buffer: wgpu::Buffer,
    pub model_instances: HashMap<String, Vec<model::Instance>>,
    pub lods: Vec<HashMap<plane::ConnectType, plane::LodBuffer>>,
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
        let distance = vec2(self.x, self.z).distance(vec2(viewport.eye.x, viewport.eye.z)) - self.radius;
        let z_far_range = num_traits::Float::sqrt(viewport.z_far * viewport.z_far + viewport.z_far * viewport.z_far);

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
        let (plane, y_min, y_max) = world.heightmap.sub(self.x, self.z, settings::TILE_SIZE);
        self.bounding_box.min.y = y_min;
        self.bounding_box.max.y = y_max.max(0.0);

        let lods = (0..=settings::LODS.len())
            .map(|lod| plane.create_indices(&device, lod as u32 + 1))
            .collect();

        let terrain_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain_vertex_buffer"),
            contents: bytemuck::cast_slice(&plane.vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        for asset in assets::ASSETS {
            let model = world.models.models.get(&asset.name.to_string()).expect("Model not found!");
            let assets = self.create_assets(world, asset);
            for asset in &assets {
                let asset_bb = model.bounding_box.transform(asset.transform);
                self.bounding_box = self.bounding_box.grow(&asset_bb);
            }

            model_instances.insert(asset.name.to_string(), assets);
        }

        self.data = Some(NodeData {
            terrain_buffer,
            water_buffer: world.water.create_buffer(&device, self.x, self.z),
            model_instances,
            lods,
        });
    }

    fn create_assets(&self, world: &WorldData, asset: &assets::Asset) -> Vec<pipelines::model::Instance> {
        let count = (self.size * self.size * asset.density) as u32;

        (0..count)
            .into_par_iter()
            .map(|i| {
                let seed = format!("{}_NODE_{}_{}_{}_{}", settings::MAP_SEED, self.x, self.z, asset.name, i);
                let mut rng: Pcg64 = Seeder::from(seed).make_rng();
                let m = vec2(
                    self.x + (rng.gen::<f32>() - 0.5) * self.size,
                    self.z + (rng.gen::<f32>() - 0.5) * self.size,
                );
                let v = world.get_vertex(m);
                let elev = world.get_elevation(v, m) - 0.25;
                (
                    elev,
                    v.normal,
                    m.x,
                    m.y,
                    rng.gen::<f32>(),
                    rng.gen_range(asset.min_size..asset.max_size),
                )
            })
            .filter(|(my, normal, ..)| *my > 0.0 && normal[1] > asset.max_slope)
            .map(|(my, _, mx, mz, rot, scale)| {
                let t = Matrix4::from_translation(vec3(mx, my, mz)) * Matrix4::from_angle_y(Deg(rot * 360.0)) * Matrix4::from_scale(scale);
                pipelines::model::Instance { transform: t.into() }
            })
            .collect::<Vec<pipelines::model::Instance>>()
    }

    pub fn get_nodes(&self, camera: &camera::Instance) -> Vec<(&Self, &NodeData)> {
        if camera.frustum.test_bounding_box(&self.bounding_box) {
            match &self.data {
                Some(t) => vec![(&self, &t)],
                None => self.children.iter().flat_map(|child| child.get_nodes(camera)).collect(),
            }
        } else {
            vec![]
        }
    }
}
