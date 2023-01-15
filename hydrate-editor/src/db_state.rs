use hydrate_model::{DataSet, EditorModel, ObjectId, ObjectLocation, ObjectName, ObjectPath, PathNode, SchemaCacheSingleFile, SchemaLinker, SchemaSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
    // fn object_data_source_path() -> PathBuf {
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

    fn load_schema(mut linker: SchemaLinker, schema_def_path: &Path, schema_cache_file_path: &Path) -> SchemaSet {
        let mut schema_set = SchemaSet::default();

        PathNode::register_schema(&mut linker);
        linker.add_source_dir(schema_def_path, "**.json").unwrap();
        schema_set.add_linked_types(linker).unwrap();

        if let Some(schema_cache_str) = std::fs::read_to_string(schema_cache_file_path).ok()
        {
            SchemaCacheSingleFile::load_string(&mut schema_set, &schema_cache_str);
        }

        schema_set
    }

    fn init_empty_model(schema_set: Arc<SchemaSet>, asset_data_path: &Path) -> EditorModel {
        //let mut undo_stack = UndoStack::default();
        //let mut db = hydrate_model::Database::new(Arc::new(schema_set), &undo_stack);
        let mut db = DataSet::default();

        let mut edit_model = EditorModel::new(schema_set.clone());

        let object_source_id =
            edit_model.add_file_system_object_source(asset_data_path);

        // let file_system = edit_model
        //     .file_system_treedata_source(tree_source_id)
        //     .unwrap();

        let path_node_schema_object = schema_set
            .find_named_type(PathNode::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let transform_schema_object = schema_set
            .find_named_type("Transform")
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let subdir_obj = db.new_object(
            ObjectName::new("subdir"),
            ObjectLocation::new(object_source_id, ObjectId::null()),
            &path_node_schema_object,
        );

        let subdir2_obj = db.new_object(
            ObjectName::new("subdir2"),
            ObjectLocation::new(object_source_id, subdir_obj),
            &path_node_schema_object,
        );

        let object_location = ObjectLocation::new(object_source_id, subdir2_obj);

        let prototype_obj = db.new_object(
            ObjectName::new("object_a"),
            object_location.clone(),
            &transform_schema_object,
        );
        let instance_obj = db.new_object_from_prototype(
            ObjectName::new("object_b"),
            object_location,
            prototype_obj,
        );

        db.set_property_override(
            &schema_set,
            prototype_obj,
            "position.x",
            hydrate_model::Value::F64(10.0),
        );
        db.set_property_override(
            &schema_set,
            instance_obj,
            "position.x",
            hydrate_model::Value::F64(20.0),
        );

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

        edit_model.root_edit_context_mut().restore_objects_from(db);
        edit_model
    }

    fn try_load(schema_set: Arc<SchemaSet>, asset_data_path: &Path) -> Option<EditorModel> {
        let mut editor_model = EditorModel::new(schema_set);
        editor_model.add_file_system_object_source(asset_data_path);
        if editor_model.root_edit_context().all_objects().len() == 0 {
            None
        } else {
            Some(editor_model)
        }
    }

    pub fn load_or_init_empty(linker: SchemaLinker, asset_data_path: &Path, schema_def_path: &Path, schema_cache_file_path: &Path) -> Self {
        let schema_set = Arc::new(Self::load_schema(linker, schema_def_path, schema_cache_file_path));
        let editor_model = if let Some(loaded) = Self::try_load(schema_set.clone(), asset_data_path) {
            loaded
        } else {
            Self::init_empty_model(schema_set, asset_data_path)
        };

        DbState {
            editor_model,
            schema_cache_file_path: schema_cache_file_path.to_path_buf()
        }
    }

    pub fn save(&mut self) {
        log::debug!("saving schema cache to {:?}", self.schema_cache_file_path);
        let schema_cache = SchemaCacheSingleFile::store_string(self.editor_model.schema_set());
        std::fs::write(&self.schema_cache_file_path, schema_cache).unwrap();

        self.editor_model.save_root_edit_context();
    }
}
