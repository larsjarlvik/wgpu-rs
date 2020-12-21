use cgmath::num_traits::Pow;

use crate::{camera, models};
extern crate nanoid;
use std::{collections::HashMap, time::Instant};
mod assets;
mod elevation;
mod terrain;

pub struct Tile {
    terrain: terrain::TerrainTile,
    assets: assets::AssetsTile,
}

pub struct World {
    pub tiles: HashMap<(i32, i32), Tile>,
    pub terrain: terrain::Terrain,
    pub assets: assets::Assets,
    pub noise: noise::OpenSimplex,
    pub tile_size: usize,
    pub tile_range: u32,
}

impl World {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera, models: &mut models::Models) -> World {
        let terrain = terrain::Terrain::new(&device, &queue, &camera);
        let assets = assets::Assets::new(&device, &queue, &camera, models);
        let noise = noise::OpenSimplex::new();
        let tiles = HashMap::new();

        World {
            terrain,
            assets,
            tiles,
            noise,
            tile_size: 14,
            tile_range: 14,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, camera: &camera::Camera, models: &mut models::Models) {
        let now = Instant::now();
        let (x, z) = self.get_center(camera);

        let add_dirty = self.add_tiles(device, models, x, z);
        let clear_dirty = self.clear_tiles(x, z, models);

        if add_dirty || clear_dirty {
            let terrain_tiles = self.tiles.iter().map(|t| &t.1.terrain).collect::<Vec<&terrain::TerrainTile>>();
            self.terrain.build(device, camera, terrain_tiles);
            self.assets.refresh(device, camera, models);
            println!("Generate tiles: {} ms", now.elapsed().as_millis());
        }
    }

    fn add_tiles(&mut self, device: &wgpu::Device, models: &mut models::Models, cx: i32, cz: i32) -> bool {
        let mut dirty = false;
        for z in (cz - self.tile_range as i32 - 1)..(cz + self.tile_range as i32 + 1) {
            for x in (cx - self.tile_range as i32 - 1)..(cx + self.tile_range as i32 + 1) {
                let distance = ((x as f32 - cx as f32).pow(2.0) + (z as f32 - cz as f32).powf(2.0)).sqrt().abs();
                if !self.tiles.contains_key(&(x, z)) && distance <= self.tile_range as f32 {
                    self.build_tile(&device, models, x, z);
                    dirty = true;
                }
            }
        }
        dirty
    }

    fn clear_tiles(&mut self, cx: i32, cz: i32, models: &mut models::Models) -> bool {
        let tile_count = self.tiles.len();
        let tile_range = self.tile_range as i32;
        let ass = &self.assets;

        self.tiles.retain(|key, value| {
            let (x, z) = key;
            let distance = ((*x as f32 - cx as f32).pow(2.0) + (*z as f32 - cz as f32).powf(2.0)).sqrt().abs();
            let keep = distance <= tile_range as f32 * 1.2;
            if !keep {
                ass.delete_tile(models, &value.assets);
            }
            keep
        });

        tile_count != self.tiles.len()
    }

    fn get_center(&self, camera: &camera::Camera) -> (i32, i32) {
        let c_x = (camera.target.x / self.tile_size as f32).round() as i32;
        let c_z = (camera.target.z / self.tile_size as f32).round() as i32;
        (c_x, c_z)
    }

    fn build_tile(&mut self, device: &wgpu::Device, models: &mut models::Models, x: i32, z: i32) {
        let tile_size = self.tile_size as f32;
        let terrain_tile = self.terrain.create_tile(&device, &self.noise, x, z, tile_size);
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
