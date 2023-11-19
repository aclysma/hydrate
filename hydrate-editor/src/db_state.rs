use hydrate_model::pipeline::import_util::ImportToQueue;
use hydrate_model::pipeline::ImporterRegistry;
use hydrate_model::{
    AssetId, AssetLocation, AssetName, DataSet, EditorModel, PathNode, PathNodeRoot,
    SchemaCacheSingleFile, SchemaLinker, SchemaSet, SchemaSetBuilder,
};
use std::path::{Path, PathBuf};

pub struct DbState {
    //pub db: hydrate_model::Database,
    //pub undo_stack: hydrate_model::UndoStack,
    pub editor_model: EditorModel,
    schema_cache_file_path: PathBuf,
}

impl DbState {
    // fn schema_def_path() -> PathBuf {
    //     PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/schema"))
    // }
    //
    // fn asset_data_source_path() -> PathBuf {
    //     PathBuf::from(concat!(
    //         env!("CARGO_MANIFEST_DIR"),
    //         "/data/assets"
    //     ))
    // }

    // pub fn import_data_source_path() -> PathBuf {
    //     PathBuf::from(concat!(
    //     env!("CARGO_MANIFEST_DIR"),
    //     "/data/import_data"
    //     ))
    // }
    //
    // pub fn build_data_source_path() -> PathBuf {
    //     PathBuf::from(concat!(
    //     env!("CARGO_MANIFEST_DIR"),
    //     "/data/build_data"
    //     ))
    // }

    // fn data_file_path() -> PathBuf {
    //     PathBuf::from(concat!(
    //         env!("CARGO_MANIFEST_DIR"),
    //         "/data/data_source/data_file.nxt"
    //     ))
    // }

    // fn schema_cache_file_path() -> PathBuf {
    //     PathBuf::from(concat!(
    //         env!("CARGO_MANIFEST_DIR"),
    //         "/data/schema_cache_file.json"
    //     ))
    // }

    fn init_empty_model(
        schema_set: SchemaSet,
        importer_registry: &ImporterRegistry,
        asset_id_based_data_path: &Path,
        asset_path_based_data_path: &Path,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) -> EditorModel {
        //let mut undo_stack = UndoStack::default();
        //let mut db = hydrate_model::Database::new(Arc::new(schema_set), &undo_stack);
        let mut db = DataSet::default();

        let mut edit_model = EditorModel::new(schema_set.clone());

        let asset_source_id = edit_model.add_file_system_id_based_asset_source(
            "id_file_system",
            asset_id_based_data_path,
            imports_to_queue,
        );
        let _asset_source_path = edit_model.add_file_system_path_based_data_source(
            "path_file_system",
            asset_path_based_data_path,
            importer_registry,
            imports_to_queue,
        );

        // let file_system = edit_model
        //     .file_system_treedata_source(tree_source_id)
        //     .unwrap();

        let path_node_schema_record = schema_set
            .find_named_type(PathNode::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let transform_schema_record = schema_set
            .find_named_type("Transform")
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let root_asset_id = AssetId::from_uuid(*asset_source_id.uuid());
        // db.new_asset_with_id(
        //     root_asset_id,
        //     AssetName::new("root_asset"),
        //     AssetLocation::null(),
        //     &path_node_schema_asset,
        // ).unwrap();

        let subdir_obj = db.new_asset(
            AssetName::new("subdir"),
            AssetLocation::new(root_asset_id),
            &path_node_schema_record,
        );

        let subdir2_obj = db.new_asset(
            AssetName::new("subdir2"),
            AssetLocation::new(subdir_obj),
            &path_node_schema_record,
        );

        let asset_location = AssetLocation::new(subdir2_obj);

        let prototype_obj = db.new_asset(
            AssetName::new("asset_a"),
            asset_location.clone(),
            &transform_schema_record,
        );
        let instance_obj = db
            .new_asset_from_prototype(AssetName::new("asset_b"), asset_location, prototype_obj)
            .unwrap();

        db.set_property_override(
            &schema_set,
            prototype_obj,
            "position.x",
            Some(hydrate_model::Value::F64(10.0)),
        )
        .unwrap();
        db.set_property_override(
            &schema_set,
            instance_obj,
            "position.x",
            Some(hydrate_model::Value::F64(20.0)),
        )
        .unwrap();

        let _prototype_array_element_1 = db.add_dynamic_array_override(
            &schema_set,
            prototype_obj,
            "all_fields.dynamic_array_i32",
        );
        let _prototype_array_element_2 = db.add_dynamic_array_override(
            &schema_set,
            prototype_obj,
            "all_fields.dynamic_array_i32",
        );
        let _instance_array_element_1 = db.add_dynamic_array_override(
            &schema_set,
            instance_obj,
            "all_fields.dynamic_array_i32",
        );
        let _instance_array_element_2 = db.add_dynamic_array_override(
            &schema_set,
            instance_obj,
            "all_fields.dynamic_array_i32",
        );
        let _instance_array_element_3 = db.add_dynamic_array_override(
            &schema_set,
            instance_obj,
            "all_fields.dynamic_array_i32",
        );

        let _prototype_array_element_1 = db.add_dynamic_array_override(
            &schema_set,
            prototype_obj,
            "all_fields.dynamic_array_vec3",
        );
        let _prototype_array_element_2 = db.add_dynamic_array_override(
            &schema_set,
            prototype_obj,
            "all_fields.dynamic_array_vec3",
        );
        let _instance_array_element_1 = db.add_dynamic_array_override(
            &schema_set,
            instance_obj,
            "all_fields.dynamic_array_vec3",
        );
        let _instance_array_element_2 = db.add_dynamic_array_override(
            &schema_set,
            instance_obj,
            "all_fields.dynamic_array_vec3",
        );
        let _instance_array_element_3 = db.add_dynamic_array_override(
            &schema_set,
            instance_obj,
            "all_fields.dynamic_array_vec3",
        );

        edit_model
            .root_edit_context_mut()
            .restore_assets_from(db)
            .unwrap();
        edit_model
    }

    fn try_load(
        schema_set: SchemaSet,
        importer_registry: &ImporterRegistry,
        asset_id_based_data_path: &Path,
        asset_path_based_data_path: &Path,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) -> Option<EditorModel> {
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
        if editor_model.root_edit_context().assets().len() == 0 {
            None
        } else {
            Some(editor_model)
        }
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
    pub fn load_or_init_empty(
        schema_set: &SchemaSet,
        importer_registry: &ImporterRegistry,
        asset_id_based_data_path: &Path,
        asset_path_based_data_path: &Path,
        schema_cache_file_path: &Path,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) -> Self {
        let editor_model = if let Some(loaded) = Self::try_load(
            schema_set.clone(),
            importer_registry,
            asset_id_based_data_path,
            asset_path_based_data_path,
            imports_to_queue,
        ) {
            loaded
        } else {
            Self::init_empty_model(
                schema_set.clone(),
                importer_registry,
                asset_id_based_data_path,
                asset_path_based_data_path,
                imports_to_queue,
            )
        };

        DbState {
            editor_model,
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
