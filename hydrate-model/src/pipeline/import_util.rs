use crate::edit_context::EditContext;
use crate::pipeline::Importer;
use crate::pipeline::ImporterRegistry;
use crate::{
    HashMap, ImportInfo, ImporterId, AssetId, AssetLocation, AssetName, ScannedImportable,
};
use hydrate_base::hashing::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ImportToQueue {
    pub source_file_path: PathBuf,
    pub importer_id: ImporterId,
    pub requested_importables: HashMap<Option<String>, AssetId>,
    pub assets_to_regenerate: HashSet<AssetId>,
}

pub fn create_import_info(
    source_file_path: &Path,
    importer: &Box<dyn Importer>,
    scanned_importable: &ScannedImportable,
) -> ImportInfo {
    let mut file_references = Vec::default();
    for file_reference in &scanned_importable.file_references {
        file_references.push(file_reference.path.clone());
    }

    //
    // When we import, set the import info so we track where the import comes from
    //
    ImportInfo::new(
        importer.importer_id(),
        source_file_path.to_path_buf(),
        scanned_importable.name.clone().unwrap_or_default(),
        file_references,
    )
}

pub fn create_object_name(
    source_file_path: &Path,
    scanned_importable: &ScannedImportable,
) -> AssetName {
    if let Some(file_name) = source_file_path.file_name() {
        let file_name = file_name.to_string_lossy();
        if let Some(importable_name) = &scanned_importable.name {
            AssetName::new(format!("{}.{}", file_name, importable_name))
        } else {
            AssetName::new(file_name.to_string())
        }
    } else {
        AssetName::empty()
    }
}

pub fn recursively_gather_import_operations_and_create_assets(
    source_file_path: &Path,
    importer: &Box<dyn Importer>,
    editor_context: &mut EditContext,
    importer_registry: &ImporterRegistry,
    //asset_engine: &AssetEngine,
    selected_import_location: &AssetLocation,

    // In addition to being the imports that need to be queued, this is also the objects that were
    // created. Pre-existing but referenced objects won't be in this list
    imports_to_queue: &mut Vec<ImportToQueue>,
) -> Option<AssetId> {
    //
    // We now build a list of things we will be importing from the file.
    // 1. Scan the file to see what's available
    // 2. Create/Find objects for all the things we want to import
    // 3. Enqueue the import operation
    //
    let mut requested_importables = HashMap::default();
    let mut default_importable_asset_id = None;
    let mut assets_to_regenerate = HashSet::default();

    let scanned_importables = importer.scan_file(
        source_file_path,
        editor_context.schema_set(),
        importer_registry,
    );
    for scanned_importable in &scanned_importables {
        // let mut file_references = Vec::default();
        // for file_reference in &scanned_importable.file_references {
        //     file_references.push(file_reference.path.clone());
        // }
        //
        // //
        // // When we import, set the import info so we track where the import comes from
        // //
        // let import_info = ImportInfo::new(
        //     importer.importer_id(),
        //     source_file_path.to_path_buf(),
        //     scanned_importable.name.clone().unwrap_or_default(),
        //     file_references,
        // );
        let import_info = create_import_info(source_file_path, importer, scanned_importable);

        //
        // Pick name for the asset for this file
        //
        let object_name = create_object_name(source_file_path, scanned_importable);

        let mut referenced_source_file_asset_ids = Vec::default();

        //TODO: Check referenced source files to find existing imported assets or import referenced files
        for referenced_source_file in &scanned_importable.file_references {
            let referenced_file_absolute_path = if referenced_source_file.path.is_relative() {
                source_file_path
                    .parent()
                    .unwrap()
                    .join(&referenced_source_file.path)
                    .canonicalize()
                    .unwrap()
            } else {
                referenced_source_file.path.clone()
            };

            // Does it already exist?
            let mut found = None;
            for asset_id in editor_context.all_objects() {
                if let Some(import_info) = editor_context.import_info(*asset_id) {
                    if import_info.importable_name().is_empty()
                        && import_info.source_file_path() == referenced_file_absolute_path
                    {
                        found = Some(*asset_id);
                    }
                }
            }

            // If we didn't find it, try to import it
            if found.is_none() {
                let importer = importer_registry
                    .importer(referenced_source_file.importer_id)
                    .unwrap();
                found = recursively_gather_import_operations_and_create_assets(
                    &referenced_file_absolute_path,
                    importer,
                    editor_context,
                    importer_registry,
                    selected_import_location,
                    imports_to_queue,
                );
            }

            referenced_source_file_asset_ids.push(found);
        }

        // At this point all referenced files have either been found or imported
        assert_eq!(
            referenced_source_file_asset_ids.len(),
            scanned_importable.file_references.len()
        );

        let asset_id = editor_context.new_object(
            &object_name,
            selected_import_location,
            &scanned_importable.asset_type,
        );
        //TODO: Do this when we actually import to avoid potential race conditions
        editor_context.set_import_info(asset_id, import_info.clone());

        for (k, v) in scanned_importable
            .file_references
            .iter()
            .zip(referenced_source_file_asset_ids)
        {
            if let Some(v) = v {
                editor_context.set_file_reference_override(asset_id, k.path.clone(), v);
            }
        }

        requested_importables.insert(scanned_importable.name.clone(), asset_id);

        // These are all newly created objects so we should populate their properties based on source file contents
        // A re-import of data from the source file might not want to do this
        assets_to_regenerate.insert(asset_id);

        //editor_context.build_info_mut().

        if scanned_importable.name.is_none() {
            default_importable_asset_id = Some(asset_id);
        }
    }

    //asset_engine.queue_import_operation(asset_ids, importer.importer_id(), file.to_path_buf());
    //(asset_ids, importer.importer_id(), file.to_path_buf())
    imports_to_queue.push(ImportToQueue {
        source_file_path: source_file_path.to_path_buf(),
        importer_id: importer.importer_id(),
        requested_importables,
        assets_to_regenerate,
    });

    default_importable_asset_id
}
