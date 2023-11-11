use crate::ui_state::AssetBrowserGridState;
use hydrate_model::AssetId;
use imgui::im_str;

#[derive(Copy, Clone, Debug)]
pub enum AssetBrowserGridPayload {
    Single(AssetId),
    AllSelected,
}

pub fn asset_browser_grid_assets_drag_source(
    ui: &imgui::Ui,
    grid_state: &AssetBrowserGridState,
    dragged_asset: AssetId,
) {
    let payload = if grid_state.selected_items.len() > 1
        && grid_state.selected_items.contains(&dragged_asset)
    {
        // If it's multiple assets, have the receiver look at selected objects
        AssetBrowserGridPayload::AllSelected
    } else {
        AssetBrowserGridPayload::Single(dragged_asset)
    };

    imgui::DragDropSource::new(im_str!("ASSET_BROWSER_GRID_SELECTION")).begin_payload(ui, payload);
}

pub fn asset_browser_grid_assets_drag_target_printf(
    ui: &imgui::Ui,
    grid_state: &AssetBrowserGridState,
) -> Option<AssetBrowserGridPayload> {
    if let Some(target) = imgui::DragDropTarget::new(ui) {
        if let Some(payload) = target.accept_payload::<AssetBrowserGridPayload>(
            im_str!("ASSET_BROWSER_GRID_SELECTION"),
            imgui::DragDropFlags::empty(),
        ) {
            match payload.unwrap().data {
                AssetBrowserGridPayload::Single(asset_id) => {
                    println!("received payload {:?}", asset_id.as_uuid());
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
