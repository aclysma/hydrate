mod file_system_id_based;

pub use file_system_id_based::*;
use std::path::PathBuf;

use crate::edit_context::EditContext;
use crate::AssetId;

mod file_system_path_based;
pub use file_system_path_based::*;
use hydrate_pipeline::{HydrateProjectConfiguration, ImportJobSourceFile, ImportJobToQueue};

#[derive(Default)]
pub struct PendingFileOperations {
    pub create_operations: Vec<(AssetId, PathBuf)>,
    pub modify_operations: Vec<(AssetId, PathBuf)>,
    pub delete_operations: Vec<(AssetId, PathBuf)>,
}

pub trait DataSource {
    // Replace memory with storage state
    // Reset memory to storage
    // Load storage state to memory
    fn load_from_storage(
        &mut self,
        project_config: &HydrateProjectConfiguration,
        edit_context: &mut EditContext,
        import_job_to_queue: &mut ImportJobToQueue,
    );

    // Replace storage state with memory state
    // Flush memory to storage
    fn flush_to_storage(
        &mut self,
        edit_context: &mut EditContext,
    );

    fn is_generated_asset(
        &self,
        asset_id: AssetId,
    ) -> bool;

    // fn asset_symbol_name(
    //     &self,
    //     asset_id: AssetId
    // ) -> Option<String>;

    fn persist_generated_asset(
        &mut self,
        edit_context: &mut EditContext,
        asset_id: AssetId,
    );

    fn edit_context_has_unsaved_changes(
        &self,
        edit_context: &EditContext,
    ) -> bool;

    fn append_pending_file_operations(
        &self,
        edit_context: &EditContext,
        pending_file_operations: &mut PendingFileOperations,
    );

    // fn revert_all_modified(
    //     &mut self,
    //     edit_context: &mut EditContext,
    //     import_jobs_to_queue: &mut Vec<ImportJobToQueue>,
    // );

    // fn get_file_operations_required_to_save();
    //
    //
    //
    // fn save_assets(objects: &[ObjectId]);
}
