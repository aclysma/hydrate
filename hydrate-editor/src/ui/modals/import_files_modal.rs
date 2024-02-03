use crate::modal_action::{
    default_modal_window, ModalAction, ModalActionControlFlow, ModalContext,
};
use crate::ui::components::draw_location_selector;
use hydrate_model::pipeline::{ImportJobToQueue, ImportType, ImporterRegistry};
use hydrate_model::{AssetLocation, HashSet};
use std::path::PathBuf;

pub struct ImportFilesModal {
    files_to_import: HashSet<PathBuf>,
    selected_location: Option<AssetLocation>,
}

impl ImportFilesModal {
    pub fn new(
        files_to_import: Vec<PathBuf>,
        importer_registry: &ImporterRegistry,
    ) -> Self {
        let mut all_files_to_import = HashSet::default();
        for file in &files_to_import {
            // Recursively look for files
            if file.is_dir() {
                let walker = globwalk::GlobWalkerBuilder::from_patterns(file, &["**"])
                    .file_type(globwalk::FileType::FILE)
                    .build()
                    .unwrap();

                for file in walker {
                    if let Ok(file) = file {
                        let file = dunce::canonicalize(&file.path()).unwrap();
                        if let Some(extension) = file.extension() {
                            if !importer_registry
                                .importers_for_file_extension(&*extension.to_string_lossy())
                                .is_empty()
                            {
                                all_files_to_import.insert(file.to_path_buf());
                                println!("import {:?}", file);
                            }
                        }
                    }
                }
            } else {
                all_files_to_import.insert(file.to_path_buf());
            }
        }

        ImportFilesModal {
            files_to_import: all_files_to_import,
            selected_location: None,
        }
    }
}

impl ModalAction for ImportFilesModal {
    fn draw(
        &mut self,
        context: ModalContext,
    ) -> ModalActionControlFlow {
        let mut control_flow = ModalActionControlFlow::Continue;
        default_modal_window("Import Files", context, |context, ui| {
            ui.label("Files to be imported:");

            egui::ScrollArea::vertical()
                .id_source("files")
                .auto_shrink([false, false])
                .max_height(200.0)
                .show(ui, |ui| {
                    for file in &self.files_to_import {
                        ui.label(file.to_string_lossy());
                    }
                });

            ui.separator();
            ui.label("Where to import the files");

            egui::ScrollArea::vertical()
                .id_source("locations")
                .auto_shrink([false, false])
                .max_height(200.0)
                .show(ui, |ui| {
                    draw_location_selector(
                        ui,
                        &context.db_state.editor_model,
                        context.action_queue,
                        context.ui_state,
                        &mut self.selected_location,
                    );
                });

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    control_flow = ModalActionControlFlow::End;
                }

                //TODO: Make this disable if location not set
                if ui.add_enabled(self.selected_location.is_some(), egui::Button::new("Import")).clicked() {
                    let mut import_job_to_queue = ImportJobToQueue::default();
                    for file in &self.files_to_import {
                        let extension = file.extension();
                        if let Some(extension) = extension {
                            let extension = extension.to_string_lossy().to_string();
                            let handlers = context.asset_engine.importers_for_file_extension(&extension);

                            if !handlers.is_empty() {
                                //
                                // Find the importer to use on the file
                                //
                                let importer = context.asset_engine.importer(handlers[0]).unwrap();

                                log::info!("Starting import recursively on {:?}", file);
                                hydrate_model::pipeline::recursively_gather_import_operations_and_create_assets(
                                    &context.db_state.project_configuration,
                                    file,
                                    importer,
                                    context.db_state.editor_model.root_edit_context(),
                                    context.asset_engine.importer_registry(),
                                    &self.selected_location.unwrap(),
                                    &mut import_job_to_queue,
                                ).unwrap();
                            }
                        }
                    }

                    if !import_job_to_queue.is_empty() {
                        context.asset_engine.queue_import_operation(import_job_to_queue);
                    }

                    control_flow = ModalActionControlFlow::End;
                }
            });
        });

        control_flow
    }
}
