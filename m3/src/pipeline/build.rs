use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write;
use ::image::{EncodableLayout, GenericImageView};
use nexdb::{DataSet, DataSource, EditorModel, FileSystemObjectDataSource, HashMap, HashMapKeys, BuilderId, BuildInfo, ObjectId, ObjectLocation, ObjectName, ObjectSourceId, Schema, SchemaFingerprint, SchemaLinker, SchemaNamedType, SchemaRecord, SchemaSet, SingleObject, Value};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use imnodes::EditorContext;
use rafx::api::objc::runtime::Object;
use uuid::Uuid;
use type_uuid::{TypeUuid, TypeUuidDynamic};

use nexdb::dir_tree_blob_store::{path_to_uuid, uuid_and_hash_to_path, uuid_to_path};
use nexdb::edit_context::EditContext;
use nexdb::json::SingleObjectJson;
use crate::pipeline::ImportJobs;


// An in-flight build operation we want to perform
struct BuildOp {
    object_id: ObjectId,
    //builder_id: BuilderId,
    //path: PathBuf,
}

// A known build job, each existing asset will have an associated build job.
// It could be in a completed state, or there could be a problem with it and we need to re-run it.
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

// Cache of all build jobs. This includes builds that are complete, in progress, or not started.
// We find these by scanning existing assets. We also inspect the asset and built data to see if the
// job is complete, or is in a failed or stale state.
pub struct BuildJobs {
    root_path: PathBuf,
    build_jobs: HashMap<ObjectId, BuildJob>,
    //force_rebuild_operations: Vec<BuildOp>
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
            //force_rebuild_operations: Default::default()
        }
    }

    pub fn queue_build_operation(&mut self, object_id: ObjectId) {
        // self.build_operations.push(BuildOp {
        //     object_id,
        //     //force_rebuild_operations
        // })

        //TODO: Force it to appear as stale?
    }

    pub fn update(
        &mut self,
        builder_registry: &BuilderRegistry,
        editor_model: &EditorModel,
        import_jobs: &ImportJobs,
        object_hashes: &HashMap<ObjectId, u64>,
        import_data_metadata_hashes: &HashMap<ObjectId, u64>,
        combined_build_hash: u64
    ) {
        let data_set = editor_model.root_edit_context().data_set();
        let schema_set = editor_model.schema_set();

        // let mut build_operations = Vec::default();
        // for (&object_id, build_job) in &editor_model.root_edit_context().objects() {
        //     //TODO: Check if it's stale? For now just assume we build everything
        //
        //     build_operations.push(BuildOp {
        //         object_id
        //     })
        // }

        let mut build_operations = Vec::default();
        for (&object_id, _) in object_hashes {
            build_operations.push(BuildOp {
                object_id
            });
        }

        let mut build_hashes = HashMap::default();

        for build_op in &build_operations {
            let object_id = build_op.object_id;
            let object_type = editor_model.root_edit_context().object_schema(object_id).unwrap();

            let builder = builder_registry.builder_for_asset(object_type.fingerprint());
            let builder = if builder.is_none() {
                log::warn!("can't find builder for object type {}", object_type.name());
                continue;
            } else {
                builder.unwrap()
            };

            log::info!("building object type {}", object_type.name());
            let dependencies = builder.dependencies(
                object_id,
                data_set,
                schema_set,
            );

            let mut imported_data = HashMap::default();
            let mut imported_data_hash = 0;

            for dependency_object_id in dependencies {
                // Not all objects have import info...
                let import_info = data_set.import_info(dependency_object_id);
                if import_info.is_none() {
                    continue;
                }

                // Load data from disk
                let import_data = import_jobs.load_import_data(schema_set, dependency_object_id);

                // Hash the dependency import data for the build
                let mut inner_hasher = siphasher::sip::SipHasher::default();
                dependency_object_id.hash(&mut inner_hasher);
                import_data.import_data.hash(&mut inner_hasher);
                imported_data_hash = imported_data_hash ^ inner_hasher.finish();

                // validate the file metadata hash is what we expected to see
                //import_data.metadata_hash

                // Place in map for the builder to use
                imported_data.insert(dependency_object_id, import_data.import_data);
            }

            let properties_hash = editor_model.root_edit_context().data_set().hash_properties(object_id).unwrap();

            let mut build_hasher = siphasher::sip::SipHasher::default();
            properties_hash.hash(&mut build_hasher);
            imported_data_hash.hash(&mut build_hasher);
            let build_hash = build_hasher.finish();

            // Check if a build for this hash already exists?
            let built_data = builder.build_asset(
                object_id,
                data_set,
                schema_set,
                &imported_data
            );

            //
            let path = uuid_and_hash_to_path(&self.root_path, build_op.object_id.as_uuid(), build_hash, "bf");
            build_hashes.insert(build_op.object_id, build_hash);

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }

            std::fs::write(&path, built_data).unwrap()
        }

        //
        // Write the manifest file
        //
        let mut manifest_path = self.root_path.clone();
        manifest_path.push("manifests");
        std::fs::create_dir_all(&manifest_path).unwrap();
        manifest_path.push(format!("{:x}.manifest", combined_build_hash));
        let file = File::create(manifest_path).unwrap();
        let mut file = std::io::BufWriter::new(file);
        for (object_id, build_hash) in build_hashes {
            write!(file, "{:x},{:x}\n", object_id.0, build_hash).unwrap();
            //file.write(&object_id.0.to_le_bytes()).unwrap();
            //file.write(&build_hash.to_le_bytes()).unwrap();
        }

        //std::fs::write(self.root_path.join("latest.txt"), format!("{:x}", combined_build_hash)).unwrap();



        //self.build_operations.clear();

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











