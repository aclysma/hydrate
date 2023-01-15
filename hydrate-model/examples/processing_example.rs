use hydrate_model::edit_context::EditContext;
use hydrate_model::{
    BufferId, DataObjectInfo, DataSet, EditorModel, HashMap, ObjectId, ObjectLocation, ObjectPath,
    SchemaLinker, SchemaSet, Value,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

// fn uuid_to_path(root_path: &Path, uuid: Uuid, extension: &str) -> PathBuf {
//     // Convert UUID to a 32-character hex string (no hyphens)
//     // example: 8cf25195abd839981ea3c93c8fd2843f
//     let mut buffer = [0; 32];
//     let encoded = uuid.to_simple().encode_lower(&mut buffer).to_string();
//     // Produce path like [root_path]/8/cf/25195abd839981ea3c93c8fd2843f
//     root_path.join(&encoded[0..1]).join(&encoded[1..3]).join(format!("{}.{}", &encoded[3..32], extension))
// }
//
// fn path_to_uuid(root_path: &Path, file_path: &Path) -> Option<Uuid> {
//     // Remove root_path from the path
//     let relative_path_from_root = file_path
//         .strip_prefix(root_path)
//         .ok()?;
//
//     // We append the path into this string
//     let mut path_and_name = String::with_capacity(32);
//
//     // Split the path by directory paths
//     let components: Vec<_> = relative_path_from_root.components().collect();
//
//     // Iterate all segments of the path except the last one
//     if components.len() > 1 {
//         for component in components[0..components.len()-1].iter() {
//             path_and_name.push_str(&component.as_os_str().to_str().unwrap());
//         }
//     }
//
//     // Append the last segment, removing the extension if there is one
//     if let Some(last_component) = components.last() {
//         let mut last_str = last_component.as_os_str().to_str()?;
//
//         // Remove the extension
//         if let Some(extension_begin) = last_str.find('.') {
//             last_str = last_str.strip_suffix(&last_str[extension_begin..]).unwrap();
//         }
//
//         // Add zero padding between dirs (which should be highest order bits) and filename
//         //TODO: Maybe just assert all the component lengths are as expected
//         let str_len = path_and_name.len() + last_str.len();
//         if str_len < 32 {
//             path_and_name.push_str(&"0".repeat(32 - str_len));
//         }
//
//         path_and_name.push_str(last_str);
//     }
//
//     u128::from_str_radix(&path_and_name, 16).ok().map(|x| Uuid::from_u128(x))
// }
//
//
//
// fn find_files<P: AsRef<Path>, S: AsRef<str>>(root_path: P, patterns: &[S]) -> Vec<Uuid>
// {
//     let root_path = root_path.as_ref();
//     let walker = globwalk::GlobWalkerBuilder::from_patterns(root_path, patterns)
//         .file_type(globwalk::FileType::FILE)
//         .build()
//         .unwrap();
//
//     let mut files = HashMap::<Uuid, DirectoryFile>::default();
//
//     for file in walker {
//         if let Ok(file) = file {
//             //println!("dir file {:?}", file);
//             let dir_uuid = path_to_uuid(root_path, file.path()).unwrap();
//             file.path().extension();
//             //let contents = std::fs::read_to_string(file.path()).unwrap();
//
//             //directories.insert(dir_uuid, dir_file);
//         }
//     }
//
//     files
// }
//
//
//

// How to handle multiple objects
// Subobjects? We have type-safe migrated data stored as subobjects and buffers
// We have:
// - Authored data that describes an import operation
//   - 1:1 relationship to an import object of same ID as the authored object of a pre-defined type
//    - It can create more import sub objects/buffers on import, maybe there's a table of contents/dictionary
//      to other objects? Maybe we store errors/warnings in here?
// -
//

struct BufferState {
    modified: bool,
    loaded: bool,
    location: String, //enum? Arc?

    size: usize,
    data: Option<Vec<u8>>,
}

//TODO: Do we store the buffers in different sets? Maybe we have it in same set but bookkeep where
// the buffers came from. Maybe this is part of DataSet? However we don't want undo/redo for
// this stuff I think?
struct BufferSet {
    buffers: HashMap<BufferId, BufferState>,
}

impl BufferSet {
    fn load_buffer() {}
    fn unload_buffer() {}

    // Requires buffer to be loaded to get
    fn get_buffer_data() {}
    // If setting, it's immediately in loaded state
    fn set_buffer_data() {}

    fn save_buffer() {}
    fn save_all_dirty_buffers() {}
}

struct BuildObjectId(Uuid);

struct BuildObjectState {
    object: DataObjectInfo,
}

struct BuildObjectSet {
    objects: HashMap<BuildObjectId, BuildObjectState>,
    buffers: BufferSet,
}

struct ImportObjectId(Uuid);

struct ImportObjectState {
    object: DataObjectInfo,
}

struct ImportObjectSet {
    objects: HashMap<ImportObjectId, ImportObjectState>,
    buffers: BufferSet,
}

//Need a different way to store import and build data for objects?
// DataSet using buffers?
// Implement buffers?

//RAII object to load a buffer?
struct LoadedBuffer {}
/*
struct BufferSet {

}

impl BufferSet {
    fn load_buffer(&self, buffer: u128) {

    }

    fn unload_buffer(&self, buffer: u128) {

    }
}*/

#[derive(Default)]
struct ImageAsset {
    path: String,
    compress: bool,
}

impl ImageAsset {
    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type("ImageAsset", |x| {
                x.add_string("path");
                x.add_boolean("compress");
            })
            .unwrap();
    }

    pub fn read_from_dataset(
        &mut self,
        edit_context: &EditContext,
        object_id: ObjectId,
    ) {
        self.path = edit_context
            .resolve_property(object_id, "path")
            .unwrap()
            .as_string()
            .unwrap()
            .to_string();
        self.compress = edit_context
            .resolve_property(object_id, "compress")
            .unwrap()
            .as_boolean()
            .unwrap();
    }

    pub fn write_to_dataset(
        &self,
        edit_context: &mut EditContext,
        object_id: ObjectId,
    ) {
        edit_context.set_property_override(object_id, "path", Value::String(self.path.clone()));
        edit_context.set_property_override(object_id, "compress", Value::Boolean(self.compress));
    }
}

