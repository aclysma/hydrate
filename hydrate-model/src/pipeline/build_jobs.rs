use crate::{BuildInfo, BuilderId, DataSet, DataSource, EditorModel, HashMap, HashMapKeys, ObjectId, ObjectLocation, ObjectName, ObjectSourceId, Schema, SchemaFingerprint, SchemaLinker, SchemaNamedType, SchemaRecord, SchemaSet, SingleObject, Value};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Write};
use std::path::{Path, PathBuf};
use hydrate_base::{BuiltObjectMetadata, AssetUuid};
use hydrate_base::handle::DummySerdeContextHandle;

use super::ImportJobs;

use hydrate_base::uuid_path::{path_to_uuid, path_to_uuid_and_hash, uuid_and_hash_to_path, uuid_to_path};

use super::*;

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
    build_data_exists: HashSet<u64>,
    asset_exists: bool,
}

impl BuildJob {
    pub fn new(object_id: ObjectId) -> Self {
        BuildJob {
            object_id,
            build_data_exists: Default::default(),
            asset_exists: false,
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
        root_path: PathBuf,
    ) -> Self {
        let build_jobs = BuildJobs::find_all_jobs(builder_registry, editor_model, &root_path);

        BuildJobs {
            root_path,
            build_jobs,
            //force_rebuild_operations: Default::default()
        }
    }

    pub fn queue_build_operation(
        &mut self,
        object_id: ObjectId,
    ) {
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
        combined_build_hash: u64,
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
            build_operations.push(BuildOp { object_id });
        }

        let mut build_hashes = HashMap::default();

        for build_op in &build_operations {
            let object_id = build_op.object_id;
            let object_type = editor_model
                .root_edit_context()
                .object_schema(object_id)
                .unwrap();

            let builder = builder_registry.builder_for_asset(object_type.fingerprint());
            let builder = if builder.is_none() {
                log::trace!("can't find builder for object type {}", object_type.name());
                continue;
            } else {
                builder.unwrap()
            };

            //log::info!("building object type {}", object_type.name());
            let dependencies = builder.enumerate_dependencies(object_id, data_set, schema_set);

            let mut imported_data = HashMap::default();
            let mut imported_data_hash = 0;

            //
            // Just load in the import data hashes
            //
            for &dependency_object_id in &dependencies {
                // Not all objects have import info...
                let import_info = data_set.import_info(dependency_object_id);
                if import_info.is_none() {
                    continue;
                }

                // Load data from disk
                let import_data_hash = import_jobs.load_import_data_hash(schema_set, dependency_object_id);

                // Hash the dependency import data for the build
                let mut inner_hasher = siphasher::sip::SipHasher::default();
                dependency_object_id.hash(&mut inner_hasher);
                import_data_hash.metadata_hash.hash(&mut inner_hasher);
                //TODO: We could also hash the raw bytes of the file
                imported_data_hash = imported_data_hash ^ inner_hasher.finish();
            }

            let properties_hash = editor_model
                .root_edit_context()
                .data_set()
                .hash_properties(object_id)
                .unwrap();

            let mut build_hasher = siphasher::sip::SipHasher::default();
            properties_hash.hash(&mut build_hasher);
            //TODO: This doesn't handle looking at objects referenced by this object
            imported_data_hash.hash(&mut build_hasher);
            let build_hash = build_hasher.finish();

            //let dummy_serde_context = DummySerdeContext::new();

            //dummy_serde_context.

            let mut can_use_cached_build_data = false;
            if let Some(build_job) = self.build_jobs.get(&object_id) {
                if build_job.build_data_exists.contains(&build_hash) {
                    can_use_cached_build_data = true;
                }
            }

            // Include this data in the manifest
            build_hashes.insert(build_op.object_id, build_hash);

            if can_use_cached_build_data {
                //println!("  using cached build data {:?}", data_set.object_name(object_id));
            } else {
                println!("  rebuilding asset {:?}", data_set.object_name(object_id));
                //
                // Go get the actual import data
                //
                for dependency_object_id in dependencies {
                    // Not all objects have import info...
                    let import_info = data_set.import_info(dependency_object_id);
                    if import_info.is_none() {
                        continue;
                    }

                    // Load data from disk
                    let import_data = import_jobs.load_import_data(schema_set, dependency_object_id);

                    // Place in map for the builder to use
                    imported_data.insert(dependency_object_id, import_data.import_data);
                }

                //TODO: This might be able to be replaced with using the schema info? But that doesn't work with built data
                // which is arbitrary binary
                let mut ctx = DummySerdeContextHandle::default();
                ctx.begin_serialize_asset(AssetUuid(*object_id.as_uuid().as_bytes()));
                let mut built_data = ctx.scope(|| {
                    builder.build_asset(object_id, data_set, schema_set, &imported_data)
                });

                let referenced_assets = ctx.end_serialize_asset(AssetUuid(*object_id.as_uuid().as_bytes()));
                assert!(built_data.metadata.dependencies.is_empty()); //TODO: Pretty big bug here, we overwrite dependencies that the builder returns
                built_data.metadata.dependencies = referenced_assets.into_iter().map(|x| ObjectId(uuid::Uuid::from_bytes(x.0.0).as_u128())).collect();

                //
                let path = uuid_and_hash_to_path(
                    &self.root_path,
                    build_op.object_id.as_uuid(),
                    build_hash,
                    "bf",
                );

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }

                // let metadata = hydrate_model::BuiltObjectMetadata {
                //     asset_type: Uuid::default(),
                //     subresource_count: 0,
                //     dependencies: vec![],
                // };

                let mut file = std::fs::File::create(&path).unwrap();
                built_data.metadata.write_header(&mut file).unwrap();
                file.write(&built_data.data).unwrap();

                let job = self.build_jobs
                    .entry(object_id)
                    .or_insert_with(|| BuildJob::new(object_id));
                job.asset_exists = true;
                job.build_data_exists.insert(build_hash);
            }

            //std::fs::write(&path, built_data).unwrap()
        }

