
pub use super::*;



#[derive(Serialize, Deserialize)]
struct MaterialJsonFileFormat {
    pub base_color_factor: [f32; 4], // default: 1,1,1,1
    pub emissive_factor: [f32; 3],   // default: 0,0,0
    pub metallic_factor: f32,        // default: 1,
    pub roughness_factor: f32,       // default: 1,
    pub normal_texture_scale: f32,   // default: 1

    #[serde(default)]
    pub color_texture: Option<String>,
    #[serde(default)]
    pub metallic_roughness_texture: Option<String>,
    #[serde(default)]
    pub normal_texture: Option<String>,
    #[serde(default)]
    pub emissive_texture: Option<String>,

    #[serde(default)]
    pub shadow_method: Option<String>,
    #[serde(default)]
    pub blend_method: Option<String>,
    #[serde(default)]
    pub alpha_threshold: Option<f32>,
    #[serde(default)]
    pub backface_culling: Option<bool>,
    #[serde(default)]
    pub color_texture_has_alpha_channel: bool,
}






pub struct MaterialAsset {}

impl MaterialAsset {
    pub fn schema_name() -> &'static str {
        "MaterialAsset"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                //x.add_reference("imported_data", MaterialImportedData::schema_name());
                //x.add_string("path");
                //x.add_boolean("compress");
            })
            .unwrap();
    }
}

pub struct MaterialImportOptions {}

impl MaterialImportOptions {
    pub fn schema_name() -> &'static str {
        "MaterialImportOptions"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                // No options
            })
            .unwrap();
    }
}

pub struct MaterialImportedData {}

impl MaterialImportedData {
    pub fn schema_name() -> &'static str {
        "MaterialImportedData"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                //x.add_reference("asset", MaterialImportedData::schema_name());
                //x.add_bytes("image_bytes"); // TODO: this would be a buffer
                //x.add_u32("width");
                //x.add_u32("height");
            })
            .unwrap();
    }
}

#[derive(Serialize, Deserialize)]
struct MaterialProcessedData {

}

#[derive(TypeUuid, Default)]
#[uuid = "e7c83acb-f73b-4b3c-b14d-fe5cc17c0fa3"]
pub struct MaterialImporter;

impl Importer for MaterialImporter {
    fn register_schemas(&self, schema_linker: &mut SchemaLinker) {
        MaterialAsset::register_schema(schema_linker);
        MaterialImportedData::register_schema(schema_linker);
    }

    fn supported_file_extensions(&self) -> &[&'static str] {
        &["blender_material"]
    }

    fn scan_file(&self, path: &Path, schema_set: &SchemaSet) -> Vec<ScannedImportable> {
        let asset_type = schema_set.find_named_type(MaterialAsset::schema_name()).unwrap().as_record().unwrap().clone();
        vec![ScannedImportable {
            name: None,
            asset_type,
            referenced_source_file_paths: Default::default()
        }]
    }

    // fn asset_types(&self) -> &[&'static str] {
    //     &["MaterialAsset"]
    // }

    // fn create_default_import_options(&self, schema_set: &SchemaSet) -> SingleObject {
    //     let options_type = schema_set.find_named_type(MaterialImportedData::schema_name()).unwrap();
    //     SingleObject::new(options_type.as_record().unwrap())
    // }

    fn create_default_asset(&self, editor_model: &mut EditorModel, object_name: ObjectName, object_location: ObjectLocation) -> ObjectId {
        let schema_record = editor_model.root_edit_context_mut().schema_set().find_named_type(MaterialAsset::schema_name()).unwrap().as_record().unwrap().clone();
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
        // // TODO: Replace with a shim so we can track what files are being read
        // // - We trigger the importer for them by specifying the file path and kind of file (i.e. an image, specific type of JSON file, etc.)
        // // - We may need to let the "import" dialog try to perform the import to get error messages and discover what will end up being imported
        // let bytes = std::fs::read(path).unwrap();
        //
        // let decoded_image =
        //     ::image::load_from_memory_with_format(&bytes, ::image::MaterialFormat::Png).unwrap();
        // let (width, height) = decoded_image.dimensions();
        // let image_bytes = decoded_image.into_rgba8().to_vec();

        let material_imported_data_schema = schema
            .find_named_type(MaterialImportedData::schema_name())
            .unwrap()
            .as_record()
            .unwrap();

        let mut import_object = SingleObject::new(material_imported_data_schema);
        // import_object.set_property_override(schema, "image_bytes", Value::Bytes(image_bytes));
        // import_object.set_property_override(schema, "width", Value::U32(width));
        // import_object.set_property_override(schema, "height", Value::U32(height));
        //import_object

        let mut imported_objects = HashMap::default();
        imported_objects.insert(None, import_object);
        imported_objects
    }
}

/*
#[derive(TypeUuid)]
#[uuid = "da6760e7-5b24-43b4-830d-6ee4515096b8"]
pub struct MaterialProcessor {}

impl Processor for MaterialProcessor {
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

        let processed_data = MaterialProcessedData {
            image_bytes,
            width,
            height,
        };

        let serialized = bincode::serialize(&processed_data).unwrap();
        serialized
    }
}
*/