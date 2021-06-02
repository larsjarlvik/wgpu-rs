use conrod_core::*;

use crate::ui::config;

widget_ids! {
    pub struct Ids {
        canvas,
        title,
    }
}

pub fn create(ui: &mut Ui, message: &str) {
    let ids = Ids::new(ui.widget_id_generator());
    let mut widgets = ui.set_widgets();

    widget::Canvas::new().pad(config::SPACING_XL).set(ids.canvas, &mut widgets);
    widget::Text::new(message)
        .bottom_left_of(ids.canvas)
        .color(color::WHITE)
        .font_size(config::FONT_L)
        .set(ids.title, &mut widgets);
}
