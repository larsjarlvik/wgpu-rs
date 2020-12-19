use crate::{camera, models};
extern crate nanoid;
use rand::*;
use rayon::prelude::*;
use std::{collections::HashMap, time::Instant};
use terrain::TerrainTile;
mod elevation;
pub mod terrain;

pub struct Tile {
    terrain_tile: TerrainTile,
    instance_ids: Vec<String>,
}

pub struct World {
    pub tiles: HashMap<(i32, i32), Tile>,
    pub terrain: terrain::Terrain,
    pub tile_size: usize,
    pub tile_range: u32,
}

impl World {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera, models: &mut models::Models) -> World {
        models.load_model(&device, &queue, "pine", "pine.glb");
        models.refresh_render_bundle(&device, &camera);
        let terrain = terrain::Terrain::new(&device, &camera);
        let tiles = HashMap::new();

        World {
            terrain,
            tiles,
            tile_size: 12,
            tile_range: 20,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, camera: &camera::Camera, models: &mut models::Models) {
        let now = Instant::now();
        let (x, z) = self.get_center(camera);

        let add_dirty = self.add_tiles(device, models, x, z);
        let clear_dirty = self.clear_tiles(x, z, models);

        if add_dirty || clear_dirty {
            models.write_instance_buffers(&device, "pine");
            models.refresh_render_bundle(&device, &camera);

            let terrain_tiles = self.tiles.iter().map(|t| &t.1.terrain_tile).collect::<Vec<&TerrainTile>>();
            self.terrain.build(device, camera, terrain_tiles);
            println!("Generate tiles: {} ms", now.elapsed().as_millis());
        }
    }

    fn add_tiles(&mut self, device: &wgpu::Device, models: &mut models::Models, cx: i32, cz: i32) -> bool {
        let mut dirty = false;
        for z in (cz - self.tile_range as i32)..(cz + self.tile_range as i32) {
            for x in (cx - self.tile_range as i32)..(cx + self.tile_range as i32) {
                if !self.tiles.contains_key(&(x, z)) {
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
        self.tiles.retain(|key, value| {
            let (x, z) = key;
            let keep = (cx - *x).abs() <= tile_range && (cz - *z).abs() <= tile_range;
            if !keep {
                for id in &value.instance_ids {
                    models.remove_instance("pine", &id);
                }
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
        let terrain_tile = self.terrain.build_tile(&device, x, z, tile_size);
        let density = 0.3;
        let count = (self.tile_size as f32 * density) as u32;

        let terrain = &self.terrain;
        let tile_size = self.tile_size as f32;
        let instances = (0..count)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::thread_rng();
                let mx = (x as f32 + (&rng.gen::<f32>() - 0.5)) * tile_size;
                let mz = (z as f32 + (&rng.gen::<f32>() - 0.5)) * tile_size;
                let my = terrain.get_elevation(mx, mz);
                models::data::Instance {
                    transform: {
                        cgmath::Matrix4::from_translation(cgmath::Vector3 { x: mx, y: my, z: mz })
                            * cgmath::Matrix4::from_angle_y(cgmath::Deg(&rng.gen::<f32>() * 360.0))
                            * cgmath::Matrix4::from_scale(&rng.gen::<f32>() * 2.0 + 0.5)
                    }
                    .into(),
                }
            })
            .collect::<Vec<models::data::Instance>>();

        let mut instance_ids = Vec::new();
        for instance in instances {
            let id = nanoid::simple();
            models.add_instance("pine", id.clone(), instance);
            instance_ids.push(id);
        }

        self.tiles.insert(
            (x, z),
            Tile {
                terrain_tile,
                instance_ids,
            },
        );
    }
}
