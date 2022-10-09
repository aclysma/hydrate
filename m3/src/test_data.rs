use std::path::PathBuf;
use nexdb::{DataStorageJsonSingleFile, Schema, SchemaCacheSingleFile, SchemaDefType};

pub struct TestData {
    pub db: nexdb::Database,
    pub prototype_obj: nexdb::ObjectId,
    pub instance_obj: nexdb::ObjectId,
}

impl TestData {
    fn create() -> Self {
        let mut linker = nexdb::SchemaLinker::default();
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src/schema"));
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

        TestData {
            db,
            prototype_obj,
            instance_obj,
        }
    }

    fn data_file_path() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/out/data_file_out.json"))
    }

    fn schema_cache_file_path() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/out/schema_cache_file_out.json"))
    }

    fn try_load() -> Option<Self> {
        let schema_cache_str = std::fs::read_to_string(Self::schema_cache_file_path()).ok()?;
        let data_str = std::fs::read_to_string(Self::data_file_path()).ok()?;

        let mut linker = nexdb::SchemaLinker::default();
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src/schema"));
        linker.add_source_dir(&path, "*.json").unwrap();

        let mut db = nexdb::Database::default();
        SchemaCacheSingleFile::load_string(&mut db, &schema_cache_str);
        DataStorageJsonSingleFile::load_string(&mut db, &data_str);

        db.add_linked_types(linker).unwrap();

        let mut prototype_and_instance = None;
        for object in db.all_objects() {
            let prototype = db.object_prototype(*object);
            if let Some(prototype) = prototype {
                prototype_and_instance = Some((prototype, *object));
                break;
            }
        }

        if let Some((prototype_obj, instance_obj)) = prototype_and_instance {
            Some(TestData {
                db,
                prototype_obj,
                instance_obj,
            })
        } else {
            None
        }
    }

    pub fn load_or_create() -> Self {
        if let Some(loaded) = Self::try_load() {
            loaded
        } else {
            Self::create()
        }
    }

    fn save(&self) {
        let data_file_path = Self::data_file_path();
        log::debug!("write data to {:?}", data_file_path);
        let data = DataStorageJsonSingleFile::store_string(&self.db);
        std::fs::write(data_file_path, data).unwrap();

        let schema_cache_file_path = Self::schema_cache_file_path();
        log::debug!("write schema cache to {:?}", schema_cache_file_path);
        let schema_cache = SchemaCacheSingleFile::store_string(&self.db);
        std::fs::write(schema_cache_file_path, schema_cache).unwrap();
    }
}
