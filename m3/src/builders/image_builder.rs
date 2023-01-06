use nexdb::{DataSet, ObjectId, SchemaSet};
use crate::builders::Builder;

pub use super::*;


#[derive(Serialize, Deserialize)]
struct ImageBuiltData {
    image_bytes: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(TypeUuid, Default)]
#[uuid = "da6760e7-5b24-43b4-830d-6ee4515096b8"]
pub struct ImageBuilder {}

impl Builder for ImageBuilder {

    fn register_schemas(&self, schema_linker: &mut SchemaLinker) {

    }

    fn asset_type(&self) -> &'static str {
        ImageAsset::schema_name()
    }

    fn build_asset(&self, asset_id: ObjectId, data_set: &DataSet, schema: &SchemaSet) -> Vec<u8> {
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

        let processed_data = ImageBuiltData {
            image_bytes,
            width,
            height,
        };

        let serialized = bincode::serialize(&processed_data).unwrap();
        serialized
    }
}
