use hydrate_base::hashing::HashSet;
use hydrate_base::AssetId;
use hydrate_model::{AssetPathCache, EditorModel, LocationTree, PendingFileOperations};

trait ModalWindow {
    fn draw(ui: &mut egui::Ui);
}

pub struct EditorModelUiState {
    pub asset_path_cache: AssetPathCache,
    pub edited_objects: HashSet<AssetId>,
    pub pending_file_operations: PendingFileOperations,
    pub location_tree: LocationTree,
}

impl Default for EditorModelUiState {
    fn default() -> Self {
        EditorModelUiState {
            asset_path_cache: AssetPathCache::empty(),
            edited_objects: Default::default(),
            pending_file_operations: PendingFileOperations::default(),
            location_tree: Default::default(),
        }
    }
}

impl EditorModelUiState {
    pub fn update(
        &mut self,
        editor_model: &EditorModel,
    ) {
        self.asset_path_cache = AssetPathCache::build(editor_model).unwrap();

        let mut edited_objects = HashSet::default();
        let pending_file_operations = editor_model.pending_file_operations();
        for (asset_id, _) in &pending_file_operations.create_operations {
            edited_objects.insert(*asset_id);
        }

        for (asset_id, _) in &pending_file_operations.modify_operations {
            edited_objects.insert(*asset_id);
        }

        for (asset_id, _) in &pending_file_operations.delete_operations {
            edited_objects.insert(*asset_id);
        }
        self.pending_file_operations = pending_file_operations;
        self.edited_objects = edited_objects;

        self.location_tree = LocationTree::build(&editor_model, &self.asset_path_cache);
    }
}
