mod build;

use std::path::PathBuf;
pub use build::*;

mod change_detector;

mod import;
pub use import::*;
use nexdb::{BuilderId, EditorModel, HashMap, ImporterId, ObjectId, SchemaFingerprint, SchemaLinker, SchemaSet};
use crate::db_state::DbState;

mod image;
pub use self::image::*;

mod blender_material;
pub use self::blender_material::*;

pub trait AssetPlugin {
    fn setup(schema_linker: &mut SchemaLinker, importer_registry: &mut ImporterRegistry, builder_registry: &mut BuilderRegistry);
}

pub struct AssetEngineBuilder {
    importer_registry: ImporterRegistry,
    builder_registry: BuilderRegistry,
}

impl AssetEngineBuilder {
    pub fn new() -> Self {
        AssetEngineBuilder {
            importer_registry: Default::default(),
            builder_registry: Default::default(),
        }
    }

    pub fn register_plugin<T: AssetPlugin>(mut self, schema_linker: &mut SchemaLinker) -> Self {
        T::setup(schema_linker, &mut self.importer_registry, &mut self.builder_registry);
        self
    }

    pub fn build(mut self, editor_model: &EditorModel) -> AssetEngine {
        self.importer_registry.finished_linking(editor_model.schema_set());
        self.builder_registry.finished_linking(editor_model.schema_set());
        let import_jobs = ImportJobs::new(&self.importer_registry, &editor_model, DbState::import_data_source_path());
        let build_jobs = BuildJobs::new(&self.builder_registry, &editor_model, DbState::build_data_source_path());
        AssetEngine {
            importer_registry: self.importer_registry,
            import_jobs,
            builder_registry: self.builder_registry,
            build_jobs
        }
    }
}

pub struct AssetEngine {
    importer_registry: ImporterRegistry,
    import_jobs: ImportJobs,
    builder_registry: BuilderRegistry,
    build_jobs: BuildJobs
}

impl AssetEngine {
    pub fn register_plugin<T: AssetPlugin>(&mut self, schema_linker: &mut SchemaLinker) {
        T::setup(schema_linker, &mut self.importer_registry, &mut self.builder_registry);
    }

    pub fn finish_linking(&mut self, schema_set: &SchemaSet) {
        self.importer_registry.finished_linking(schema_set);
        self.builder_registry.finished_linking(schema_set);
    }

    pub fn update(&mut self, editor_model: &EditorModel) {
        // First, run any user-initiated imports
        self.import_jobs.update(&self.importer_registry, editor_model);

        // If we detected a source-file change, we can queue them up as well

        // Store the hashes of known import data and assets and begin a build process
        // Fail the build if assets or import data changes are detected during the build, and restart





        // State machine
        // - Gather hash/timestamps/whatever for all the things (frequently)
        // - Run imports as needed
        //

        let mut object_hashes = HashMap::default();
        for (object_id, object) in editor_model.root_edit_context().objects() {
            let hash = editor_model.root_edit_context().data_set().hash_properties(*object_id).unwrap();
            object_hashes.insert(*object_id, hash);
        }

        //
        let import_data_metadata_hashes = self.import_jobs.clone_import_data_metadata_hashes();

        // Check if our import state is consistent, if it is we save expected hashes and run builds
        self.build_jobs.update(&self.builder_registry, editor_model, &self.import_jobs);
    }

    pub fn importers_for_file_extension(&self, extension: &str) -> &[ImporterId] {
        self.importer_registry.importers_for_file_extension(extension)
    }

    pub fn importer(&self, importer_id: ImporterId) -> Option<&Box<Importer>> {
        self.importer_registry.importer(importer_id)
    }

    pub fn builder_for_asset(&self, fingerprint: SchemaFingerprint) -> Option<BuilderId> {
        self.builder_registry.builder_for_asset(fingerprint)
    }

    pub fn builder(&self, builder_id: BuilderId) -> Option<&Box<Builder>> {
        self.builder_registry.builder(builder_id)
    }

    pub fn queue_import_operation(&mut self, object_ids: HashMap<Option<String>, ObjectId>, importer_id: ImporterId, path: PathBuf) {
        self.import_jobs.queue_import_operation(object_ids, importer_id, path);
    }

    pub fn queue_build_operation(&mut self, object_id: ObjectId) {
        self.build_jobs.queue_build_operation(object_id);
    }
}