// Keeps track of all known builders
#[derive(Default)]
pub struct BuilderRegistry {
    registered_builders: Vec<Box<Builder>>,
    //file_extension_associations: HashMap<String, Vec<BuilderId>>,
    asset_type_to_builder: HashMap<SchemaFingerprint, BuilderId>,
}

impl BuilderRegistry {
    //
    // Called before creating the schema to add handlers
    //
    pub fn register_handler<T: Builder + Default + 'static>(&mut self, linker: &mut SchemaLinker) {
        let handler = Box::new(T::default());
        self.registered_builders.push(handler);
    }

    //
    // Called after finished linking the schema so we can associate schema fingerprints with handlers
    //
    pub fn finished_linking(&mut self, schema_set: &SchemaSet) {
        let mut asset_type_to_builder = HashMap::default();

        for (builder_index, builder) in self.registered_builders.iter().enumerate() {
            let builder_id = BuilderId(builder_index);
            let asset_type = schema_set.find_named_type(builder.asset_type()).unwrap().fingerprint();
            let insert_result = asset_type_to_builder.insert(asset_type, builder_id);
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

    pub fn builder_for_asset(&self, fingerprint: SchemaFingerprint) -> Option<&Box<Builder>> {
        // if let Some(builder_id) = self.asset_type_to_builder.get(&fingerprint).copied() {
        //     Some(&self.registered_builders[builder_id.0])
        // } else {
        //     None
        // }
        self.asset_type_to_builder.get(&fingerprint).copied().map(|x| &self.registered_builders[x.0])
    }

    // pub fn builder(&self, builder_id: BuilderId) -> Option<&Box<Builder>> {
    //     self.registered_builders.get(&builder_id)
    // }
}

// Interface all builders must implement
pub trait Builder {
    // fn builder_id(&self) -> BuilderId {
    //     BuilderId(Uuid::from_bytes(self.uuid()))
    // }

    //fn register_schemas(&self, schema_linker: &mut SchemaLinker);

    // The type of asset that this builder handles
    fn asset_type(&self) -> &'static str;

    // Returns the assets that this build job needs to be available to complete
    fn dependencies(&self, asset_id: ObjectId, data_set: &DataSet, schema: &SchemaSet) -> Vec<ObjectId>;

    fn build_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
        dependency_data: &HashMap<ObjectId, SingleObject>
    ) -> Vec<u8>;
}
