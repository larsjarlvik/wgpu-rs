use std::f32::consts::PI;

use super::WorldData;
use crate::{assets, pipelines, settings};
use cgmath::*;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

struct NodeAsset {
    pos: Vector2<f32>,
    tile: Vector3<f32>,
    normal: Vector3<f32>,
    scale: f32,
    temp_probability: f32,
    moist_probability: f32,
    key: String,
}

impl NodeAsset {
    fn new(rng: &mut Pcg64, x: f32, z: f32, size: f32, world: &WorldData, mesh: &assets::Mesh, key: String) -> Self {
        let m = vec2(x + (rng.gen::<f32>() - 0.5) * size, z + (rng.gen::<f32>() - 0.5) * size);
        let scale = rng.gen_range(mesh.size_range[0]..mesh.size_range[1]);
        let (temp, moist) = world.map.get_biome(m);

        let temp_probability = if temp < mesh.temp_preferred {
            (mesh.temp_preferred - mesh.temp_range[0]).abs() / 1.0 - (mesh.temp_preferred - temp).abs()
        } else {
            (mesh.temp_preferred - mesh.temp_range[1]).abs() / 1.0 - (mesh.temp_preferred - temp).abs()
        };

        let moist_probability = if moist < mesh.moist_preferred {
            (mesh.moist_preferred - mesh.moist_range[0]).abs() / 1.0 - (mesh.moist_preferred - moist).abs()
        } else {
            (mesh.moist_preferred - mesh.moist_range[1]).abs() / 1.0 - (mesh.moist_preferred - moist).abs()
        };

        let (pos, _) = world.map.get_position_normal(m);
        let normal = get_offsets(mesh.radius, scale)
            .iter()
            .map(|o| world.map.get_smooth_normal(vec2(pos.x + o.x, pos.z + o.z)))
            .sum::<Vector3<f32>>()
            / 4.0;

        Self {
            pos: m,
            tile: pos,
            normal,
            scale,
            temp_probability,
            moist_probability,
            key,
        }
    }

    fn validate(&self, mesh: &assets::Mesh) -> bool {
        let seed = format!("{}_CREATE_{}_{}_{}", settings::MAP_SEED, self.tile.x, self.tile.z, self.key);
        let mut rng: Pcg64 = Seeder::from(seed).make_rng();

        if 1.0 - self.normal.y < mesh.slope_range[0] || 1.0 - self.normal.y > mesh.slope_range[1] {
            return false;
        }

        if self.temp_probability < rng.gen() || self.moist_probability < rng.gen() {
            return false;
        }

        if self.tile.y <= 0.0 {
            return false;
        }

        true
    }

    fn create_instance(&self, mesh: &assets::Mesh, world: &WorldData) -> pipelines::assets::Instance {
        let seed = format!("{}_TRANSFORM_{}_{}_{}", settings::MAP_SEED, self.tile.x, self.tile.z, self.key);
        let mut rng: Pcg64 = Seeder::from(seed).make_rng();

        let elev = get_offsets(mesh.radius, self.scale)
            .iter()
            .map(|o| {
                let p = vec3(self.tile.x + o.x, self.tile.y, self.tile.z + o.z);
                world.map.get_smooth_elevation(self.pos, (p, self.normal))
            })
            .sum::<f32>()
            / 4.0;

        let mut r = Quaternion::from(Euler {
            x: get_rot_deg(mesh.rotation[0], &mut rng),
            y: get_rot_deg(mesh.rotation[1], &mut rng),
            z: get_rot_deg(mesh.rotation[2], &mut rng),
        });

        if mesh.align {
            r = Quaternion::from_arc(Vector3::unit_y(), self.normal, Some(self.normal)) * r;
        }

        let t = Matrix4::from_translation(vec3(self.pos.x, elev, self.pos.y)) * Matrix4::from(r) * Matrix4::from_scale(self.scale);
        pipelines::assets::Instance { transform: t.into() }
    }
}

fn get_rot_deg(rot: (f32, f32), rng: &mut Pcg64) -> Rad<f32> {
    Rad(if rot.0 - rot.1 < 0.0 {
        rng.gen_range(rot.0..rot.1) * (PI / 180.0)
    } else {
        0.0
    })
}

fn get_offsets(radius: f32, scale: f32) -> Vec<Vector3<f32>> {
    let radius = radius * scale;
    vec![
        vec3(-radius, 0.0, -radius),
        vec3(radius, 0.0, -radius),
        vec3(-radius, 0.0, radius),
        vec3(radius, 0.0, radius),
    ]
}

pub fn create_assets(x: f32, z: f32, size: f32, world: &WorldData, mesh: &assets::Mesh, key: &str) -> Vec<pipelines::assets::Instance> {
    let seed = format!("{}_NODE_{}_{}_{}", settings::MAP_SEED, x, z, key);
    let mut rng: Pcg64 = Seeder::from(seed).make_rng();
    let mut count = (rng.gen::<f32>() * mesh.density.floor() * settings::TILE_SIZE as f32 / 20.0) as usize;

    if rng.gen::<f32>() < mesh.density.fract() {
        count += 1;
    }

    (0..count)
        .map(|_| NodeAsset::new(&mut rng, x, z, size, world, mesh, key.to_string()))
        .filter(|node_asset| node_asset.validate(&mesh))
        .map(|node_asset| node_asset.create_instance(mesh, world))
        .collect()
}
