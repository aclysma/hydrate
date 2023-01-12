mod build;

use std::hash::{Hash, Hasher};
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
pub use blender_material::*;

mod glsl;
pub use glsl::*;

mod simple_data;
pub use simple_data::*;

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
            build_jobs,
            previous_combined_build_hash: None
        }
    }
}

pub struct AssetEngine {
    importer_registry: ImporterRegistry,
    import_jobs: ImportJobs,
    builder_registry: BuilderRegistry,
    build_jobs: BuildJobs,
    previous_combined_build_hash: Option<u64>,

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
        //
        // If user changes any asset data, cancel the in-flight build
        // If user initiates any import jobs, cancel the in-flight build
        // If file changes are detected on asset, import, or build data, cancel the in-flight build
        //

        //
        // If there are import jobs pending, cancel the in-flight build and execute them
        //
        self.import_jobs.update(&self.importer_registry, editor_model);

        //
        // If we don't have any pending import jobs, and we don't have a build in-flight, and
        // something has been changed since the last build, we can start a build now. We need to
        // first store the hashes of everything that will potentially go into the build.
        //
        let mut combined_build_hash = 0;
        let mut object_hashes = HashMap::default();
        for (object_id, object) in editor_model.root_edit_context().objects() {
            let hash = editor_model.root_edit_context().data_set().hash_properties(*object_id).unwrap();
            object_hashes.insert(*object_id, hash);

            let mut inner_hasher = siphasher::sip::SipHasher::default();
            object_id.hash(&mut inner_hasher);
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

        let needs_rebuild = if let Some(previous_combined_build_hash) = self.previous_combined_build_hash {
            previous_combined_build_hash != combined_build_hash
        } else {
            true
        };

        if !needs_rebuild {
            return;
        }

        //
        // Process the in-flight build. It will be cancelled and restarted if any data is detected
        // as changing during the build.
        //

        // Check if our import state is consistent, if it is we save expected hashes and run builds
        self.build_jobs.update(&self.builder_registry, editor_model, &self.import_jobs, &object_hashes, &import_data_metadata_hashes, combined_build_hash);
        self.previous_combined_build_hash = Some(combined_build_hash);
    }

    pub fn importers_for_file_extension(&self, extension: &str) -> &[ImporterId] {
        self.importer_registry.importers_for_file_extension(extension)
    }

    pub fn importer(&self, importer_id: ImporterId) -> Option<&Box<Importer>> {
        self.importer_registry.importer(importer_id)
    }

    pub fn builder_for_asset(&self, fingerprint: SchemaFingerprint) -> Option<&Box<Builder>> {
        self.builder_registry.builder_for_asset(fingerprint)
    }

    // pub fn builder(&self, builder_id: BuilderId) -> Option<&Box<Builder>> {
    //     self.builder_registry.builder(builder_id)
    // }

    pub fn queue_import_operation(&mut self, object_ids: HashMap<Option<String>, ObjectId>, importer_id: ImporterId, path: PathBuf) {
        self.import_jobs.queue_import_operation(object_ids, importer_id, path);
    }

    pub fn queue_build_operation(&mut self, object_id: ObjectId) {
        self.build_jobs.queue_build_operation(object_id);
    }
}
