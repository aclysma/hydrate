use crate::{EditorModel, HashMap, AssetId};
use hydrate_base::{ArtifactId, BuiltArtifactMetadata, DebugArtifactManifestDataJson, DebugManifestFileJson, StringHash};
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;

use super::ImportJobs;

use hydrate_base::uuid_path::uuid_and_hash_to_path;

use super::*;

struct BuildRequest {
    asset_id: AssetId,
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
    build_jobs: HashMap<AssetId, BuildJob>,
    //force_rebuild_operations: Vec<BuildOp>
}

impl BuildJobs {
    pub fn new(
        schema_set: &SchemaSet,
        job_processor_registry: &JobProcessorRegistry,
        import_data_root_path: PathBuf,
        job_data_root_path: PathBuf,
        build_data_root_path: PathBuf,
    ) -> Self {
        //TODO: May need to scan disk to see what is cached?
        let job_executor = JobExecutor::new(schema_set, job_processor_registry, import_data_root_path, job_data_root_path, build_data_root_path.clone());
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
        _asset_id: AssetId,
    ) {
        // self.build_operations.push(BuildOp {
        //     asset_id,
        //     //force_rebuild_operations
        // })

        //TODO: Force it to appear as stale?
        unimplemented!();
    }

    #[profiling::function]
    pub fn update(
        &mut self,
        builder_registry: &BuilderRegistry,
        editor_model: &mut EditorModel,
        import_jobs: &ImportJobs,
        asset_hashes: &HashMap<AssetId, u64>,
        _import_data_metadata_hashes: &HashMap<AssetId, u64>,
        combined_build_hash: u64,
    ) {
        profiling::scope!("Process Build Operations");
        editor_model.refresh_tree_node_cache();

        let data_set = {
            profiling::scope!("Clone Dataset");
            Arc::new(editor_model.root_edit_context().data_set().clone())
        };
        let schema_set = editor_model.schema_set();

        //
        // Decide what assets we will initially request. This could be everything or just
        // a small set of assets (like a level, or all assets marked as "always export")
        //
        let mut requested_build_ops = VecDeque::default();
        for (&asset_id, _) in asset_hashes {
            assert!(!editor_model
                .is_path_node_or_root(data_set.asset_schema(asset_id).unwrap().fingerprint()));

            //TODO: Skip assets that aren't explicitly requested, if any were requested
            //      For now just build everything
            requested_build_ops.push_back(BuildRequest { asset_id });
        }

        //
        // Main loop driving processing of jobs in dependency order. We may queue up additional
        // assets during this loop.
        //
        let mut started_build_ops = HashSet::<AssetId>::default();
        let mut build_hashes = HashMap::default();
        let mut artifact_asset_lookup = HashMap::default();

        struct BuiltArtifactInfo {
            asset_id: AssetId,
            artifact_key_debug_name: Option<String>,
            metadata: BuiltArtifactMetadata,
        }

        let mut built_artifact_info = HashMap::default();
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
            {
                //profiling::scope!("Start Jobs");
                while let Some(request) = requested_build_ops.pop_front() {
                    if started_build_ops.contains(&request.asset_id) {
                        continue;
                    }

                    let asset_id = request.asset_id;
                    started_build_ops.insert(asset_id);

                    let asset_type = editor_model
                        .root_edit_context()
                        .asset_schema(asset_id)
                        .unwrap();

                    let Some(builder) = builder_registry.builder_for_asset(asset_type.fingerprint())
                        else {
                            continue;
                        };

                    builder.start_jobs(asset_id, &data_set, schema_set, self.job_executor.job_api());
                }
            }

            //
            // Pump the job executor, this will schedule work to be done on threads
            //
            {
                //profiling::scope!("Job Executor Update");
                self.job_executor.update(&data_set, schema_set, import_jobs);
            }

            {
                //profiling::scope!("Take written artifacts");

                //
                // Jobs will produce artifacts. We will save these to disk and possibly trigger
                // additional jobs for assets that they reference.
                //
                let written_artifacts = self
                    .job_executor
                    .take_written_artifacts(&mut artifact_asset_lookup);

                for written_artifact in written_artifacts {
                    //
                    // Trigger building any dependencies.
                    //
                    for &dependency_artifact_id in &written_artifact.metadata.dependencies {
                        let dependency_asset_id =
                            *artifact_asset_lookup.get(&dependency_artifact_id).unwrap();
                        requested_build_ops.push_back(BuildRequest {
                            asset_id: dependency_asset_id,
                        });
                    }

                    //
                    // Ensure the artifact will be in the metadata
                    //
                    build_hashes.insert(written_artifact.artifact_id, written_artifact.build_hash);

                    let job = self
                        .build_jobs
                        .entry(written_artifact.asset_id)
                        .or_insert_with(|| BuildJob::new());
                    job.asset_exists = true;
                    job.build_data_exists
                        .insert((written_artifact.artifact_id, written_artifact.build_hash));

                    built_artifact_info.insert(written_artifact.artifact_id, BuiltArtifactInfo {
                        asset_id: written_artifact.asset_id,
                        artifact_key_debug_name: written_artifact.artifact_key_debug_name,
                        metadata: written_artifact.metadata,
                    });
                }
            }
        }

