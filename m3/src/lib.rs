use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};

mod hashing;
use hashing::*;
use bevy_egui::egui::Widget;

#[derive(Default)]
struct UiState {
    selected_items: HashSet<String>
}

pub fn run() {
    let mut window_descriptor = WindowDescriptor::default();
    window_descriptor.title = "m3".to_string();

    App::build()
        .insert_resource(window_descriptor)
        .insert_resource(UiState::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_system(ui_update.system())
        .run();
}

fn ui_update(egui_context: ResMut<EguiContext>, ui_state: ResMut<UiState>) {
    let mut style = egui::Style::default();
    style.animation_time = style.animation_time / 2.0;
    egui_context.ctx().set_style(style);

    egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .min_height(32.0)
        .show(egui_context.ctx(), |ui| {
            egui::ScrollArea::auto_sized().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Expandable Upper Panel");
                });
                // ui.add(egui::Label::new("some text").small().weak());
                // ui.add(egui::Label::new("some text").small().weak());
                // ui.add(egui::Label::new("some text").small().weak());
                // ui.add(egui::Label::new("some text").small().weak());
                // ui.add(egui::Label::new("some text").small().weak());
                // ui.add(egui::Label::new("some text").small().weak());
                // ui.add(egui::Label::new("some text").small().weak());
            });
        });

    // egui::TopBottomPanel::top("bottom")
    //     .resizable(true)
    //     .show(egui_context.ctx(), |ui| {
    //     //ui.allocate_space(egui::Vec2::new(0.0, ui.available_height()));
    //     // let s = ui.available_size();
    //     // dbg!(s);
    //     // ui.horizontal(|ui| {
    //     //     ui.label("hi 2");
    //     // });
    //
    //     egui::Separator::default().vertical().ui(ui);
    //
    //     ui.label("hi");
    //     ui.label("hi");
    //     ui.label("hi");
    //     ui.label("hi");
    //     ui.label("hi");
    //
    //
    //
    //     egui::ScrollArea::auto_sized().show(ui, |ui| {
    //         ui.label("x");
    //         ui.label("x");
    //         ui.label("x");
    //         ui.label("x");
    //     });
    // });

    egui::SidePanel::left("left_panel").show(egui_context.ctx(), |ui| {
        ui.label("hi");


        ui.collapsing("folder1", |ui| {
            ui.collapsing("subfolder1", |ui| {
                ui.selectable_label(true, "subitem1");
                ui.selectable_label(false, "subitem2");
                ui.selectable_label(false, "subitem3");
            });

            ui.label("item1");
            ui.label("item2");
        });
    });
}
