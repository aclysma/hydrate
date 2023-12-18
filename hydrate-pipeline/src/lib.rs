use std::path::PathBuf;
use std::sync::Arc;

pub use hydrate_schema::*;

pub use hydrate_data::*;

mod pipeline_error;

mod project;
mod build;
mod import;
mod thumbnails;
pub use thumbnails::*;

pub use import::{
    ImportJobSourceFile, ScannedImportable, RequestedImportable, ImporterRegistry, Importer, ImporterRegistryBuilder,
    ImportJobs, ImportStatus, ImportStatusImporting, ImportType, ScanContext, import_util::create_asset_name, ImportContext,
    import_util::recursively_gather_import_operations_and_create_assets, ImportJobToQueue,
};

pub use project::{HydrateProjectConfiguration, NamePathPair};

pub use pipeline_error::*;
pub use crate::build::{
    Builder, BuilderRegistry, BuilderRegistryBuilder, BuildJobs, BuildStatus, BuildStatusBuilding, JobProcessorRegistry, JobProcessorRegistryBuilder,
    BuilderContext, JobInput, JobOutput, JobId, JobProcessor, RunContext, HandleFactory, EnumerateDependenciesContext, JobEnumeratedDependencies, AssetArtifactIdPair,
};

mod uuid_newtype;
mod log_events;
pub use log_events::*;

pub struct AssetPluginRegistries {
    pub importer_registry: ImporterRegistry,
    pub builder_registry: BuilderRegistry,
    pub job_processor_registry: JobProcessorRegistry,
    pub thumbnail_provider_registry: ThumbnailProviderRegistry,
}

pub struct AssetPluginSetupContext<'a> {
    pub importer_registry: &'a mut ImporterRegistryBuilder,
    pub builder_registry: &'a mut BuilderRegistryBuilder,
    pub job_processor_registry: &'a mut JobProcessorRegistryBuilder,
    pub thumbnail_provider_registry: &'a mut ThumbnailProviderRegistryBuilder,
}

pub trait AssetPlugin {
    fn setup(context: AssetPluginSetupContext);
}

pub struct AssetPluginRegistryBuilders {
    importer_registry: ImporterRegistryBuilder,
    builder_registry: BuilderRegistryBuilder,
    job_processor_registry: JobProcessorRegistryBuilder,
    thumbnail_provider_registry: ThumbnailProviderRegistryBuilder,
}

impl AssetPluginRegistryBuilders {
    pub fn new() -> Self {
        AssetPluginRegistryBuilders {
            importer_registry: Default::default(),
            builder_registry: Default::default(),
            job_processor_registry: Default::default(),
            thumbnail_provider_registry: Default::default(),
        }
    }

    pub fn register_plugin<T: AssetPlugin>(
        mut self,
    ) -> Self {
        T::setup(AssetPluginSetupContext {
            importer_registry: &mut self.importer_registry,
            builder_registry: &mut self.builder_registry,
            job_processor_registry: &mut self.job_processor_registry,
            thumbnail_provider_registry: &mut self.thumbnail_provider_registry,
        });
        self
    }

    pub fn finish(
        self,
        schema_set: &SchemaSet,
    ) -> AssetPluginRegistries {
        let importer_registry = self.importer_registry.build();
        let builder_registry = self.builder_registry.build(schema_set);
        let job_processor_registry = self.job_processor_registry.build();
        let thumbnail_provider_registry = self.thumbnail_provider_registry.build(schema_set);

        AssetPluginRegistries {
            importer_registry,
            builder_registry,
            job_processor_registry,
            thumbnail_provider_registry
        }
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
        canonical_path_references: &HashMap<CanonicalPathReference, AssetId>,
        path_references: &HashMap<PathReferenceHash, CanonicalPathReference>,
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
    ImportCompleted(Arc<ImportLogData>),
    BuildCompleted(Arc<BuildLogData>),
}

pub struct AssetEngine {
    importer_registry: ImporterRegistry,
    import_jobs: ImportJobs,
    builder_registry: BuilderRegistry,
    build_jobs: BuildJobs,
    thumbnail_system: ThumbnailSystem,
}

impl AssetEngine {
    pub fn new(
        schema_set: &SchemaSet,
        registries: AssetPluginRegistries,
        editor_model: &dyn DynEditorModel,
        project_configuration: &HydrateProjectConfiguration
    ) -> Self {
        let import_jobs = ImportJobs::new(
            project_configuration,
            &registries.importer_registry,
            editor_model,
            &project_configuration.import_data_path,
        );

        let build_jobs = BuildJobs::new(
            schema_set,
            &registries.job_processor_registry,
            project_configuration.import_data_path.clone(),
            project_configuration.job_data_path.clone(),
            project_configuration.build_data_path.clone(),
        );

        let thumbnail_system = ThumbnailSystem::new(
            project_configuration,
            registries.thumbnail_provider_registry,
            schema_set
        );

        //TODO: Consider looking at disk to determine previous combined build hash so we don't for a rebuild every time we open

        AssetEngine {
            importer_registry: registries.importer_registry,
            import_jobs,
            builder_registry: registries.builder_registry,
            build_jobs,
            thumbnail_system,
        }
    }

    pub fn current_task_log_data(&self) -> LogDataRef {
        if let Some(build_log) = self.build_jobs.current_build_log() {
            LogDataRef::Build(build_log)
        } else if let Some(import_log) = self.import_jobs.current_import_log() {
            LogDataRef::Import(import_log)
        } else {
            LogDataRef::None
        }
    }

    pub fn thumbnail_provider_registry(&self) -> &ThumbnailProviderRegistry {
        self.thumbnail_system.thumbnail_provider_registry()
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
        if !self.build_jobs.is_building() {
            let import_state = self.import_jobs
                .update(&self.importer_registry, editor_model)?;

            match import_state {
                ImportStatus::Idle => {
                    // We can go to the next step
                },
                ImportStatus::Importing(importing_state) => {
                    return Ok(AssetEngineState::Importing(importing_state))
                },
                ImportStatus::Completed(import_log_data) => {
                    return Ok(AssetEngineState::ImportCompleted(import_log_data))
                }
            }
        }

        if !self.build_jobs.is_building() {
            assert!(!self.import_jobs.is_importing());
            self.thumbnail_system.update(editor_model.data_set(), editor_model.schema_set());
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
            BuildStatus::Completed(build_log_data) => {
                return Ok(AssetEngineState::BuildCompleted(build_log_data))
            }
        }
    }

    pub fn thumbnail_system_state(
        &self
    ) -> &ThumbnailSystemState {
        self.thumbnail_system.system_state()
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
        import_job_to_queue: ImportJobToQueue,
        // asset_ids: HashMap<ImportableName, RequestedImportable>,
        // importer_id: ImporterId,
        // path: PathBuf,
        // import_type: ImportType,
    ) {
        self.import_jobs
            //.queue_import_operation(asset_ids, importer_id, path, import_type);
            .queue_import_operation(import_job_to_queue);
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
