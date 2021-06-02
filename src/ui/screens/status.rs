use conrod_core::*;

use crate::ui::config;

widget_ids! {
    pub struct Ids {
        canvas,
        column,
        fps,
        anti_aliasing,
    }
}

pub struct Status {
    pub fps: u32,
    pub anti_aliasing: String,
}

pub fn create(ui: &mut Ui, status: &Status) {
    let ids = Ids::new(ui.widget_id_generator());
    let mut widgets = ui.set_widgets();

    widget::Canvas::new()
        .color(color::TRANSPARENT)
        .border_color(color::TRANSPARENT)
        .pad(18.0)
        .set(ids.canvas, &mut widgets);

    widget::Text::new(format!("FPS: {}", status.fps).as_str())
        .top_left_of(ids.canvas)
        .color(color::BLACK)
        .font_size(config::FONT_S)
        .set(ids.fps, &mut widgets);

    widget::Text::new(format!("Anti Aliasing: {}", status.anti_aliasing).as_str())
        .down_from(ids.fps, config::SPACING_S)
        .color(color::BLACK)
        .font_size(config::FONT_S)
        .set(ids.anti_aliasing, &mut widgets);
}