        //
        // Write the manifest file
        //TODO: Only if it doesn't already exist? We could skip the whole building process in that case
        //
        let mut manifest_path = self.root_path.clone();
        manifest_path.push("manifests");
        std::fs::create_dir_all(&manifest_path).unwrap();
        manifest_path.push(format!("{:0>16x}.manifest", combined_build_hash));
        let file = File::create(manifest_path).unwrap();
        let mut file = std::io::BufWriter::new(file);
        for (object_id, build_hash) in build_hashes {
            write!(file, "{:0>16x},{:0>16x}\n", object_id.0, build_hash).unwrap();
            //file.write(&object_id.0.to_le_bytes()).unwrap();
            //file.write(&build_hash.to_le_bytes()).unwrap();
        }

        //
        // Write a new TOC with summary of this build
        //
        let mut toc_path = self.root_path.clone();
        toc_path.push("toc");
        std::fs::create_dir_all(&toc_path).unwrap();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        toc_path.push(format!("{:0>16x}.toc", timestamp));

        std::fs::write(toc_path, format!("{:0>16x}", combined_build_hash)).unwrap();

        //std::fs::write(self.root_path.join("latest.txt"), format!("{:x}", combined_build_hash)).unwrap();

        //self.build_operations.clear();

        // Send/mark for processing?
    }

    fn find_all_jobs(
        builder_registry: &BuilderRegistry,
        editor_model: &EditorModel,
        root_path: &Path,
    ) -> HashMap<ObjectId, BuildJob> {
        let mut build_jobs = HashMap::<ObjectId, BuildJob>::default();

        //
        // Scan build dir for known build data
        //
        let walker = globwalk::GlobWalkerBuilder::from_patterns(root_path, &["**.bf"])
            .file_type(globwalk::FileType::FILE)
            .build()
            .unwrap();

        for file in walker {
            if let Ok(file) = file {
                //println!("built file {:?}", file);
                let (built_file_uuid, built_file_hash) = path_to_uuid_and_hash(root_path, file.path()).unwrap();
                let object_id = ObjectId(built_file_uuid.as_u128());
                let job = build_jobs
                    .entry(object_id)
                    .or_insert_with(|| BuildJob::new(object_id));
                job.build_data_exists.insert(built_file_hash);
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
                let job = build_jobs
                    .entry(*object_id)
                    .or_insert_with(|| BuildJob::new(*object_id));
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
