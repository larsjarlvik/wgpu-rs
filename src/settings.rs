use cgmath::*;

pub const COLOR_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
pub const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.1,
    b: 0.1,
    a: 1.0,
};
pub const CAMERA_ACCELERATION: f32 = 10.0;
pub const CAMERA_FRICTION: f32 = 1.0;
pub const CAMERA_SENSITIVITY: f32 = 10.0;
pub const HORIZONTAL_SCALE: f32 = 0.005;
pub const VERTICAL_SCALE: f32 = 150.0;
pub const SEA_LEVEL: f32 = 0.4;
pub const TERRAIN_OCTAVES: u32 = 4;
pub const TILE_SIZE: u32 = 80;
pub const TILE_DEPTH: u32 = 5;
pub static LODS: [f32; 3] = [0.1, 0.2, 0.3];

pub static SKY_COLOR: Vector3<f32> = vec3(0.312, 0.573, 0.757);
pub const SKY_FADE_DISTANCE: f32 = 0.5;

pub const LIGHT_DIR: Vector3<f32> = vec3(0.5, -1.0, 0.0);
pub static LIGHT_COLOR: Vector3<f32> = vec3(1.0, 0.9, 0.5);
pub const LIGHT_AMBIENT: f32 = 0.3;
pub const LIGHT_INTENSITY: f32 = 2.0;
pub const SHADOW_CASCADE_SPLITS: [f32; 4] = [0.05, 0.15, 0.3, 1.0];
pub const SHADOW_RESOLUTION: u32 = 4096;

pub const MAP_SEED: &str = "RANDOM_SEED";
