use hydrate_model::pipeline::{
    HydrateProjectConfiguration, ImportJobSourceFile, ImportJobToQueue, ImportJobs,
    ImporterRegistry,
};
use hydrate_model::{
    EditorModel, PathNode, PathNodeRoot, SchemaCacheSingleFile, SchemaLinker, SchemaSet,
    SchemaSetBuilder,
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
        import_job_to_queue: &mut ImportJobToQueue,
    ) -> EditorModel {
        let mut editor_model = EditorModel::new(project_configuration.clone(), schema_set);
        for pair in &project_configuration.id_based_asset_sources {
            editor_model.add_file_system_id_based_asset_source(
                project_configuration,
                &pair.name,
                &pair.path,
                import_job_to_queue,
            );
        }
        for pair in &project_configuration.path_based_asset_sources {
            editor_model.add_file_system_path_based_data_source(
                project_configuration,
                &pair.name,
                &pair.path,
                importer_registry,
                import_job_to_queue,
            );
        }

        editor_model
    }

    #[profiling::function]
    pub fn load_schema(hydrate_project_configuration: &HydrateProjectConfiguration) -> SchemaSet {
        let mut linker = SchemaLinker::default();
        let mut schema_set = SchemaSetBuilder::default();

        PathNode::register_schema(&mut linker);
        PathNodeRoot::register_schema(&mut linker);
        for path in &hydrate_project_configuration.schema_def_paths {
            linker.add_source_dir(path, "**.json").unwrap();
        }
        schema_set.add_linked_types(linker).unwrap();

        schema_set.build()
    }

    #[profiling::function]
    pub fn load(
        schema_set: &SchemaSet,
        importer_registry: &ImporterRegistry,
        project_configuration: &HydrateProjectConfiguration,
        import_job_to_queue: &mut ImportJobToQueue,
    ) -> Self {
        let editor_model = Self::do_load(
            schema_set.clone(),
            importer_registry,
            &project_configuration,
            import_job_to_queue,
        );

        DbState {
            project_configuration: project_configuration.clone(),
            editor_model,
        }
    }

    pub fn save(&mut self) {
        self.editor_model.save_root_edit_context();
    }
}
