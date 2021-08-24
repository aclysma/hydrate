use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

pub fn run() {
    let mut window_descriptor = WindowDescriptor::default();
    window_descriptor.title = "m3".to_string();

    App::build()
        .insert_resource(window_descriptor)
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_system(ui_example.system())
        .run();
}

fn ui_example(egui_context: ResMut<EguiContext>) {
    egui::SidePanel::left("left_panel").show(egui_context.ctx(), |ui| {
        ui.label("hi");
    });
}
