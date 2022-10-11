use imgui::im_str;
use nexdb::ObjectId;
use crate::app_state::AssetBrowserGridState;

#[derive(Copy, Clone, Debug)]
pub enum AssetBrowserGridPayload {
    Single(ObjectId),
    AllSelected
}

pub fn asset_browser_grid_drag_source(ui: &imgui::Ui, grid_state: &AssetBrowserGridState, object_id: ObjectId) {
    let payload = if grid_state.selected_items.len() > 1 && grid_state.selected_items.contains(&object_id) {
        // If it's multiple objects, have the receiver look at selected objects
        AssetBrowserGridPayload::AllSelected
    } else {
        AssetBrowserGridPayload::Single(object_id)
    };

    imgui::DragDropSource::new(im_str!("MOCK_SOURCE"))
        .begin_payload(ui, payload);
}

pub fn asset_browser_grid_drag_target(ui: &imgui::Ui, grid_state: &AssetBrowserGridState) -> Option<AssetBrowserGridPayload> {
    if let Some(target) = imgui::DragDropTarget::new(ui) {
        if let Some(payload) = target.accept_payload::<AssetBrowserGridPayload>(im_str!("MOCK_SOURCE"), imgui::DragDropFlags::empty()) {
            match payload.unwrap().data {
                AssetBrowserGridPayload::Single(object_id) => {
                    println!("received payload {:?}", object_id.as_uuid());
                }
                AssetBrowserGridPayload::AllSelected => {
                    println!("received payload {:?}", grid_state.selected_items);
                }
            }

            return Some(payload.unwrap().data);
        }
    }

    None
}