use crate::{pipelines::render_targets, settings};
pub struct Args<'t> {
    pub color_targets: &'t [&'t String],
    pub depth_target: Option<&'t String>,
    pub clear_color: bool,
    pub clear_depth: bool,
    pub bundles: Vec<&'t wgpu::RenderBundle>,
}

pub fn render(label: &str, encoder: &mut wgpu::CommandEncoder, render_targets: &render_targets::RenderTargets, args: Args) {
    let ops = wgpu::Operations {
        load: match args.clear_color {
            true => wgpu::LoadOp::Clear(settings::CLEAR_COLOR),
            false => wgpu::LoadOp::Load,
        },
        store: true,
    };

    let depth_stencil_attachment = match args.depth_target {
        Some(attachment) => Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
            attachment: render_targets.get(attachment),
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
                    attachment: render_targets.get(attachment),
                    resolve_target: None,
                    ops,
                })
                .collect::<Vec<wgpu::RenderPassColorAttachmentDescriptor>>()
                .as_slice(),
            depth_stencil_attachment,
        })
        .execute_bundles(args.bundles.into_iter());
}
