use crate::{camera, models};
use rand::Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;

struct Asset {
    name: String,
    density: f32,
    min_size: f32,
    max_size: f32,
}

pub struct AssetsTile {
    instance_ids: HashMap<String, Vec<String>>,
}

pub struct Assets {
    assets: Vec<Asset>,
}

impl Assets {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &camera::Camera, models: &mut models::Models) -> Assets {
        let mut assets = Vec::new();
        assets.push(Asset {
            name: "pine-1".to_string(),
            density: 0.075,
            min_size: 1.5,
            max_size: 2.5,
        });
        assets.push(Asset {
            name: "pine-2".to_string(),
            density: 0.075,
            min_size: 1.5,
            max_size: 2.5,
        });
        assets.push(Asset {
            name: "pine-3".to_string(),
            density: 0.075,
            min_size: 1.5,
            max_size: 2.5,
        });
        assets.push(Asset {
            name: "rock-1".to_string(),
            density: 0.075,
            min_size: 1.5,
            max_size: 4.5,
        });

        for asset in &assets {
            models.load_model(&device, &queue, asset.name.as_str(), format!("{}.glb", asset.name).as_str());
        }

        models.refresh_render_bundle(&device, &camera);
        Assets { assets }
    }

    pub fn create_tile(&mut self, models: &mut models::Models, x: i32, z: i32, tile_size: f32) -> AssetsTile {
        let mut instance_ids = HashMap::new();
        for asset in &self.assets {
            let a = add_asset(models, asset, x, z, tile_size);
            instance_ids.insert(asset.name.clone(), a);
        }

        AssetsTile { instance_ids }
    }

    pub fn refresh(&self, device: &wgpu::Device, camera: &camera::Camera, models: &mut models::Models) {
        for asset in &self.assets {
            models.write_instance_buffers(&device, asset.name.as_str());
        }
        models.refresh_render_bundle(&device, &camera);
    }

    pub fn delete_tile(&self, models: &mut models::Models, assets_tile: &AssetsTile) {
        for asset in &self.assets {
            for ids in assets_tile.instance_ids.values() {
                for id in ids {
                    models.remove_instance(asset.name.as_str(), &id);
                }
            }
        }
    }
}

fn add_asset(models: &mut models::Models, asset: &Asset, x: i32, z: i32, tile_size: f32) -> Vec<String> {
    let count = (tile_size * asset.density) as u32;
    let tile_size = tile_size;
    let instances = (0..count)
        .into_par_iter()
        .map(|_| {
            let mut rng = rand::thread_rng();
            let mx = (x as f32 + (&rng.gen::<f32>() - 0.5)) * tile_size;
            let mz = (z as f32 + (&rng.gen::<f32>() - 0.5)) * tile_size;
            let my = 0.0;

            models::data::Instance {
                transform: {
                    cgmath::Matrix4::from_translation(cgmath::Vector3 { x: mx, y: my, z: mz })
                        * cgmath::Matrix4::from_angle_y(cgmath::Deg(&rng.gen::<f32>() * 360.0))
                        * cgmath::Matrix4::from_scale(rng.gen_range(asset.min_size, asset.max_size))
                }
                .into(),
            }
        })
        .collect::<Vec<models::data::Instance>>();

    let mut instance_ids = Vec::new();
    for instance in instances {
        let id = nanoid::simple();
        models.add_instance(asset.name.as_str(), id.clone(), instance);
        instance_ids.push(id);
    }

    instance_ids
}
