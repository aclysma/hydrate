use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

pub use hydrate_schema::*;

pub use hydrate_data::*;

mod import_jobs;
pub use import_jobs::*;

mod import_types;
pub use import_types::*;

mod importer_registry;
pub use importer_registry::*;

pub mod job_system;
pub use job_system::*;

mod build_jobs;
pub use build_jobs::*;

mod build_types;
pub use build_types::*;

mod builder_registry;
pub use builder_registry::*;

mod import_thread_pool;

pub mod import_util;

mod pipeline_error;
mod import_storage;

pub use pipeline_error::*;

pub trait AssetPlugin {
    fn setup(
        schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    );
}

pub struct AssetPluginRegistrationHelper {
    importer_registry: ImporterRegistryBuilder,
    builder_registry: BuilderRegistryBuilder,
    job_processor_registry: JobProcessorRegistryBuilder,
}

impl AssetPluginRegistrationHelper {
    pub fn new() -> Self {
        AssetPluginRegistrationHelper {
            importer_registry: Default::default(),
            builder_registry: Default::default(),
            job_processor_registry: Default::default(),
        }
    }

    pub fn register_plugin<T: AssetPlugin>(
        mut self,
        schema_linker: &mut SchemaLinker,
    ) -> Self {
        T::setup(
            schema_linker,
            &mut self.importer_registry,
            &mut self.builder_registry,
            &mut self.job_processor_registry,
        );
        self
    }

    pub fn finish(
        self,
        schema_set: &SchemaSet,
    ) -> (ImporterRegistry, BuilderRegistry, JobProcessorRegistry) {
        let importer_registry = self.importer_registry.build();
        let builder_registry = self.builder_registry.build(schema_set);
        let job_processor_registry = self.job_processor_registry.build();

        (importer_registry, builder_registry, job_processor_registry)
    }
}

pub trait DynEditorModel {
    fn schema_set(&self) -> &SchemaSet;

    fn init_from_single_object(
        &mut self,
        asset_id: AssetId,
        single_object: &SingleObject,
    ) -> DataSetResult<()>;

    fn set_import_info(
        &mut self,
        asset_id: AssetId,
        import_info: ImportInfo,
    ) -> DataSetResult<()>;

    fn refresh_tree_node_cache(&mut self);

    fn data_set(&self) -> &DataSet;

    fn is_path_node_or_root(
        &self,
        schema_record: &SchemaRecord,
    ) -> bool;

    fn asset_display_name_long(
        &self,
        asset_id: AssetId,
    ) -> String;
}

pub trait DynEditContext {
    fn data_set(&self) -> &DataSet;

    fn schema_set(&self) -> &SchemaSet;

    fn new_asset(
        &mut self,
        asset_name: &AssetName,
        asset_location: &AssetLocation,
        schema: &SchemaRecord,
    ) -> AssetId;

    fn set_import_info(
        &mut self,
        asset_id: AssetId,
        import_info: ImportInfo,
    ) -> DataSetResult<()>;

    fn set_file_reference_override(
        &mut self,
        asset_id: AssetId,
        path: PathReference,
        referenced_asset_id: AssetId,
    ) -> DataSetResult<()>;
}

pub struct AssetEngine {
    importer_registry: ImporterRegistry,
    import_jobs: ImportJobs,
    builder_registry: BuilderRegistry,
    build_jobs: BuildJobs,

    previous_combined_build_hash: Option<u64>,
}

impl AssetEngine {
    pub fn new(
        schema_set: &SchemaSet,
        importer_registry: ImporterRegistry,
        builder_registry: BuilderRegistry,
        job_processor_registry: JobProcessorRegistry,
        editor_model: &dyn DynEditorModel,
        import_data_root_path: PathBuf,
        job_data_path: PathBuf,
        build_data_path: PathBuf,
    ) -> Self {
        let import_data_root_path = dunce::canonicalize(&import_data_root_path).unwrap();
        let job_data_path = dunce::canonicalize(&job_data_path).unwrap();
        let build_data_path = dunce::canonicalize(&build_data_path).unwrap();
        let import_jobs = ImportJobs::new(
            &importer_registry,
            editor_model,
            &import_data_root_path, /*DbState::import_data_source_path()*/
        );

        let build_jobs = BuildJobs::new(
            schema_set,
            &job_processor_registry,
            import_data_root_path,
            job_data_path,
            build_data_path,
        );

        //TODO: Consider looking at disk to determine previous combined build hash so we don't for a rebuild every time we open

        AssetEngine {
            importer_registry,
            import_jobs,
            builder_registry,
            build_jobs,
            previous_combined_build_hash: None,
        }
    }

