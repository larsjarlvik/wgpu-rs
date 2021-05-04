use super::primitive;

pub struct Mesh {
    pub name: String,
    pub primitives: Vec<primitive::Primitive>,
}

impl Mesh {
    pub fn new<'a>(mesh: gltf::Mesh<'_>, buffers: &Vec<gltf::buffer::Data>) -> Self {
        let mut primitives = vec![];
        let name = String::from(mesh.name().unwrap());

        for primitive in mesh.primitives() {
            primitives.push(primitive::Primitive::new(buffers, &primitive, &name));
        }

        Mesh { name, primitives }
    }
}
