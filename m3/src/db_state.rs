use nexdb::{DataSet, EditorModel, ObjectPath, SchemaCacheSingleFile, SchemaSet};
use std::path::PathBuf;
use std::sync::Arc;

pub struct DbState {
    //pub db: nexdb::Database,
    //pub undo_stack: nexdb::UndoStack,
    pub editor_model: EditorModel,
}

impl DbState {
    fn schema_def_path() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/schema"))
    }

    fn data_source_path() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/data_source"))
    }

    fn data_file_path() -> PathBuf {
        PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/data_source/data_file.nxt"
        ))
    }

    fn schema_cache_file_path() -> PathBuf {
        PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/data/schema_cache_file.json"
        ))
    }

    fn mount_path() -> ObjectPath {
        ObjectPath::root().join(&ObjectPath::new("data/"))
    }

    fn load_schema() -> SchemaSet {
        let mut schema_set = SchemaSet::default();

        let mut linker = nexdb::SchemaLinker::default();
        let path = Self::schema_def_path();
        linker.add_source_dir(&path, "*.json").unwrap();
        schema_set.add_linked_types(linker).unwrap();

        if let Some(schema_cache_str) = std::fs::read_to_string(Self::schema_cache_file_path()).ok()
        {
            SchemaCacheSingleFile::load_string(&mut schema_set, &schema_cache_str);
        }

        schema_set
    }

    fn init_empty_model() -> EditorModel {
        let schema_set = Arc::new(Self::load_schema());

        //let mut undo_stack = UndoStack::default();
        //let mut db = nexdb::Database::new(Arc::new(schema_set), &undo_stack);
        let mut db = DataSet::default();

        let mut edit_model = EditorModel::new(schema_set.clone());
        let objects_source_id =
            edit_model.open_file_system_source(Self::data_source_path(), Self::mount_path());
        let file_system = edit_model
            .file_system_data_source(objects_source_id)
            .unwrap();

        let transform_schema_object = schema_set
            .find_named_type("Transform")
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        //let data_path = Self::data_file_path()

        let prototype_obj = db.new_object(
            file_system
                .file_system_path_to_location(&Self::data_file_path())
                .unwrap(),
            &transform_schema_object,
        );
        let instance_obj = db.new_object_from_prototype(
            file_system
                .file_system_path_to_location(&Self::data_file_path())
                .unwrap(),
            prototype_obj,
        );

        db.set_property_override(
            &schema_set,
            prototype_obj,
            "position.x",
            nexdb::Value::F64(10.0),
        );
        db.set_property_override(
            &schema_set,
            instance_obj,
            "position.x",
            nexdb::Value::F64(20.0),
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

        edit_model.root_edit_context_mut().import_objects(db);
        edit_model
    }

    fn try_load() -> Option<EditorModel> {
        let schema_cache_str = std::fs::read_to_string(Self::schema_cache_file_path()).ok()?;

        let mut schema_set = SchemaSet::default();

        let mut linker = nexdb::SchemaLinker::default();
        let path = Self::schema_def_path();
        linker.add_source_dir(&path, "*.json").unwrap();
        schema_set.add_linked_types(linker).unwrap();

        SchemaCacheSingleFile::load_string(&mut schema_set, &schema_cache_str);

        let mut editor_model = EditorModel::new(Arc::new(schema_set));
        editor_model.open_file_system_source(Self::data_source_path(), Self::mount_path());
        if editor_model.root_edit_context().all_objects().len() == 0 {
            None
        } else {
            Some(editor_model)
        }
    }

    pub fn load_or_init_empty() -> Self {
        let editor_model = if let Some(loaded) = Self::try_load() {
            loaded
        } else {
            Self::init_empty_model()
        };

        Self { editor_model }
    }

    pub fn save(&mut self) {
        let schema_cache_file_path = Self::schema_cache_file_path();
        log::debug!("saving schema cache to {:?}", schema_cache_file_path);
        let schema_cache = SchemaCacheSingleFile::store_string(self.editor_model.schema_set());
        std::fs::write(schema_cache_file_path, schema_cache).unwrap();

        self.editor_model.save_root_edit_context();
    }
}
