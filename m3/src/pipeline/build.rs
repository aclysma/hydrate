use ::image::{EncodableLayout, GenericImageView};
use nexdb::{DataSet, DataSource, EditorModel, FileSystemObjectDataSource, HashMap, HashMapKeys, BuilderId, BuildInfo, ObjectId, ObjectLocation, ObjectName, ObjectSourceId, Schema, SchemaFingerprint, SchemaLinker, SchemaNamedType, SchemaRecord, SchemaSet, SingleObject, Value};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use imnodes::EditorContext;
use rafx::api::objc::runtime::Object;
use uuid::Uuid;
use type_uuid::{TypeUuid, TypeUuidDynamic};

use nexdb::dir_tree_blob_store::{path_to_uuid, uuid_to_path};
use nexdb::edit_context::EditContext;
use nexdb::json::SingleObjectJson;
use crate::pipeline::ImportJobs;


struct BuildOp {
    object_id: ObjectId,
    //builder_id: BuilderId,
    //path: PathBuf,
}

struct BuildJob {
    object_id: ObjectId,
    build_data_exists: bool,
    asset_exists: bool,
}

impl BuildJob {
    pub fn new(object_id: ObjectId) -> Self {
        BuildJob {
            object_id,
            build_data_exists: false,
            asset_exists: false
        }
    }
}

// Cache of all build jobs. This includes builds that are complete, in progress, or not started
pub struct BuildJobs {
    root_path: PathBuf,
    build_jobs: HashMap<ObjectId, BuildJob>,
    build_operations: Vec<BuildOp>
}

impl BuildJobs {
    pub fn new(
        builder_registry: &BuilderRegistry,
        editor_model: &EditorModel,
        root_path: PathBuf
    ) -> Self {
        let build_jobs = BuildJobs::find_all_jobs(builder_registry, editor_model, &root_path);

        BuildJobs {
            root_path,
            build_jobs,
            build_operations: Default::default()
        }
    }

    pub fn queue_build_operation(&mut self, object_id: ObjectId) {
        self.build_operations.push(BuildOp {
            object_id,
            //build_info
        })
    }

    pub fn update(&mut self, builder_registry: &BuilderRegistry, editor_model: &EditorModel, import_jobs: &ImportJobs) {
        let data_set = editor_model.root_edit_context().data_set();
        let schema_set = editor_model.schema_set();

        for build_op in &self.build_operations {
            let object_id = build_op.object_id;
            let object_type = editor_model.root_edit_context().object_schema(object_id).unwrap();
            let builder_id = builder_registry.builder_for_asset(object_type.fingerprint()).unwrap();
            let builder = builder_registry.builder(builder_id).unwrap();

            let dependencies = builder.dependencies(
                object_id,
                data_set,
                schema_set,
            );

            let mut imported_data = HashMap::default();
            for dependency_object_id in dependencies {
                let import_data = import_jobs.load_import_data(schema_set, dependency_object_id);

                // load it
                imported_data.insert(dependency_object_id, import_data);
            }

            let built_data = builder.build_asset(
                object_id,
                data_set,
                schema_set,
                &imported_data
            );

            let path = uuid_to_path(&self.root_path, build_op.object_id.as_uuid(), "af");

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }

            std::fs::write(&path, built_data).unwrap()
        }

        self.build_operations.clear();