        /*


                for build_op in &build_operations {
                    //log::info!("building asset type {}", asset_type.name());
                    let dependencies = builder.enumerate_dependencies(asset_id, data_set, schema_set);

                    let mut imported_data = HashMap::default();
                    let mut imported_data_hash = 0;

                    //
                    // Just load in the import data hashes
                    //
                    for &dependency_asset_id in &dependencies {
                        // Not all assets have import info...
                        let import_info = data_set.import_info(dependency_asset_id);
                        if import_info.is_none() {
                            continue;
                        }

                        // Load data from disk
                        let import_data_hash = import_jobs.load_import_data_hash(dependency_asset_id);

                        // Hash the dependency import data for the build
                        let mut inner_hasher = siphasher::sip::SipHasher::default();
                        dependency_asset_id.hash(&mut inner_hasher);
                        import_data_hash.metadata_hash.hash(&mut inner_hasher);
                        //TODO: We could also hash the raw bytes of the file
                        imported_data_hash = imported_data_hash ^ inner_hasher.finish();
                    }

                    let properties_hash = editor_model
                        .root_edit_context()
                        .data_set()
                        .hash_properties(asset_id)
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

        // This is a more compact file that is run at release
        let manifest_path_release = manifest_path.join(format!("{:0>16x}.manifest_release", combined_build_hash));
        let manifest_release_file = std::fs::File::create(manifest_path_release).unwrap();
        let mut manifest_release_file_writer = std::io::BufWriter::new(manifest_release_file);

        // This is a json file that supplements the release manifest
        let manifest_path_debug = manifest_path.join(format!("{:0>16x}.manifest_debug", combined_build_hash));

        let mut manifest_json = DebugManifestFileJson::default();

        let mut all_hashes = HashSet::default();
        for (artifact_id, build_hash) in build_hashes {
            let built_artifact_info = built_artifact_info.get(&artifact_id).unwrap();
            let asset_id = built_artifact_info.asset_id;

            let is_default_artifact = artifact_id.as_uuid() == asset_id.as_uuid();
            let symbol_name = if is_default_artifact {
                // editor_model.path_node_id_to_path(asset_id.get)
                // //let location = edit_context.asset_location(asset_id).unwrap();
                //TODO: Assert the cached asset path tree is not stale?
                let path = editor_model.asset_display_name_long(asset_id);
                assert!(!path.is_empty());
                Some(path)
            } else {
                None
            };

            let symbol_name_hash = StringHash::from_runtime_str(&symbol_name.clone().unwrap_or_default()).hash();
            if symbol_name_hash != 0 {
                let newly_inserted = all_hashes.insert(symbol_name_hash);
                // We have a hash collision if this assert fires
                assert!(newly_inserted);
            }

            let debug_name = if let Some(artifact_key_debug_name) = &built_artifact_info.artifact_key_debug_name {
                format!("{}#{}", editor_model.asset_display_name_long(asset_id), artifact_key_debug_name)
            } else {
                editor_model.asset_display_name_long(asset_id)
            };

            manifest_json.artifacts.push(DebugArtifactManifestDataJson {
                artifact_id,
                build_hash: format!("{:0>16x}", build_hash),
                symbol_hash: format!("{:0>32x}", symbol_name_hash),
                symbol_name: symbol_name.unwrap_or_default(),
                artifact_type: built_artifact_info.metadata.asset_type,
                debug_name,
                //dependencies: artifact_metadata.dependencies.clone(),
            });

            // Write the artifact ID, build hash, asset type, and hash of symbol name in CSV (this could be very compact binary one day
            write!(manifest_release_file_writer, "{:0>32x},{:0>16x},{:0>32x},{:0>32x}\n", artifact_id.as_u128(), build_hash, built_artifact_info.metadata.asset_type.as_u128(), symbol_name_hash).unwrap();
        }

        drop(manifest_release_file_writer);

        {
            profiling::scope!("Write debug manifest data");
            let json = {
                profiling::scope!("serde_json::to_string_pretty");
                serde_json::to_string_pretty(&manifest_json).unwrap()
            };

            profiling::scope!("std::fs::write");
            std::fs::write(
                manifest_path_debug,
                json,
            ).unwrap();
        }

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
    ) -> HashMap<AssetId, BuildJob> {
        let mut build_jobs = HashMap::<AssetId, BuildJob>::default();

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
                let asset_id = AssetId(built_file_uuid.as_u128());
                let job = build_jobs
                    .entry(asset_id)
                    .or_insert_with(|| BuildJob::new(asset_id));
                job.build_data_exists.insert(built_file_hash);
            }
        }

        //
        // Scan assets to find any asset that has an associated builder
        //
        let data_set = editor_model.root_edit_context().data_set();
        for asset_id in data_set.all_assets() {
            // if let Some(build_info) = data_set.build_info(*asset_id) {
            //     let builder_id = build_info.builder_id();
            //     let builder = builder_registry.builder(builder_id);
            //     if builder.is_some() {
            //         let job = build_jobs.entry(*asset_id).or_insert_with(|| BuildJob::new(*asset_id));
            //         job.asset_exists = true;
            //     }
            // }

            let schema_fingerprint = data_set.asset_schema(*asset_id).unwrap().fingerprint();
            let builder = builder_registry.builder_for_asset(schema_fingerprint);

            if builder.is_some() {
                let job = build_jobs
                    .entry(*asset_id)
                    .or_insert_with(|| BuildJob::new(*asset_id));
                job.asset_exists = true;
            }
        }

        build_jobs

        // for (asset_id, job) in build_jobs {
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
