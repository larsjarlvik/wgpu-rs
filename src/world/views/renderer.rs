use crate::settings;
pub struct Args<'t> {
    pub color_targets: &'t [&'t wgpu::TextureView],
    pub depth_target: Option<&'t wgpu::TextureView>,
    pub clear_color: bool,
    pub clear_depth: bool,
    pub bundles: Vec<&'t wgpu::RenderBundle>,
}

pub fn render(label: &str, encoder: &mut wgpu::CommandEncoder, args: Args) {
    optick::event!();
    let ops = wgpu::Operations {
        load: match args.clear_color {
            true => wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            false => wgpu::LoadOp::Load,
        },
        store: true,
    };

    let depth_stencil_attachment = match args.depth_target {
        Some(attachment) => Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment,
            depth_ops: Some(wgpu::Operations {
                load: match args.clear_depth {
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
            color_attachments: args
                .color_targets
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
        .execute_bundles(args.bundles.into_iter());
}
