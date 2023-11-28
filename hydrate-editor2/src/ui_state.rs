use hydrate_model::{AssetId, AssetName, AssetPath, AssetPathCache, EditorModel, HashMap, HashSet, LocationTree, SchemaRecord};


// Keyboard shortcuts? Associate with actions/commands?
// Action string names and tooltips?
// Actions icons?
// General way of knowing if an action is eligible to run?
// Command pallete?
// How to handle modals?
// Dirty detection?
// Cache of things like asset path tree?

pub enum UiCommands {
    Quit,
}


trait ModalWindow {
    fn draw(ui: &mut egui::Ui);
}


pub struct AssetInfo {
    // Everything needed to draw in the asset gallery view
    pub id: AssetId,
    pub name: AssetName,
    pub path: AssetPath,
    pub schema: SchemaRecord,
    pub is_dirty: bool,
    pub is_generated: bool,
    // schema?
    // thumbnail? maybe as a content-addressable hash that makes it easy to cache last N thumbnails
}

pub struct AssetDetails {
    // All property values?
    // Maybe there is some ddata per-asset and some summarized data based on the selected assets
}

pub enum PipelineState {
    Idle,
    Importing,
    Building,
}

pub struct EditorModelUiState {
    // Sorted by something?
    pub path_lookup: AssetPathCache,
    pub location_tree: LocationTree,
    pub all_asset_info: HashMap<AssetId, AssetInfo>,
    pub all_selected_asset_details: Vec<AssetDetails>,
    pub selected_assets: HashSet<AssetId>,
    // any type of caching of paths/PathNode?
    // any kind of dirty state?
    pub errors: Vec<String>,
    pub current_operation: PipelineState,
}

impl Default for EditorModelUiState {
    fn default() -> Self {
        EditorModelUiState {
            path_lookup: AssetPathCache::empty(),
            location_tree: Default::default(),
            all_asset_info: Default::default(),
            all_selected_asset_details: vec![],
            selected_assets: Default::default(),
            errors: vec![],
            current_operation: PipelineState::Idle,

        }
    }
}

impl EditorModelUiState {
    pub fn update(&mut self, editor_model: &EditorModel) {
        self.path_lookup = AssetPathCache::build(editor_model);
        self.location_tree = LocationTree::build(&editor_model, &self.path_lookup);

        let root_edit_context = editor_model.root_edit_context();

        self.all_asset_info.clear();
        self.all_asset_info.reserve(root_edit_context.assets().len());
        for (asset_id, asset_info) in editor_model.root_edit_context().assets() {
            let path = editor_model.asset_path(*asset_id, &self.path_lookup);

            let old = self.all_asset_info.insert(*asset_id, AssetInfo {
                id: *asset_id,
                name: asset_info.asset_name().clone(),
                path,
                schema: asset_info.schema().clone(),
                is_dirty: root_edit_context.is_asset_modified(*asset_id),
                is_generated: editor_model.is_generated_asset(*asset_id)
            });
            assert!(old.is_none());
        }
    }
}
