use conrod_core::*;

widget_ids! {
    pub struct Ids {
        canvas,
        title,
    }
}

pub fn create(ui: &mut Ui, message: &str) {
    let ids = Ids::new(ui.widget_id_generator());
    let mut widgets = ui.set_widgets();

    widget::Canvas::new().pad(36.0).set(ids.canvas, &mut widgets);
    widget::Text::new(message)
        .bottom_left_of(ids.canvas)
        .color(color::WHITE)
        .font_size(32)
        .set(ids.title, &mut widgets);
}
