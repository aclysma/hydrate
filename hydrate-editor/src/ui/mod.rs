use hydrate_base::AssetId;
use hydrate_model::EditorModel;
use crate::image_loader::AssetThumbnailImageLoader;

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

const THUMBNAIL_STACK_DEPTH: usize = 3;
const THUMBNAIL_STACK_SPACING: f32 = 4.0;
const THUMBNAIL_STACK_SIZE_PER_ELEMENT: f32 = 64.0;

fn thumbnail_stack_size() -> egui::Vec2 {
    let size = THUMBNAIL_STACK_SIZE_PER_ELEMENT + THUMBNAIL_STACK_DEPTH as f32 * THUMBNAIL_STACK_SPACING;
    egui::vec2(size, size)
}

fn draw_thumbnail_stack(
    ui: &mut egui::Ui,
    editor_model: &EditorModel,
    thumbnail_image_loader: &AssetThumbnailImageLoader,
    primary_asset_id: AssetId,
    all_asset_ids: impl Iterator<Item = AssetId>,
) {

    // Put up to three of the assets in the list with the primary at index 0
    let mut assets_to_draw = vec![primary_asset_id];
    for asset_id in all_asset_ids {
        if assets_to_draw.contains(&asset_id) {
            continue;
        }

        assets_to_draw.push(asset_id);
        if assets_to_draw.len() >= THUMBNAIL_STACK_DEPTH {
            break;
        }
    }

    let spacing = THUMBNAIL_STACK_SPACING;
    let element_size = THUMBNAIL_STACK_SIZE_PER_ELEMENT;
    let (_, thumbnail_space) = ui.allocate_space(egui::vec2(
        element_size + spacing * (assets_to_draw.len() as f32),
        element_size + spacing * (assets_to_draw.len() as f32)
    ));
    for i in (0..assets_to_draw.len()).rev() {
        let i_f32 = i as f32;
        let child_rect = egui::Rect {
            min: thumbnail_space.min + egui::vec2(spacing * i_f32, spacing * i_f32),
            max: thumbnail_space.min + egui::vec2(element_size + spacing * i_f32, element_size + spacing * i_f32)
        };
        let mut child_ui = ui.child_ui(child_rect, egui::Layout::centered_and_justified(egui::Direction::LeftToRight));
        child_ui.add(egui::Image::new(
            thumbnail_image_loader.thumbnail_uri_for_asset(
                editor_model.root_edit_context(),
                assets_to_draw[i])).max_size(egui::vec2(element_size, element_size)));
    }
}