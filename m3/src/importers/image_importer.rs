
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

#[derive(Serialize, Deserialize)]
struct ImageProcessedData {
    image_bytes: Vec<u8>,
    width: u32,
    height: u32,
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

    fn asset_types(&self) -> &[&'static str] {
        &["ImageAsset"]
    }

    fn create_default_asset(&self, editor_model: &mut EditorModel, object_name: ObjectName, object_location: ObjectLocation) -> ObjectId {
        let schema_record = editor_model.root_edit_context_mut().schema_set().find_named_type(ImageAsset::schema_name()).unwrap().as_record().unwrap().clone();
        editor_model.root_edit_context_mut().new_object(&object_name, &object_location, &schema_record)
    }

    fn scan_file(&self, path: &Path) {
        // Nothing to do, images don't reference other files or assets
    }

    fn import_file(
        &self,
        path: &Path,
        object_id: ObjectId,
        data_set: &mut DataSet,
        schema: &SchemaSet,
    ) -> SingleObject {
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
        import_object
    }
}

#[derive(TypeUuid)]
#[uuid = "da6760e7-5b24-43b4-830d-6ee4515096b8"]
pub struct ImageProcessor {}

impl Processor for ImageProcessor {
    fn process_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
    ) -> Vec<u8> {
        //
        // Read asset properties
        //
        let compressed = data_set
            .resolve_property(schema, asset_id, "compress")
            .unwrap()
            .as_boolean()
            .unwrap();
        let imported_data = data_set
            .resolve_property(schema, asset_id, "imported_data")
            .unwrap()
            .as_object_ref()
            .unwrap();

        //
        // Read imported data
        //
        let image_bytes = data_set
            .resolve_property(schema, imported_data, "image_bytes")
            .unwrap()
            .as_bytes()
            .unwrap()
            .clone();
        let width = data_set
            .resolve_property(schema, imported_data, "width")
            .unwrap()
            .as_u32()
            .unwrap();
        let height = data_set
            .resolve_property(schema, imported_data, "height")
            .unwrap()
            .as_u32()
            .unwrap();

        //
        // Compress the image, or just return the raw image bytes
        //
        let image_bytes = if compressed {
            let mut compressor_params = basis_universal::CompressorParams::new();
            compressor_params.set_basis_format(basis_universal::BasisTextureFormat::UASTC4x4);
            compressor_params.set_generate_mipmaps(true);
            compressor_params.set_color_space(basis_universal::ColorSpace::Srgb);
            compressor_params.set_uastc_quality_level(basis_universal::UASTC_QUALITY_DEFAULT);

            let mut source_image = compressor_params.source_image_mut(0);

            source_image.init(&image_bytes, width, height, 4);
            let mut compressor = basis_universal::Compressor::new(4);
            unsafe {
                compressor.init(&compressor_params);
                log::debug!("Compressing texture");
                compressor.process().unwrap();
                log::debug!("Compressed texture");
            }
            let compressed_basis_data = compressor.basis_file().to_vec();
            compressed_basis_data
        } else {
            image_bytes
        };

        let processed_data = ImageProcessedData {
            image_bytes,
            width,
            height,
        };

        let serialized = bincode::serialize(&processed_data).unwrap();
        serialized
    }
}
