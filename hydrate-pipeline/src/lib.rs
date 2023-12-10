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

mod import_storage;
mod pipeline_error;

mod project;
pub use project::{HydrateProjectConfiguration, NamePathPair};

use crate::import_util::RequestedImportable;
pub use pipeline_error::*;

pub trait AssetPlugin {
    fn setup(
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    );
}

pub struct AssetPluginRegistry {
    importer_registry: ImporterRegistryBuilder,
    builder_registry: BuilderRegistryBuilder,
    job_processor_registry: JobProcessorRegistryBuilder,
}

impl AssetPluginRegistry {
    pub fn new() -> Self {
        AssetPluginRegistry {
            importer_registry: Default::default(),
            builder_registry: Default::default(),
            job_processor_registry: Default::default(),
        }
    }

    pub fn register_plugin<T: AssetPlugin>(
        mut self,
    ) -> Self {
        T::setup(
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

    fn handle_import_complete(
        &mut self,
        asset_id: AssetId,
        asset_name: AssetName,
        asset_location: AssetLocation,
        default_asset: &SingleObject,
        replace_with_default_asset: bool,
        import_info: ImportInfo,
        path_references: &HashMap<PathReference, AssetId>,
    ) -> DataSetResult<()>;

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
}

pub enum AssetEngineState {
    Idle,
    Importing(ImportStatusImporting),
    Building(BuildStatusBuilding),
}

pub struct AssetEngine {
    importer_registry: ImporterRegistry,
    import_jobs: ImportJobs,
    builder_registry: BuilderRegistry,
    build_jobs: BuildJobs,
}

impl AssetEngine {
    pub fn new(
        schema_set: &SchemaSet,
        importer_registry: ImporterRegistry,
        builder_registry: BuilderRegistry,
        job_processor_registry: JobProcessorRegistry,
        editor_model: &dyn DynEditorModel,
        project_configuration: &HydrateProjectConfiguration
    ) -> Self {
        let import_jobs = ImportJobs::new(
            &importer_registry,
            editor_model,
            &project_configuration.import_data_path,
        );

        let build_jobs = BuildJobs::new(
            schema_set,
            &job_processor_registry,
            project_configuration.import_data_path.clone(),
            project_configuration.job_data_path.clone(),
            project_configuration.build_data_path.clone(),
        );

        //TODO: Consider looking at disk to determine previous combined build hash so we don't for a rebuild every time we open

        AssetEngine {
            importer_registry,
            import_jobs,
            builder_registry,
            build_jobs,
        }
    }

    pub fn importer_registry(&self) -> &ImporterRegistry {
        &self.importer_registry
    }

    #[profiling::function]
    pub fn update(
        &mut self,
        editor_model: &mut dyn DynEditorModel,
    ) -> PipelineResult<AssetEngineState> {
        //
        // If user changes any asset data, cancel the in-flight build
        // If user initiates any import jobs, cancel the in-flight build
        // If file changes are detected on asset, import, or build data, cancel the in-flight build
        //

        //
        // If there are import jobs pending, cancel the in-flight build and execute them
        //
        let import_state = self.import_jobs
            .update(&self.importer_registry, editor_model)?;

        match import_state {
            ImportStatus::Idle => {
                // We can go to the next step
            },
            ImportStatus::Importing(importing_state) => {
                return Ok(AssetEngineState::Importing(importing_state))
            }
        }

        //
        // Process the in-flight build. It will be cancelled and restarted if any data is detected
        // as changing during the build.
        //

        // Check if our import state is consistent, if it is we save expected hashes and run builds
        let build_state = self.build_jobs.update(
            &self.builder_registry,
            editor_model,
            &self.import_jobs,
        )?;

        match build_state {
            BuildStatus::Idle => {
                Ok(AssetEngineState::Idle)
            }
            BuildStatus::Building(building_state) => {
                return Ok(AssetEngineState::Building(building_state))
            }
        }
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
        asset_ids: HashMap<ImportableName, RequestedImportable>,
        importer_id: ImporterId,
        path: PathBuf,
        import_type: ImportType,
    ) {
        self.import_jobs
            .queue_import_operation(asset_ids, importer_id, path, import_type);
    }

    pub fn queue_build_asset(
        &mut self,
        asset_id: AssetId,
    ) {
        self.build_jobs.queue_build_operation(asset_id);
    }

    pub fn needs_build(
        &self
    ) -> bool {
        self.build_jobs.needs_build()
    }

    pub fn queue_build_all(
        &mut self,
    ) {
        self.build_jobs.build();
    }
}
