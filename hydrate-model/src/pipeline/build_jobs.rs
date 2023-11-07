use crate::{EditorModel, HashMap, ObjectId};
use hydrate_base::{ArtifactId, ManifestFileEntryJson, ManifestFileJson};
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

use super::ImportJobs;

use hydrate_base::uuid_path::uuid_and_hash_to_path;

use super::*;

struct BuildRequest {
    object_id: ObjectId,
}

// A known build job, each existing asset will have an associated build job.
// It could be in a completed state, or there could be a problem with it and we need to re-run it.
struct BuildJob {
    build_data_exists: HashSet<(ArtifactId, u64)>,
    asset_exists: bool,
}

impl BuildJob {
    pub fn new() -> Self {
        BuildJob {
            build_data_exists: Default::default(),
            asset_exists: false,
        }
    }
}

// Cache of all build jobs. This includes builds that are complete, in progress, or not started.
// We find these by scanning existing assets. We also inspect the asset and built data to see if the
// job is complete, or is in a failed or stale state.
pub struct BuildJobs {
    build_data_root_path: PathBuf,
    job_executor: JobExecutor,
    build_jobs: HashMap<ObjectId, BuildJob>,
    //force_rebuild_operations: Vec<BuildOp>
}

impl BuildJobs {
    pub fn new(
        job_processor_registry: &JobProcessorRegistry,
        job_data_root_path: PathBuf,
        build_data_root_path: PathBuf,
    ) -> Self {
        //TODO: May need to scan disk to see what is cached?
        let job_executor = JobExecutor::new(job_data_root_path, job_processor_registry);
        let build_jobs = Default::default();

        BuildJobs {
            build_data_root_path,
            job_executor,
            build_jobs,
            //force_rebuild_operations: Default::default()
        }
    }

    pub fn queue_build_operation(
        &mut self,
        _object_id: ObjectId,
    ) {
        // self.build_operations.push(BuildOp {
        //     object_id,
        //     //force_rebuild_operations
        // })

        //TODO: Force it to appear as stale?
        unimplemented!();
    }

