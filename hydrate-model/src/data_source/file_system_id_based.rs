use crate::edit_context::EditContext;
use hydrate_base::uuid_path::{path_to_uuid, uuid_to_path};
use crate::{AssetEngine, DataSource, HashSet, ObjectId, ObjectSourceId, PathNode, PathNodeRoot};
use std::path::{Path, PathBuf};
use uuid::Uuid;
use hydrate_data::ObjectLocation;
use hydrate_schema::SchemaNamedType;
use crate::import_util::ImportToQueue;

fn load_asset_files(
    edit_context: &mut EditContext,
    root_path: &Path,
    object_source_id: ObjectSourceId,
    all_object_ids_on_disk: &mut HashSet<ObjectId>,
) {
    let walker = globwalk::GlobWalkerBuilder::from_patterns(root_path, &["**.af"])
        .file_type(globwalk::FileType::FILE)
        .build()
        .unwrap();

    for file in walker {
        if let Ok(file) = file {
            println!("asset file {:?}", file);
            let file_uuid = path_to_uuid(root_path, file.path()).unwrap();
            let contents = std::fs::read_to_string(file.path()).unwrap();
            crate::json_storage::EditContextObjectJson::load_edit_context_object_from_string(
                edit_context,
                Some(file_uuid),
                object_source_id,
                None,
                &contents,
            );
            let object_id = ObjectId(file_uuid.as_u128());
            let object_location = edit_context
                .objects()
                .get(&object_id)
                .unwrap()
                .object_location()
                .clone();
            edit_context.clear_object_modified_flag(object_id);
            edit_context.clear_location_modified_flag(&object_location);
            all_object_ids_on_disk.insert(object_id);
        }
    }
}

pub struct FileSystemIdBasedDataSource {
    object_source_id: ObjectSourceId,
    file_system_root_path: PathBuf,

    // Any object ID we know to exist on disk is in this list to help us quickly determine which
    // deleted IDs need to be cleaned up
    all_object_ids_on_disk: HashSet<ObjectId>,

    path_node_schema: SchemaNamedType,
    path_node_root_schema: SchemaNamedType,
}

impl FileSystemIdBasedDataSource {
    fn is_object_owned_by_this_data_source(&self, edit_context: &EditContext, object_id: ObjectId) -> bool {
        if edit_context.object_schema(object_id).unwrap().fingerprint() == self.path_node_root_schema.fingerprint() {
            return false;
        }

        //TODO: is_null means we default to using this source
        let root_location = edit_context.object_location_chain(object_id).last().cloned().unwrap_or_else(ObjectLocation::null);
        root_location.path_node_id().as_uuid() == *self.object_source_id.uuid() || root_location.is_null()
    }

    pub fn object_source_id(&self) -> ObjectSourceId {
        self.object_source_id
    }

    pub fn new<RootPathT: Into<PathBuf>>(
        file_system_root_path: RootPathT,
        edit_context: &mut EditContext,
        object_source_id: ObjectSourceId,
    ) -> Self {
        let path_node_schema = edit_context.schema_set().find_named_type(PathNode::schema_name()).unwrap().clone();
        let path_node_root_schema = edit_context.schema_set().find_named_type(PathNodeRoot::schema_name()).unwrap().clone();

        let file_system_root_path = file_system_root_path.into();
        log::info!(
            "Creating file system object data source {:?}",
            file_system_root_path,
        );

        FileSystemIdBasedDataSource {
            object_source_id,
            file_system_root_path: file_system_root_path.into(),
            all_object_ids_on_disk: Default::default(),
            path_node_schema,
            path_node_root_schema,
        }
    }

    fn find_all_modified_objects(&self, edit_context: &EditContext) -> HashSet<ObjectId> {
        // We need to handle objects that were moved into this data source that weren't previous in it
        let mut modified_objects = edit_context.modified_objects().clone();

        for object_id in edit_context.objects().keys() {
            if self.is_object_owned_by_this_data_source(edit_context, *object_id) {
                if !self.all_object_ids_on_disk.contains(object_id) {
                    modified_objects.insert(*object_id);
                }
            }
        }

        modified_objects
    }
}

impl DataSource for FileSystemIdBasedDataSource {
    fn reload_all(
        &mut self,
        edit_context: &mut EditContext,
        imports_to_queue: &mut Vec<ImportToQueue>,
    ) {
        load_asset_files(
            edit_context,
            &self.file_system_root_path,
            self.object_source_id,
            &mut self.all_object_ids_on_disk,
        );
    }

    fn save_all_modified(
        &mut self,
        edit_context: &mut EditContext,
    ) {
        // Delete files for objects that were deleted
        let modified_objects = self.find_all_modified_objects(edit_context);
        for object_id in &modified_objects {
            if self.all_object_ids_on_disk.contains(&object_id)
                && !edit_context.has_object(*object_id)
            {
                //TODO: delete the object file
                self.all_object_ids_on_disk.remove(&object_id);
            }
        }

        for object_id in &modified_objects {
            if let Some(object_info) = edit_context.objects().get(object_id) {
                if self.is_object_owned_by_this_data_source(edit_context, *object_id) {
                    if object_id.as_uuid() == *self.object_source_id.uuid() {
                        // never save the root object
                        continue;
                    }

                    let parent_dir = object_info.object_location().path_node_id().as_uuid();
                    let parent_dir = if parent_dir == Uuid::nil() || parent_dir == *self.object_source_id.uuid() {
                        None
                    } else {
                        Some(parent_dir)
                    };

                    let data = crate::json_storage::EditContextObjectJson::save_edit_context_object_to_string(
                        edit_context,
                        *object_id,
                        false, //don't include ID because we assume it by file name
                        parent_dir
                    );
                    let file_path =
                        uuid_to_path(&self.file_system_root_path, object_id.as_uuid(), "af");
                    self.all_object_ids_on_disk.insert(*object_id);

                    if let Some(parent) = file_path.parent() {
                        std::fs::create_dir_all(parent).unwrap();
                    }

                    std::fs::write(file_path, data).unwrap();
                }
            }
        }
    }

    fn reload_all_modified(
        &mut self,
        edit_context: &mut EditContext,
    ) {
        let mut existing_modified_objects: Vec<_> = Default::default();
        let mut saved_modified_objects: Vec<_> = Default::default();

        // Find all existing modified objects
        // I think this includes added, deleted, and edited objects?
        for modified_object in &self.find_all_modified_objects(edit_context) {
            if let Some(object_info) = edit_context.objects().get(modified_object) {
                if self.is_object_owned_by_this_data_source(edit_context, *modified_object) {
                    existing_modified_objects.push(*modified_object);
                }
            }

            if self.all_object_ids_on_disk.contains(modified_object) {
                saved_modified_objects.push(*modified_object);
            }
        }

        // Delete any modified object that exists in the edit context belonging to this data source
        for modified_object in existing_modified_objects {
            edit_context.delete_object(modified_object);
        }

        // Reload any modified object that exists on disk belonging to this data source
        for modified_object in saved_modified_objects {
            let file_path =
                uuid_to_path(&self.file_system_root_path, modified_object.as_uuid(), "af");

            if let Ok(contents) = std::fs::read_to_string(file_path) {
                crate::json_storage::EditContextObjectJson::load_edit_context_object_from_string(
                    edit_context,
                    Some(modified_object.as_uuid()),
                    self.object_source_id,
                    None,
                    &contents
                );
            }
        }
    }
}