    pub fn importer_registry(&self) -> &ImporterRegistry {
        &self.importer_registry
    }

    #[profiling::function]
    pub fn update(
        &mut self,
        editor_model: &mut dyn DynEditorModel,
    ) -> PipelineResult<()> {
        //
        // If user changes any asset data, cancel the in-flight build
        // If user initiates any import jobs, cancel the in-flight build
        // If file changes are detected on asset, import, or build data, cancel the in-flight build
        //

        //
        // If there are import jobs pending, cancel the in-flight build and execute them
        //
        self.import_jobs
            .update(&self.importer_registry, editor_model)?;

        //
        // If we don't have any pending import jobs, and we don't have a build in-flight, and
        // something has been changed since the last build, we can start a build now. We need to
        // first store the hashes of everything that will potentially go into the build.
        //
        let mut combined_build_hash = 0;
        let mut object_hashes = HashMap::default();
        for (asset_id, object) in editor_model.data_set().assets() {
            let hash = editor_model.data_set().hash_properties(*asset_id).unwrap();

            if !editor_model.is_path_node_or_root(object.schema()) {
                object_hashes.insert(*asset_id, hash);
            }

            let mut inner_hasher = siphasher::sip::SipHasher::default();
            asset_id.hash(&mut inner_hasher);
            hash.hash(&mut inner_hasher);
            combined_build_hash = combined_build_hash ^ inner_hasher.finish();
        }

        let import_data_metadata_hashes = self.import_jobs.clone_import_data_metadata_hashes();
        for (k, v) in &import_data_metadata_hashes {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            k.hash(&mut inner_hasher);
            v.hash(&mut inner_hasher);
            combined_build_hash = combined_build_hash ^ inner_hasher.finish();
        }

        let needs_rebuild =
            if let Some(previous_combined_build_hash) = self.previous_combined_build_hash {
                previous_combined_build_hash != combined_build_hash
            } else {
                true
            };

        if !needs_rebuild {
            return Ok(());
        }

        //
        // Process the in-flight build. It will be cancelled and restarted if any data is detected
        // as changing during the build.
        //

        // Check if our import state is consistent, if it is we save expected hashes and run builds
        self.build_jobs.update(
            &self.builder_registry,
            editor_model,
            &self.import_jobs,
            &object_hashes,
            &import_data_metadata_hashes,
            combined_build_hash,
        )?;
        self.previous_combined_build_hash = Some(combined_build_hash);
        Ok(())
    }

    pub fn importers_for_file_extension(
        &self,
        extension: &str,
    ) -> &[ImporterId] {
        self.importer_registry
            .importers_for_file_extension(extension)
    }

    pub fn importer(
        &self,
        importer_id: ImporterId,
    ) -> Option<&Arc<dyn Importer>> {
        self.importer_registry.importer(importer_id)
    }

    pub fn builder_for_asset(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> Option<&Box<dyn Builder>> {
        self.builder_registry.builder_for_asset(fingerprint)
    }

    // pub fn builder(&self, builder_id: BuilderId) -> Option<&Box<Builder>> {
    //     self.builder_registry.builder(builder_id)
    // }

    pub fn queue_import_operation(
        &mut self,
        asset_ids: HashMap<ImportableName, AssetId>,
        importer_id: ImporterId,
        path: PathBuf,
        assets_to_regenerate: HashSet<AssetId>,
        import_type: ImportType,
    ) {
        self.import_jobs
            .queue_import_operation(asset_ids, importer_id, path, assets_to_regenerate, import_type);
    }

    pub fn queue_build_operation(
        &mut self,
        asset_id: AssetId,
    ) {
        self.build_jobs.queue_build_operation(asset_id);
    }
}
