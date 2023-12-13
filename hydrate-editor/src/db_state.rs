use hydrate_model::pipeline::{HydrateProjectConfiguration, ImporterRegistry, ImportToQueue};
use hydrate_model::{
    EditorModel,
    PathNode, PathNodeRoot, SchemaCacheSingleFile, SchemaLinker, SchemaSet, SchemaSetBuilder,
};

pub struct DbState {
    pub project_configuration: HydrateProjectConfiguration,
    pub editor_model: EditorModel,
}

impl DbState {
    fn do_load(
        schema_set: SchemaSet,
        importer_registry: &ImporterRegistry,
        project_configuration: &HydrateProjectConfiguration,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) -> EditorModel {
        let mut editor_model = EditorModel::new(project_configuration.clone(), schema_set);
        for pair in &project_configuration.id_based_asset_sources {
            editor_model.add_file_system_id_based_asset_source(
                project_configuration,
                &pair.name,
                &pair.path,
                imports_to_queue,
            );
        }
        for pair in &project_configuration.path_based_asset_sources {
            editor_model.add_file_system_path_based_data_source(
                project_configuration,
                &pair.name,
                &pair.path,
                importer_registry,
                imports_to_queue,
            );
        }

        editor_model
    }

    #[profiling::function]
    pub fn load_schema(
        hydrate_project_configuration: &HydrateProjectConfiguration,
    ) -> SchemaSet {
        let mut linker = SchemaLinker::default();
        let mut schema_set = SchemaSetBuilder::default();

        PathNode::register_schema(&mut linker);
        PathNodeRoot::register_schema(&mut linker);
        for path in &hydrate_project_configuration.schema_def_paths {
            linker.add_source_dir(path, "**.json").unwrap();
        }
        schema_set.add_linked_types(linker).unwrap();

        if let Some(schema_cache_str) = std::fs::read_to_string(&hydrate_project_configuration.schema_cache_file_path).ok() {
            let named_types = SchemaCacheSingleFile::load_string(&schema_cache_str);
            schema_set.restore_named_types(named_types);
        }

        schema_set.build()
    }

    #[profiling::function]
    pub fn load(
        schema_set: &SchemaSet,
        importer_registry: &ImporterRegistry,
        project_configuration: &HydrateProjectConfiguration,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) -> Self {
        let editor_model = Self::do_load(
            schema_set.clone(),
            importer_registry,
            &project_configuration,
            imports_to_queue,
        );

        DbState {
            project_configuration: project_configuration.clone(),
            editor_model,
        }
    }

    pub fn save(&mut self) {
        log::debug!("saving schema cache to {:?}", self.project_configuration.schema_cache_file_path);
        let schema_cache =
            SchemaCacheSingleFile::store_string(self.editor_model.schema_set().schemas());
        std::fs::write(&self.project_configuration.schema_cache_file_path, schema_cache).unwrap();

        self.editor_model.save_root_edit_context();
    }
}
