use ::image::{EncodableLayout, GenericImageView};
use nexdb::{
    DataSet, ObjectId, ObjectLocation, ObjectName, Schema, SchemaLinker, SchemaSet, Value,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

mod image;

trait Importer {
    fn import_file(
        &self,
        path: &Path,
        data_set: &mut DataSet,
        schema: &SchemaSet,
    );
}

trait Processor {
    fn process_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
    ) -> Vec<u8>;
}

struct ImageAsset {}

impl ImageAsset {
    pub fn schema_name() -> &'static str {
        "ImageAsset"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                x.add_reference("imported_data", ImageImportedData::schema_name());
                //x.add_string("path");
                x.add_boolean("compress");
            })
            .unwrap();
    }
}

struct ImageImportedData {}

impl ImageImportedData {
    pub fn schema_name() -> &'static str {
        "ImageImportedData"
    }

    pub fn register_schema(linker: &mut SchemaLinker) {
        linker
            .register_record_type(Self::schema_name(), |x| {
                x.add_reference("asset", ImageAsset::schema_name());
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

struct ImageImporter {}

impl Importer for ImageImporter {
    fn import_file(
        &self,
        path: &Path,
        data_set: &mut DataSet,
        schema: &SchemaSet,
    ) {
        // TODO: Replace with a shim so we can track what files are being read
        // - We trigger the importer for them by specifying the file path and kind of file (i.e. an image, specific type of JSON file, etc.)
        // - We may need to let the "import" dialog try to perform the import to get error messages and discover what will end up being imported
        let bytes = std::fs::read(path).unwrap();

        let decoded_image =
            ::image::load_from_memory_with_format(&bytes, ::image::ImageFormat::Png).unwrap();
        let (width, height) = decoded_image.dimensions();
        let image_bytes = decoded_image.into_rgba8().to_vec();

        let asset_id = Uuid::new_v4();
        let import_id = Uuid::new_v4();

        let image_asset_schema = schema
            .find_named_type(ImageAsset::schema_name())
            .unwrap()
            .as_record()
            .unwrap();
        let image_imported_data_schema = schema
            .find_named_type(ImageImportedData::schema_name())
            .unwrap()
            .as_record()
            .unwrap();

        let asset = data_set.new_object(
            ObjectName::new(path.file_name().unwrap().to_string_lossy().to_string()),
            ObjectLocation::null(),
            image_asset_schema,
        );
        let imported_data = data_set.new_object(
            ObjectName::empty(),
            ObjectLocation::null(),
            image_imported_data_schema,
        );

        data_set.set_property_override(schema, asset, "compress", Value::Boolean(true));
        data_set.set_property_override(
            schema,
            asset,
            "imported_data",
            Value::ObjectRef(ObjectId(import_id.as_u128())),
        );
        data_set.set_property_override(schema, imported_data, "data", Value::Bytes(image_bytes));
        data_set.set_property_override(
            schema,
            imported_data,
            "asset",
            Value::ObjectRef(ObjectId(asset_id.as_u128())),
        );
    }
}

struct ImageProcessor {}

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
