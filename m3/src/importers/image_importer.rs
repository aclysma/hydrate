
pub use super::*;

pub struct ImageAsset {}

impl ImageAsset {
    pub fn schema_name() -> &'static str {
        "ImageAsset"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                //x.add_reference("imported_data", ImageImportedData::schema_name());
                //x.add_string("path");
                x.add_boolean("compress");
            })
            .unwrap();
    }
}

// pub struct ImageImportOptions {}
//
// impl ImageImportOptions {
//     pub fn schema_name() -> &'static str {
//         "ImageImportOptions"
//     }
//
//     pub fn register_schema(linker: &mut SchemaLinker) {
//         linker
//             .register_record_type(Self::schema_name(), |x| {
//                 // No options
//             })
//             .unwrap();
//     }
// }

pub struct ImageImportedData {}

impl ImageImportedData {
    pub fn schema_name() -> &'static str {
        "ImageImportedData"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                //x.add_reference("asset", ImageImportedData::schema_name());
                x.add_bytes("image_bytes"); // TODO: this would be a buffer
                x.add_u32("width");
                x.add_u32("height");
            })
            .unwrap();
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "e7c83acb-f73b-4b3c-b14d-fe5cc17c0fa3"]
pub struct ImageImporter;

impl Importer for ImageImporter {
    fn register_schemas(&self, schema_linker: &mut SchemaLinker) {
        ImageAsset::register_schema(schema_linker);
        ImageImportedData::register_schema(schema_linker);
    }

    fn supported_file_extensions(&self) -> &[&'static str] {
        &["png"]
    }

    fn scan_file(&self, path: &Path, schema_set: &SchemaSet) -> Vec<ScannedImportable> {
        let asset_type = schema_set.find_named_type(ImageAsset::schema_name()).unwrap().as_record().unwrap().clone();
        vec![ScannedImportable {
            name: None,
            asset_type,
            referenced_source_files: Default::default()
        }]
    }

    // fn asset_types(&self) -> &[&'static str] {
    //     &["ImageAsset"]
    // }

    // fn create_default_import_options(&self, schema_set: &SchemaSet) -> SingleObject {
    //     let options_type = schema_set.find_named_type(ImageImportedData::schema_name()).unwrap();
    //     SingleObject::new(options_type.as_record().unwrap())
    // }

    fn create_default_asset(&self, editor_model: &mut EditorModel, object_name: ObjectName, object_location: ObjectLocation) -> ObjectId {
        let schema_record = editor_model.root_edit_context_mut().schema_set().find_named_type(ImageAsset::schema_name()).unwrap().as_record().unwrap().clone();
        editor_model.root_edit_context_mut().new_object(&object_name, &object_location, &schema_record)
    }

    // fn scan_for_referenced_source_file_paths(&self, path: &Path) {
    //     // Nothing to do, images don't reference other files or assets
    // }

    fn import_file(
        &self,
        path: &Path,
        object_ids: &HashMap<Option<String>, ObjectId>,
        schema: &SchemaSet,
        //import_info: &ImportInfo,
        referenced_source_file_paths: &mut Vec<PathBuf>,
    ) -> HashMap<Option<String>, SingleObject> {
        // TODO: Replace with a shim so we can track what files are being read
        // - We trigger the importer for them by specifying the file path and kind of file (i.e. an image, specific type of JSON file, etc.)
        // - We may need to let the "import" dialog try to perform the import to get error messages and discover what will end up being imported
        let bytes = std::fs::read(path).unwrap();

        let decoded_image =
            ::image::load_from_memory_with_format(&bytes, ::image::ImageFormat::Png).unwrap();
        let (width, height) = decoded_image.dimensions();
        let image_bytes = decoded_image.into_rgba8().to_vec();

        let image_imported_data_schema = schema
            .find_named_type(ImageImportedData::schema_name())
            .unwrap()
            .as_record()
            .unwrap();

        let mut import_object = SingleObject::new(image_imported_data_schema);
        import_object.set_property_override(schema, "image_bytes", Value::Bytes(image_bytes));
        import_object.set_property_override(schema, "width", Value::U32(width));
        import_object.set_property_override(schema, "height", Value::U32(height));

        let mut imported_objects = HashMap::default();
        imported_objects.insert(None, import_object);
        imported_objects
    }
}
