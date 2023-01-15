use crate::edit_context::EditContext;
use crate::uuid_path::{path_to_uuid, uuid_to_path};
use crate::{DataSource, HashMap, HashSet, ObjectId, ObjectLocation, ObjectPath, ObjectSourceId};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

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
            crate::data_storage::json::EditContextObjectJson::load_edit_context_object_from_string(
                edit_context,
                file_uuid,
                object_source_id,
                &contents,
            );
            let object_id = ObjectId(file_uuid.as_u128());
            let object_location = edit_context
                .objects()
                .get(&object_id)
                .unwrap()
                .object_location
                .clone();
            edit_context.clear_object_modified_flag(object_id);
            edit_context.clear_location_modified_flag(&object_location);
            all_object_ids_on_disk.insert(object_id);
        }
    }
}

pub struct FileSystemObjectDataSource {
    object_source_id: ObjectSourceId,
    file_system_root_path: PathBuf,
    //file_states: HashMap<PathBuf, FileState>,
    //object_locations: HashMap<ObjectId, PathBuf>,
    //dir_uuid_to_path: HashMap<Uuid, ObjectPath>,
    //path_to_dir_uuid: HashMap<ObjectPath, Uuid>,

    // Any object ID we know to exist on disk is in this list to help us quickly determine which
    // deleted IDs need to be cleaned up
    all_object_ids_on_disk: HashSet<ObjectId>,
}

impl DataSource for FileSystemObjectDataSource {
    fn reload_all(
        &mut self,
        edit_context: &mut EditContext,
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
        for object_id in edit_context.modified_objects() {
            if self.all_object_ids_on_disk.contains(object_id)
                && !edit_context.has_object(*object_id)
            {
                //TODO: delete the object file
                self.all_object_ids_on_disk.remove(object_id);
            }
        }

        for object_id in edit_context.modified_objects() {
            if let Some(object_info) = edit_context.objects().get(object_id) {
                if object_info.object_location().source() == self.object_source_id {
                    //let object_path = object_info.object_location.path();
                    //let parent_dir = self.path_to_dir_uuid.get(object_path).copied();

                    //TODO: create dir objects?
                    //let parent_dir = self.get_or_create_dir(object_path);

                    let parent_dir = object_info.object_location().path_node_id().as_uuid();
                    let parent_dir = if parent_dir == Uuid::nil() {
                        None
                    } else {
                        Some(parent_dir)
                    };

                    let data = crate::data_storage::json::EditContextObjectJson::save_edit_context_object_to_string(edit_context, *object_id, parent_dir);
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
        for modified_object in edit_context.modified_objects() {
            if let Some(object_info) = edit_context.objects().get(modified_object) {
                if object_info.object_location().source() == self.object_source_id {
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
                crate::data_storage::json::EditContextObjectJson::load_edit_context_object_from_string(edit_context, modified_object.as_uuid(), self.object_source_id, &contents);
            }
        }
    }
}

impl FileSystemObjectDataSource {
    pub fn object_source_id(&self) -> ObjectSourceId {
        self.object_source_id
    }

    pub fn new<RootPathT: Into<PathBuf>>(
        file_system_root_path: RootPathT,
        edit_context: &mut EditContext,
    ) -> Self {
        let object_source_id = ObjectSourceId::new();
        let file_system_root_path = file_system_root_path.into();
        log::info!(
            "Creating file system object data source {:?}",
            file_system_root_path,
        );

        FileSystemObjectDataSource {
            object_source_id,
            file_system_root_path: file_system_root_path.into(),
            all_object_ids_on_disk: Default::default(),
        }
    }
}
