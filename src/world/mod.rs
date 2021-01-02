use crate::{camera, models, noise, settings};
use camera::frustum;
use cgmath::*;
extern crate nanoid;
use std::{collections::HashMap, time::Instant};
mod assets;
mod terrain_pipeline;
mod terrain_tile;

pub struct Tile {
    pub terrain: terrain_tile::TerrainTile,
    assets: assets::AssetsTile,
}

pub struct World {
    pub tiles: HashMap<(i32, i32), Tile>,
    pub terrain: terrain_pipeline::Terrain,
    pub assets: assets::Assets,
    noise: noise::Noise,
    tile_size: u32,
    tile_range: u32,
}

impl World {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &camera::Camera,
        noise: noise::Noise,
        models: &mut models::Models,
    ) -> World {
        let tile_size = 40;
        let terrain = terrain_pipeline::Terrain::new(device, queue, camera, &noise, tile_size);
        let assets = assets::Assets::new(device, queue, camera, models);
        let tiles = HashMap::new();
        let tile_range = (camera.z_far / tile_size as f32).ceil() as u32;

        World {
            terrain,
            assets,
            tiles,
            tile_size,
            tile_range,
            noise,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera, models: &mut models::Models) {
        let now = Instant::now();
        let dirty = self.clear_tiles(camera, models) || self.add_tiles(device, queue, camera, models);

        if dirty {
            let terrain_tiles = self.tiles.iter().map(|t| &t.1.terrain).collect::<Vec<&terrain_tile::TerrainTile>>();
            self.terrain.refresh(device, camera, terrain_tiles);
            self.assets.refresh(device, camera, models);
            println!("Generate tiles: {} ms", now.elapsed().as_millis());
        }
    }

    fn add_tiles(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera, models: &mut models::Models) -> bool {
        let (cx, cz) = self.get_center(camera);
        let mut dirty = false;
        for z in (cz - self.tile_range as i32 - 1)..(cz + self.tile_range as i32 + 1) {
            for x in (cx - self.tile_range as i32 - 1)..(cx + self.tile_range as i32 + 1) {
                if !self.tiles.contains_key(&(x, z)) && in_frustum(camera, x, z, self.tile_size) {
                    self.build_tile(device, queue, models, x, z);
                    dirty = true;
                }
            }
        }
        dirty
    }

    fn clear_tiles(&mut self, camera: &camera::Camera, models: &mut models::Models) -> bool {
        let tile_count = self.tiles.len();
        let tile_size = self.tile_size;
        let ass = &self.assets;

        self.tiles.retain(|key, value| {
            let (x, z) = key;
            let retain = in_frustum(camera, *x, *z, tile_size);
            if !retain {
                ass.delete_tile(models, &value.assets);
            }
            retain
        });

        tile_count != self.tiles.len()
    }

    fn get_center(&self, camera: &camera::Camera) -> (i32, i32) {
        let c_x = (camera.target.x / self.tile_size as f32).round() as i32;
        let c_z = (camera.target.z / self.tile_size as f32).round() as i32;
        (c_x, c_z)
    }

    fn build_tile(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, models: &mut models::Models, x: i32, z: i32) {
        let tile_size = self.tile_size as f32;
        let terrain_tile = self.terrain.create_tile(device, queue, x, z);
        let assets_tile = self.assets.create_tile(&self.noise, models, x, z, tile_size);

        self.tiles.insert(
            (x, z),
            Tile {
                terrain: terrain_tile,
                assets: assets_tile,
            },
        );
    }
}

fn in_frustum(camera: &camera::Camera, x: i32, z: i32, tile_size: u32) -> bool {
    let ts = tile_size as f32;
    let bounding_box = frustum::BoundingBox::from_params(
        vec3((x as f32 - 0.5) * ts, -100.0, (z as f32 - 0.5) * ts),
        vec3((x as f32 + 0.5) * ts, 100.0, (z as f32 + 0.5) * ts),
    );
    match camera.frustum.test_bounding_box(bounding_box) {
        frustum::Intersection::Inside | frustum::Intersection::Partial => true,
        frustum::Intersection::Outside => false,
    }
}

pub fn get_elevation(p: Vector2<f32>, noise: &noise::Noise) -> f32 {
    let xz = p * settings::TERRAIN_SCALE;
    let q = vec2(
        noise.fbm(xz, settings::TERRAIN_OCTAVES),
        noise.fbm(xz + vec2(1.0, 1.0), settings::TERRAIN_OCTAVES),
    );

    let r = vec2(
        noise.fbm(xz + q + vec2(1.7 + 0.15, 9.2 + 0.15), settings::TERRAIN_OCTAVES),
        noise.fbm(xz + q + vec2(8.3 + 0.126, 2.8 + 0.126), settings::TERRAIN_OCTAVES),
    );

    (noise.fbm(xz + r, settings::TERRAIN_OCTAVES) - 0.3) / settings::TERRAIN_SCALE / 2.0
}
