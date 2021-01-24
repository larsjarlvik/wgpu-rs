use cgmath::Matrix4;

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
pub const TERRAIN_SCALE: f32 = 0.005;
pub const TERRAIN_OCTAVES: u32 = 5;
pub const TILE_SIZE: u32 = 40;
pub const TILE_DEPTH: i32 = 6;
pub static LODS: [f32; 3] = [0.2, 0.4, 0.6];
