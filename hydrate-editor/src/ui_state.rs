use hydrate_model::{
    AssetPathCache, EditorModel, LocationTree,
};

trait ModalWindow {
    fn draw(ui: &mut egui::Ui);
}

pub struct EditorModelUiState {
    pub asset_path_cache: AssetPathCache,
    pub location_tree: LocationTree,
}

impl Default for EditorModelUiState {
    fn default() -> Self {
        EditorModelUiState {
            asset_path_cache: AssetPathCache::empty(),
            location_tree: Default::default(),
        }
    }
}

impl EditorModelUiState {
    pub fn update(
        &mut self,
        editor_model: &EditorModel,
    ) {
        self.asset_path_cache = AssetPathCache::build(editor_model);
        self.location_tree = LocationTree::build(&editor_model, &self.asset_path_cache);
    }
}