    pub fn update(
        &mut self,
        builder_registry: &BuilderRegistry,
        editor_model: &mut EditorModel,
        import_jobs: &ImportJobs,
        object_hashes: &HashMap<ObjectId, u64>,
        _import_data_metadata_hashes: &HashMap<ObjectId, u64>,
        combined_build_hash: u64,
    ) {
        editor_model.refresh_tree_node_cache();

        let data_set = editor_model.root_edit_context().data_set();
        let schema_set = editor_model.schema_set();

        //
        // Decide what assets we will initially request. This could be everything or just
        // a small set of assets (like a level, or all assets marked as "always export")
        //
        let mut requested_build_ops = VecDeque::default();
        for (&object_id, _) in object_hashes {
            assert!(!editor_model
                .is_path_node_or_root(data_set.object_schema(object_id).unwrap().fingerprint()));

            //TODO: Skip objects that aren't explicitly requested, if any were requested
            //      For now just build everything
            requested_build_ops.push_back(BuildRequest { object_id });
        }

        //
        // Main loop driving processing of jobs in dependency order. We may queue up additional
        // assets during this loop.
        //
        let mut started_build_ops = HashSet::<ObjectId>::default();
        let mut build_hashes = HashMap::default();
        let mut artifact_asset_lookup = HashMap::default();
        let mut built_artifact_metadata = HashMap::default();
        loop {
            //
            // If the job is finished, exit the loop
            //
            if requested_build_ops.is_empty() && self.job_executor.is_idle() {
                break;
            }

            //
            // For all the requested assets, see if there is a builder for the asset. If there is,
            // kick off the jobs needed to produce the asset for it
            //
            while let Some(request) = requested_build_ops.pop_front() {
                if started_build_ops.contains(&request.object_id) {
                    continue;
                }

                let object_id = request.object_id;
                started_build_ops.insert(object_id);

                println!("find {:?}", object_id);
                let object_type = editor_model
                    .root_edit_context()
                    .object_schema(object_id)
                    .unwrap();

                let Some(builder) = builder_registry.builder_for_asset(object_type.fingerprint())
                else {
                    continue;
                };

                println!("building {:?} {}", object_id, object_type.name());
                builder.start_jobs(object_id, data_set, schema_set, &self.job_executor);
            }

            //
            // Pump the job executor, this will schedule work to be done on threads
            //
            self.job_executor.update(data_set, schema_set, import_jobs);

            //
            // Jobs will produce artifacts. We will save these to disk and possibly trigger
            // additional jobs for assets that they reference.
            //
            let built_artifacts = self
                .job_executor
                .take_built_artifacts(&mut artifact_asset_lookup);
            for built_artifact in built_artifacts {
                //
                // Trigger building any dependencies
                //
                //TODO: I'm getting back handles to artifacts but I don't know what the associated asset
                // ID is
                for &dependency_artifact_id in &built_artifact.metadata.dependencies {
                    let dependency_object_id =
                        *artifact_asset_lookup.get(&dependency_artifact_id).unwrap();
                    requested_build_ops.push_back(BuildRequest {
                        object_id: dependency_object_id,
                    });
                }

                //
                // Serialize the artifacts to disk
                //
                let mut hasher = siphasher::sip::SipHasher::default();
                built_artifact.data.hash(&mut hasher);
                built_artifact.metadata.hash(&mut hasher);
                let build_hash = hasher.finish();

                let path = uuid_and_hash_to_path(
                    &self.build_data_root_path,
                    built_artifact.artifact_id.as_uuid(),
                    build_hash,
                    "bf",
                );

                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).unwrap();
                }

                let mut file = std::fs::File::create(&path).unwrap();
                built_artifact.metadata.write_header(&mut file).unwrap();
                file.write(&built_artifact.data).unwrap();

                //
                // Ensure the artifact will be in the metadata
                //
                build_hashes.insert(built_artifact.artifact_id, build_hash);

                let job = self
                    .build_jobs
                    .entry(built_artifact.asset_id)
                    .or_insert_with(|| BuildJob::new());
                job.asset_exists = true;
                job.build_data_exists
                    .insert((built_artifact.artifact_id, build_hash));

                built_artifact_metadata.insert(built_artifact.artifact_id, built_artifact.metadata);
            }
        }

        /*


                for build_op in &build_operations {
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
                        let import_data_hash = import_jobs.load_import_data_hash(dependency_object_id);

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


                    //std::fs::write(&path, built_data).unwrap()
                }
        */

        //
        // Write the manifest file
        //TODO: Only if it doesn't already exist? We could skip the whole building process in that case
        //
        let mut manifest_path = self.build_data_root_path.clone();
        manifest_path.push("manifests");
        std::fs::create_dir_all(&manifest_path).unwrap();
        manifest_path.push(format!("{:0>16x}.manifest", combined_build_hash));
        //let file = std::fs::File::create(manifest_path).unwrap();
        //let mut file = std::io::BufWriter::new(file);

        println!("built artifacts {:#?}", artifact_asset_lookup);

        let mut manifest_json = ManifestFileJson::default();

        for (artifact_id, build_hash) in build_hashes {
            println!("find asset for artifact {:?}", artifact_id);
            let asset_id = *artifact_asset_lookup.get(&artifact_id).unwrap();

            let artifact_metadata = built_artifact_metadata.get(&artifact_id).unwrap();

            let is_default_artifact = artifact_id.as_u128() == asset_id.0;
            let symbol_name = if is_default_artifact {
                if asset_id.as_uuid()
                    == Uuid::from_str("07ab9227-432d-49c8-8899-146acd803235").unwrap()
                {
                    println!("this one");
                }

                // editor_model.path_node_id_to_path(asset_id.get)
                // //let location = edit_context.object_location(object_id).unwrap();
                //TODO: Assert the cached asset path tree is not stale?
                let path = editor_model.object_display_name_long(asset_id);
                path

                //editor_model.object_symbol_name(ObjectId::from_uuid(artifact_id.as_uuid()))
            } else {
                String::default()

                //asset_id.as_uuid().to_string()
            };

            manifest_json.artifacts.push(ManifestFileEntryJson {
                artifact_id,
                build_hash: format!("{:0>16x}", build_hash),
                symbol_name,
                artifact_type: artifact_metadata.asset_type,
                //dependencies: artifact_metadata.dependencies.clone(),
            });

            //write!(file, "{:0>16x},{:0>16x},{},{},{:?}\n", artifact_id.as_u128(), build_hash, path, artifact_metadata.asset_type, artifact_metadata.dependencies).unwrap();
            //file.write(&object_id.0.to_le_bytes()).unwrap();
            //file.write(&build_hash.to_le_bytes()).unwrap();
        }

        std::fs::write(
            manifest_path,
            serde_json::to_string_pretty(&manifest_json).unwrap(),
        )
        .unwrap();

        //
        // Write a new TOC with summary of this build
        //
        let mut toc_path = self.build_data_root_path.clone();
        toc_path.push("toc");
        std::fs::create_dir_all(&toc_path).unwrap();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        toc_path.push(format!("{:0>16x}.toc", timestamp));

        std::fs::write(toc_path, format!("{:0>16x}", combined_build_hash)).unwrap();

        //std::fs::write(self.root_path.join("latest.txt"), format!("{:x}", combined_build_hash)).unwrap();
    }

    /*
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
    */
}