struct ImageAssetImported {
    data: Vec<u8>,
}

// struct ImageAssetBuilt {
//     data: Vec<u8>
// }
//
// struct ImageAssetImportJob {
//
// }
//
// impl ImageAssetImportJob {
//     pub fn update() {
//         // open file
//         // return
//     }
// }

// Images
// - ImageAsset
//   - Color space
//   - Compression settings
//   - Path to single image file
// - ImageImportOp
//   - Usually a single resource
// - ImageBuildOp
//   -
//
// Materials
// - MaterialAsset
//   - Path to json description
// - MaterialImportOp
//   - Probably a single resource, but maybe there's support for various overrides
// - Built
//   - We need to convert image path to build image UUID?

// Meshes
// - MeshAsset
//   - Path to mesh export blob
//   - compression settings
//   - batching settings
// - MeshImportOp
//   - Ingest all the data
// - MeshBuildOp
//   - Create buffers for vertex/index data?
//   - Metadata needed for those buffers
//   - Reference to material

//

// Handling a path reference for materials?
// Do we let importers associate paths with import objects?

struct ProcessAssetResult {
    // top-level resource
    // additional jobs?
    // additional resources?
}

trait AssetProcessor {
    fn process_asset(database: &DataSet) -> ProcessAssetResult;
}

struct AssetProcessorImage {}

struct AssetProcessorRegistry {}

fn main() {
    //
    // Setup logging
    //
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    //
    // Setup schema (this could be via plugins later)
    //
    let mut linker = SchemaLinker::default();
    ImageAsset::register_schema(&mut linker);

    let mut schema_set = SchemaSet::default();
    schema_set.add_linked_types(linker).unwrap();
    let schema_set = Arc::new(schema_set);

    //
    // Create editor model using the schema
    //
    let mut editor_model = EditorModel::new(schema_set.clone());

    //
    // Register the data source
    //
    let data_source_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/examples/data/processing_example/asset_data"
    ));
    let source_id =
        editor_model.add_file_system_object_source(data_source_path, ObjectPath::root());

    let image_asset_schema_record = schema_set
        .find_named_type("ImageAsset")
        .unwrap()
        .as_record()
        .unwrap()
        .clone();

    //
    // Create an image asset if one doesn't exist
    //
    if editor_model.root_edit_context().objects().is_empty() {
        println!("creating new object");

        // Create some authored data that points to objects to import
        let mut editor_context = editor_model.root_edit_context_mut();
        let object_id = editor_context.new_object(
            &ObjectLocation::new(source_id, ObjectPath::root()),
            &image_asset_schema_record,
        );
        let image_asset = ImageAsset {
            path: "source_data/test_texture.jpg".to_string(),
            compress: false,
        };

        image_asset.write_to_dataset(editor_context, object_id);

        editor_model.save_root_edit_context();
    } else {
        println!("using already-created object")
    }

    //
    // Find image assets
    //
    let mut image_asset_jobs = Vec::default();
    for (object_id, object_data) in editor_model.root_edit_context().objects() {
        if object_data.schema().fingerprint() == image_asset_schema_record.fingerprint() {
            image_asset_jobs.push(*object_id);
        }
    }

    for image_asset_id in image_asset_jobs {
        let mut image_asset = ImageAsset::default();
        image_asset.read_from_dataset(editor_model.root_edit_context(), image_asset_id);

        // process image asset
        let imported = ImageAssetImported {
            data: Default::default(),
        };

        // Store it somewhere?
    }

    //editor_model.save_root_edit_context();

    // Run the import jobs
    // -

    //editor_context.objects()

    // Run the build jobs

    // Demonstrate that the expected data was produced

    // Verify result from processing
}
