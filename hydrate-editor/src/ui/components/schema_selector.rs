use hydrate_model::{SchemaFingerprint, SchemaNamedType, SchemaRecord, SchemaSet};
use std::hash::Hash;

// Based on
// Could be improved using this example https://github.com/emilk/egui/pull/777/files
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

pub fn schema_record_selector(
    ui: &mut egui::Ui,
    id: impl Hash,
    schema_name: &mut String,
    schema_set: &SchemaSet,
) -> egui::InnerResponse<Option<SchemaRecord>> {
    let search: Vec<_> = schema_set
        .schemas()
        .values()
        .filter_map(|x| x.try_as_record())
        .filter(|x| {
            x.markup().tags.contains("asset") && !x.markup().tags.contains("has_import_data")
        })
        .map(|x| x.markup().display_name.as_deref().unwrap_or(x.name()))
        .collect();
    let response = ui.add(egui_autocomplete::AutoCompleteTextEdit::new(
        schema_name,
        search,
    ));

    let mut find_by_name = None;
    let mut find_by_display_name = None;
    for (_, named_type) in schema_set.schemas() {
        if let Some(record_type) = named_type.try_as_record() {
            if named_type.name().to_lowercase() == schema_name.to_lowercase() {
                find_by_name = Some(record_type.clone());
                break;
            }

            if let Some(display_name) = &record_type.markup().display_name {
                if display_name.to_lowercase() == schema_name.to_lowercase() {
                    find_by_display_name = Some(record_type.clone());
                    break;
                }
            }
        }
    }

    egui::InnerResponse::new(find_by_name.or(find_by_display_name), response)
}
