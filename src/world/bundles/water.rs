use crate::{
    camera, plane, settings,
    world::{self, node::Node},
};
use cgmath::*;

pub fn get_water_bundle(
    device: &wgpu::Device,
    camera: &camera::Instance,
    world_data: &world::WorldData,
    nodes: &Vec<&Node>,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: Some("water_bundle"),
        color_formats: &[settings::COLOR_TEXTURE_FORMAT],
        depth_stencil_format: Some(settings::DEPTH_TEXTURE_FORMAT),
        sample_count: 1,
    });
    encoder.set_pipeline(&world_data.water.render_pipeline);
    encoder.set_bind_group(0, &camera.uniforms.bind_group, &[]);
    encoder.set_bind_group(1, &world_data.water.uniforms.bind_group, &[]);
    encoder.set_bind_group(2, &world_data.water.noise_bindings.bind_group, &[]);
    encoder.set_bind_group(3, &world_data.water.texture_bind_group, &[]);
    encoder.set_bind_group(5, &world_data.lights.uniforms.bind_group, &[]);
    encoder.set_bind_group(6, &world_data.lights.texture_bind_group, &[]);
    encoder.set_vertex_buffer(0, world_data.water.vertex_buffer.slice(..));

    let plane = camera.uniforms.data.clip[3];
    let eye = vec3(
        camera.uniforms.data.eye_pos[0],
        camera.uniforms.data.eye_pos[1],
        camera.uniforms.data.eye_pos[2],
    );

    for node in nodes {
        for lod in 0..=settings::LODS.len() {
            let water_lod = world_data.lods.get(lod).expect("Could not get LOD!");

            if check_clip(plane, node.bounding_box.min.y)
                && plane::get_lod(eye, vec3(node.x, 0.0, node.z), camera.uniforms.data.z_far) == lod as u32
            {
                if let Some(data) = &node.data {
                    let ct = plane::get_connect_type(eye, vec3(node.x, 0.0, node.z), lod as u32, camera.uniforms.data.z_far);
                    let lod_buffer = water_lod.get(&ct).unwrap();
                    encoder.set_bind_group(4, &data.uniforms.bind_group, &[]);
                    encoder.set_index_buffer(lod_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    encoder.draw_indexed(0..lod_buffer.length, 0, 0..1);
                }
            }
        }
    }

    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some("water") })
}

fn check_clip(plane: f32, y_min: f32) -> bool {
    y_min <= plane
}
