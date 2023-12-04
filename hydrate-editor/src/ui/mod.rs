pub mod components;
pub mod drag_drop;
pub mod modals;

pub fn add_spacing(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
) {
    let prev_item_spacing = ui.spacing_mut().item_spacing;
    ui.spacing_mut().item_spacing.x = 0.0;
    ui.allocate_space(egui::vec2(width, height));
    ui.spacing_mut().item_spacing = prev_item_spacing;
}

pub fn add_icon_spacing(ui: &mut egui::Ui) {
    add_spacing(ui, ui.spacing().indent, 1.0);
}

pub fn add_indent_spacing(ui: &mut egui::Ui) {
    //add_spacing(ui, ui.spacing().icon_width / 2.0, 1.0);
    add_spacing(ui, ui.spacing().indent / 2.0, 1.0);
}
