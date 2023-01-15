use crate::ui_state::AssetBrowserGridState;
use hydrate_model::{HashSet, ObjectId};
use imgui::im_str;

#[derive(Copy, Clone, Debug)]
pub enum AssetBrowserGridPayload {
    Single(ObjectId),
    AllSelected,
}

pub fn asset_browser_grid_objects_drag_source(
    ui: &imgui::Ui,
    grid_state: &AssetBrowserGridState,
    dragged_object: ObjectId,
) {
    let payload = if grid_state.selected_items.len() > 1
        && grid_state.selected_items.contains(&dragged_object)
    {
        // If it's multiple objects, have the receiver look at selected objects
        AssetBrowserGridPayload::AllSelected
    } else {
        AssetBrowserGridPayload::Single(dragged_object)
    };

    imgui::DragDropSource::new(im_str!("ASSET_BROWSER_GRID_SELECTION")).begin_payload(ui, payload);
}

pub fn asset_browser_grid_objects_drag_target_printf(
    ui: &imgui::Ui,
    grid_state: &AssetBrowserGridState,
) -> Option<AssetBrowserGridPayload> {
    if let Some(target) = imgui::DragDropTarget::new(ui) {
        if let Some(payload) = target.accept_payload::<AssetBrowserGridPayload>(
            im_str!("ASSET_BROWSER_GRID_SELECTION"),
            imgui::DragDropFlags::empty(),
        ) {
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
