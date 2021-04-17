use crate::camera;

pub struct PrimitiveBuffers {
    pub texture_bind_group: wgpu::BindGroup,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
}

pub struct Model {
    pub primitives: Vec<PrimitiveBuffers>,
    pub bounding_box: camera::BoundingBox,
}
