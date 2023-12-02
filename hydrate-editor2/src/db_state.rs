use hydrate_model::pipeline::import_util::ImportToQueue;
use hydrate_model::pipeline::ImporterRegistry;
use hydrate_model::{
    AssetId, AssetLocation, AssetName, AssetPathCache, DataSet, EditorModel, LocationTree,
    PathNode, PathNodeRoot, SchemaCacheSingleFile, SchemaLinker, SchemaSet, SchemaSetBuilder,
};
use std::path::{Path, PathBuf};

pub struct DbState {
    //pub db: hydrate_model::Database,
    //pub undo_stack: hydrate_model::UndoStack,
    pub editor_model: EditorModel,
    //pub asset_path_cache: AssetPathCache,
    //pub location_tree: LocationTree,
    schema_cache_file_path: PathBuf,
}

impl DbState {
    fn do_load(
        schema_set: SchemaSet,
        importer_registry: &ImporterRegistry,
        asset_id_based_data_path: &Path,
        asset_path_based_data_path: &Path,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) -> EditorModel {
        let mut editor_model = EditorModel::new(schema_set);
        editor_model.add_file_system_id_based_asset_source(
            "id_file_system",
            asset_id_based_data_path,
            imports_to_queue,
        );
        editor_model.add_file_system_path_based_data_source(
            "path_file_system",
            asset_path_based_data_path,
            importer_registry,
            imports_to_queue,
        );

        editor_model
    }

    #[profiling::function]
    pub fn load_schema(
        mut linker: SchemaLinker,
        schema_def_paths: &[&Path],
        schema_cache_file_path: &Path,
    ) -> SchemaSet {
        let mut schema_set = SchemaSetBuilder::default();

        PathNode::register_schema(&mut linker);
        PathNodeRoot::register_schema(&mut linker);
        for path in schema_def_paths {
            linker.add_source_dir(path, "**.json").unwrap();
        }
        schema_set.add_linked_types(linker).unwrap();

        if let Some(schema_cache_str) = std::fs::read_to_string(schema_cache_file_path).ok() {
            let named_types = SchemaCacheSingleFile::load_string(&schema_cache_str);
            schema_set.restore_named_types(named_types);
        }

        schema_set.build()
    }

    #[profiling::function]
    pub fn load(
        schema_set: &SchemaSet,
        importer_registry: &ImporterRegistry,
        asset_id_based_data_path: &Path,
        asset_path_based_data_path: &Path,
        schema_cache_file_path: &Path,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) -> Self {
        let editor_model = Self::do_load(
            schema_set.clone(),
            importer_registry,
            asset_id_based_data_path,
            asset_path_based_data_path,
            imports_to_queue,
        );

        //let asset_path_cache = AssetPathCache::build(&editor_model);
        //let location_tree = LocationTree::build(&editor_model, &asset_path_cache);
        DbState {
            editor_model,
            //asset_path_cache,
            //location_tree,
            schema_cache_file_path: schema_cache_file_path.to_path_buf(),
        }
    }

    pub fn save(&mut self) {
        log::debug!("saving schema cache to {:?}", self.schema_cache_file_path);
        let schema_cache =
            SchemaCacheSingleFile::store_string(self.editor_model.schema_set().schemas());
        std::fs::write(&self.schema_cache_file_path, schema_cache).unwrap();

        self.editor_model.save_root_edit_context();
    }
}
