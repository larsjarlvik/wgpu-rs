use super::primitive::Primitive;

pub struct Mesh {
    pub name: String,
    pub primitives: Vec<Primitive>,
}

impl Mesh {
    pub fn new<'a>(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mesh: gltf::Mesh<'_>,
        buffers: &Vec<gltf::buffer::Data>,
        images: &Vec<gltf::image::Data>,
    ) -> Self {
        let mut primitives = vec![];
        let name = String::from(mesh.name().unwrap());

        for primitive in mesh.primitives() {
            primitives.push(Primitive::new(device, queue, buffers, images, &primitive));
        }

        Mesh { name, primitives }
    }
}
