use hydrate_model::{SchemaFingerprint, SchemaNamedType, SchemaRecord, SchemaSet};
use std::hash::Hash;

// Based on
// Could be improved using this example https://github.com/emilk/egui/pull/777/files

pub fn schema_record_selector(
    ui: &mut egui::Ui,
    id: impl Hash,
    schema_name: &mut String,
    schema_set: &SchemaSet,
) -> egui::InnerResponse<Option<SchemaRecord>> {
    // let popup_id = egui::Id::new(id);
    // let mut r = ui.text_edit_singleline(schema_name);
    // if r.gained_focus() {
    //     ui.memory_mut(|m| m.open_popup(popup_id));
    // }
    //
    // let mut changed = false;
    // egui::popup_below_widget(ui, popup_id, &r, |ui| {
    //     egui::ScrollArea::vertical().show(ui, |ui| {
    //         for (fingerprint, named_type) in schema_set.schemas() {
    //             if named_type.try_as_record().is_some() {
    //                 let name = named_type.name();
    //                 if name.to_ascii_lowercase().contains(&schema_name.to_ascii_lowercase()) {
    //                     ui.label(name);
    //                 }
    //
    //             }
    //         }
    //     });
    // });
    //
    // if changed {
    //     r.mark_changed();
    // }
    //
    // r

    // for (fingerprint, named_type) in schema_set.schemas() {
    //     if named_type.try_as_record().is_some() {
    //         let name = named_type.name();
    //         if name.to_ascii_lowercase().contains(&schema_name.to_ascii_lowercase()) {
    //             ui.label(name);
    //         }
    //
    //     }
    // }

    let search: Vec<_> = schema_set
        .schemas()
        .values()
        .filter(|x| x.try_as_record().is_some())
        .map(|x| x.name())
        .collect();
    let response = ui.add(egui_autocomplete::AutoCompleteTextEdit::new(
        schema_name,
        search,
    ));
    let fingerprint = schema_set
        .try_find_named_type(schema_name)
        .map(|x| x.try_as_record())
        .flatten()
        .cloned();
    egui::InnerResponse::new(fingerprint, response)
}