        // Send/mark for processing?
    }

    fn find_all_jobs(builder_registry: &BuilderRegistry, editor_model: &EditorModel, root_path: &Path) -> HashMap<ObjectId, BuildJob> {
        let mut build_jobs = HashMap::<ObjectId, BuildJob>::default();

        //
        // Scan build dir for known build data
        //
        let walker = globwalk::GlobWalkerBuilder::from_patterns(    root_path, &["**.i"])
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file in walker {
            if let Ok(file) = file {
                println!("dir file {:?}", file);
                let dir_uuid = path_to_uuid(root_path, file.path()).unwrap();
                let object_id = ObjectId(dir_uuid.as_u128());
                let job = build_jobs.entry(object_id).or_insert_with(|| BuildJob::new(object_id));
                job.build_data_exists = true;
            }
        }

        //
        // Scan assets to find any asset that has an associated builder
        //
        let data_set = editor_model.root_edit_context().data_set();
        for object_id in data_set.all_objects() {
            // if let Some(build_info) = data_set.build_info(*object_id) {
            //     let builder_id = build_info.builder_id();
            //     let builder = builder_registry.builder(builder_id);
            //     if builder.is_some() {
            //         let job = build_jobs.entry(*object_id).or_insert_with(|| BuildJob::new(*object_id));
            //         job.asset_exists = true;
            //     }
            // }

            let schema_fingerprint = data_set.object_schema(*object_id).unwrap().fingerprint();
            let builder = builder_registry.builder_for_asset(schema_fingerprint);

            if builder.is_some() {
                let job = build_jobs.entry(*object_id).or_insert_with(|| BuildJob::new(*object_id));
                job.asset_exists = true;
            }
        }

        build_jobs

        // for (object_id, job) in build_jobs {
        //     if job.asset_exists && !job.build_data_exists {
        //         // We need to re-build the data
        //     }
        //
        //     if !job.asset_exists && job.build_data_exists {
        //         // We need to delete the build data that no longer has an associated asset
        //     }
        //
        //     if job.asset_exists && job.build_data_exists {
        //         // We may want to validate the build data and check that it is not stale
        //     }
        // }
    }
}











#[derive(Default)]
pub struct BuilderRegistry {
    registered_builders: HashMap<BuilderId, Box<Builder>>,
    //file_extension_associations: HashMap<String, Vec<BuilderId>>,
    asset_type_to_builder: HashMap<SchemaFingerprint, BuilderId>,
}

impl BuilderRegistry {
    //
    // Called before creating the schema to add handlers
    //
    pub fn register_handler<T: TypeUuid + Builder + Default + 'static>(&mut self, linker: &mut SchemaLinker) {
        let handler = Box::new(T::default());
        //handler.register_schemas(linker);
        let uuid = Uuid::from_bytes(T::UUID);
        let builder_id = BuilderId(uuid);
        self.registered_builders.insert(builder_id, handler);

        println!("Register builder {} {}", uuid, std::any::type_name::<T>());


        // for extension in self.registered_builders[&builder_id].asset_type() {
        //     ;
        // }
    }

    //
    // Called after finished linking the schema so we can associate schema fingerprints with handlers
    //
    pub fn finished_linking(&mut self, schema_set: &SchemaSet) {
        let mut asset_type_to_builder = HashMap::default();

        for (builder_id, builder) in &self.registered_builders {
            let asset_type = schema_set.find_named_type(builder.asset_type()).unwrap().fingerprint();
            let insert_result = asset_type_to_builder.insert(asset_type, *builder_id);
            println!("builder {} handles asset fingerprint {}", builder_id.0, asset_type.as_uuid());
            if insert_result.is_some() {
                panic!("Multiple handlers registered to handle the same asset")
            }
        }

        self.asset_type_to_builder = asset_type_to_builder;
    }

    // pub fn importers_for_file_extension(&self, extension: &str) -> &[BuilderId] {
    //     const EMPTY_LIST: &'static [BuilderId] = &[];
    //     self.file_extension_associations.get(extension).map(|x| x.as_slice()).unwrap_or(EMPTY_LIST)
    // }

    pub fn builder_for_asset(&self, fingerprint: SchemaFingerprint) -> Option<BuilderId> {
        self.asset_type_to_builder.get(&fingerprint).copied()
    }

    pub fn builder(&self, builder_id: BuilderId) -> Option<&Box<Builder>> {
        self.registered_builders.get(&builder_id)
    }
}

// ID?
pub trait Builder : TypeUuidDynamic  {
    fn builder_id(&self) -> BuilderId {
        BuilderId(Uuid::from_bytes(self.uuid()))
    }

    //fn register_schemas(&self, schema_linker: &mut SchemaLinker);

    fn asset_type(&self) -> &'static str;

    fn dependencies(&self, asset_id: ObjectId, data_set: &DataSet, schema: &SchemaSet) -> Vec<ObjectId>;

    fn build_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>
    ) -> Vec<u8>;
}
