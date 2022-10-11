use std::path::PathBuf;
use nexdb::{DataStorageJsonSingleFile, Schema, SchemaCacheSingleFile, SchemaDefType};

pub struct DbState {
    pub db: nexdb::Database,
}

impl DbState {
    fn schema_def_path() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/schema"))
    }

    fn data_file_path() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/data_source/data_file.nxt"))
    }

    fn schema_cache_file_path() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/data/schema_cache_file.json"))
    }

    fn init_empty_database() -> Self {
        let mut linker = nexdb::SchemaLinker::default();
        let path = Self::schema_def_path();
        linker.add_source_dir(&path, "*.json").unwrap();

        let mut db = nexdb::Database::default();
        if let Some(schema_cache) = std::fs::read_to_string(Self::schema_cache_file_path()).ok() {
            SchemaCacheSingleFile::load_string(&mut db, &schema_cache);
        }

        db.add_linked_types(linker).unwrap();

        let transform_schema_object = db
            .find_named_type("Transform")
            .unwrap()
            .as_record()
            .unwrap()
            .clone();

        let prototype_obj = db.new_object(&transform_schema_object);
        let instance_obj = db.new_object_from_prototype(prototype_obj);

        db.set_property_override(prototype_obj, "position.x", nexdb::Value::F64(10.0));
        db.set_property_override(instance_obj, "position.x", nexdb::Value::F64(20.0));

        let prototype_array_element_1 =
            db.add_dynamic_array_override(prototype_obj, "all_fields.dynamic_array_i32");
        let prototype_array_element_2 =
            db.add_dynamic_array_override(prototype_obj, "all_fields.dynamic_array_i32");
        let instance_array_element_1 =
            db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_i32");
        let instance_array_element_2 =
            db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_i32");
        let instance_array_element_3 =
            db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_i32");

        let prototype_array_element_1 =
            db.add_dynamic_array_override(prototype_obj, "all_fields.dynamic_array_vec3");
        let prototype_array_element_2 =
            db.add_dynamic_array_override(prototype_obj, "all_fields.dynamic_array_vec3");
        let instance_array_element_1 =
            db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_vec3");
        let instance_array_element_2 =
            db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_vec3");
        let instance_array_element_3 =
            db.add_dynamic_array_override(instance_obj, "all_fields.dynamic_array_vec3");

        DbState {
            db,
        }
    }

    fn try_load() -> Option<Self> {
        let schema_cache_str = std::fs::read_to_string(Self::schema_cache_file_path()).ok()?;
        let data_str = std::fs::read_to_string(Self::data_file_path()).ok()?;

        let mut linker = nexdb::SchemaLinker::default();
        let path = Self::schema_def_path();
        linker.add_source_dir(&path, "*.json").unwrap();

        let mut db = nexdb::Database::default();
        SchemaCacheSingleFile::load_string(&mut db, &schema_cache_str);
        DataStorageJsonSingleFile::load_string(&mut db, &data_str);

        db.add_linked_types(linker).unwrap();

        Some(DbState {
            db
        })
    }

    pub fn load_or_init_empty() -> Self {
        if let Some(loaded) = Self::try_load() {
            loaded
        } else {
            Self::init_empty_database()
        }
    }

    pub fn save(&self) {
        let data_file_path = Self::data_file_path();
        log::debug!("saving data to {:?}", data_file_path);
        let data = DataStorageJsonSingleFile::store_string(&self.db);
        std::fs::write(data_file_path, data).unwrap();

        let schema_cache_file_path = Self::schema_cache_file_path();
        log::debug!("saving schema cache to {:?}", schema_cache_file_path);
        let schema_cache = SchemaCacheSingleFile::store_string(&self.db);
        std::fs::write(schema_cache_file_path, schema_cache).unwrap();
    }
}
