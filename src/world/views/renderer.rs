use crate::settings;

pub fn render(
    label: &str,
    encoder: &mut wgpu::CommandEncoder,
    color_targets: &[&wgpu::TextureView],
    depth_target: Option<&wgpu::TextureView>,
    clear_color: bool,
    clear_depth: bool,
    bundles: Vec<&wgpu::RenderBundle>,
) {
    let ops = wgpu::Operations {
        load: match clear_color {
            true => wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            false => wgpu::LoadOp::Load,
        },
        store: true,
    };

    let depth_stencil_attachment = match depth_target {
        Some(attachment) => Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment,
            depth_ops: Some(wgpu::Operations {
                load: match clear_depth {
                    true => wgpu::LoadOp::Clear(1.0),
                    false => wgpu::LoadOp::Load,
                },
                store: true,
            }),
            stencil_ops: None,
        }),
        None => None,
    };

    encoder
        .begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: color_targets
                .iter()
                .map(|attachment| wgpu::RenderPassColorAttachmentDescriptor {
                    attachment,
                    resolve_target: None,
                    ops,
                })
                .collect::<Vec<wgpu::RenderPassColorAttachmentDescriptor>>()
                .as_slice(),
            depth_stencil_attachment,
        })
        .execute_bundles(bundles.into_iter());
}
