use crate::texture;
use std::collections::HashMap;

pub struct Material {
    pub index: usize,
    pub base_color_texture: Option<wgpu::TextureView>,
    pub normal_texture: Option<wgpu::TextureView>,
    pub extras: HashMap<String, f32>,
}

impl Material {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, material: &gltf::Material, images: &Vec<gltf::image::Data>) -> Self {
        let pbr = material.pbr_metallic_roughness();
        let base_color_texture = match pbr.base_color_texture() {
            Some(texture) => {
                let image = images.iter().nth(texture.texture().source().index()).unwrap();
                Some(texture::create_mipmapped_view(
                    &device,
                    &queue,
                    &image.pixels,
                    image.width,
                    image.height,
                ))
            }
            None => None,
        };

        let normal_texture = match material.normal_texture() {
            Some(texture) => {
                let image = images.iter().nth(texture.texture().source().index()).unwrap();
                Some(texture::create_mipmapped_view(
                    &device,
                    &queue,
                    &image.pixels,
                    image.width,
                    image.height,
                ))
            }
            None => None,
        };

        let mut extras: HashMap<String, f32> = HashMap::new();
        if let Some(json) = material.extras() {
            extras = serde_json::from_str(json.get()).unwrap();
        }

        Self {
            index: material.index().unwrap(),
            base_color_texture,
            normal_texture,
            extras,
        }
    }
}
