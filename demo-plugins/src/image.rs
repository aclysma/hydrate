pub use super::*;
use ::image::GenericImageView;
use std::path::Path;

use super::generated::{GpuImageAssetRecord, GpuImageImportedDataRecord};
use demo_types::image::*;
use hydrate_model::pipeline::{AssetPlugin, Builder};
use hydrate_model::pipeline::{ImportedImportable, Importer, ScannedImportable};
use hydrate_model::{
    job_system, BuilderRegistryBuilder, DataContainer, DataContainerMut,
    DataSet, Field, HashMap, ImportableObject, ImporterRegistry, ImporterRegistryBuilder, JobApi,
    JobEnumeratedDependencies, JobInput, JobOutput, JobProcessor,
    JobProcessorRegistryBuilder, AssetId, PropertyPath, Record, SchemaLinker, SchemaSet,
    SingleObject,
};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;

#[derive(TypeUuid, Default)]
#[uuid = "e7c83acb-f73b-4b3c-b14d-fe5cc17c0fa3"]
pub struct GpuImageImporter;

impl Importer for GpuImageImporter {
    fn supported_file_extensions(&self) -> &[&'static str] {
        &["png", "jpg", "tif"]
    }

    fn scan_file(
        &self,
        _path: &Path,
        schema_set: &SchemaSet,
        _importer_registry: &ImporterRegistry,
    ) -> Vec<ScannedImportable> {
        let asset_type = schema_set
            .find_named_type(GpuImageAssetRecord::schema_name())
            .unwrap()
            .as_record()
            .unwrap()
            .clone();
        vec![ScannedImportable {
            name: None,
            asset_type,
            file_references: Default::default(),
        }]
    }

    fn import_file(
        &self,
        path: &Path,
        importable_objects: &HashMap<Option<String>, ImportableObject>,
        schema_set: &SchemaSet,
    ) -> HashMap<Option<String>, ImportedImportable> {
        //
        // Read the file
        //
        let decoded_image = ::image::open(path).unwrap();

        let (width, height) = decoded_image.dimensions();
        let image_bytes = decoded_image.into_rgba8().to_vec();

        //
        // Create import data
        //
        let import_data = {
            let mut import_object =
                GpuImageImportedDataRecord::new_single_object(schema_set).unwrap();
            let mut import_data_container =
                DataContainerMut::new_single_object(&mut import_object, schema_set);
            let x = GpuImageImportedDataRecord::default();
            x.image_bytes()
                .set(&mut import_data_container, image_bytes)
                .unwrap();
            x.width().set(&mut import_data_container, width).unwrap();
            x.height().set(&mut import_data_container, height).unwrap();
            import_object
        };

        //
        // Create the default asset
        //
        let default_asset = {
            let mut default_asset_object =
                GpuImageAssetRecord::new_single_object(schema_set).unwrap();
            let mut default_asset_data_container =
                DataContainerMut::new_single_object(&mut default_asset_object, schema_set);
            let x = GpuImageAssetRecord::default();
            x.compress()
                .set(&mut default_asset_data_container, false)
                .unwrap();
            default_asset_object
        };

        //
        // Return the created objects
        //
        let mut imported_objects = HashMap::default();
        imported_objects.insert(
            None,
            ImportedImportable {
                file_references: Default::default(),
                import_data: Some(import_data),
                default_asset: Some(default_asset),
            },
        );
        imported_objects
    }
}

#[derive(Hash, Serialize, Deserialize)]
pub struct GpuImageJobInput {
    pub asset_id: AssetId,
    pub compressed: bool,
}
impl JobInput for GpuImageJobInput {}

#[derive(Serialize, Deserialize)]
pub struct GpuImageJobOutput {}
impl JobOutput for GpuImageJobOutput {}

#[derive(Default, TypeUuid)]
#[uuid = "5311c92e-470e-4fdc-88cd-3abaf1c28f39"]
pub struct GpuImageJobProcessor;

impl JobProcessor for GpuImageJobProcessor {
    type InputT = GpuImageJobInput;
    type OutputT = GpuImageJobOutput;

    fn version(&self) -> u32 {
        1
    }

    fn enumerate_dependencies(
        &self,
        input: &GpuImageJobInput,
        _data_set: &DataSet,
        _schema_set: &SchemaSet,
    ) -> JobEnumeratedDependencies {
        // No dependencies
        JobEnumeratedDependencies {
            import_data: vec![input.asset_id],
            upstream_jobs: Vec::default(),
        }
    }

    fn run(
        &self,
        input: &GpuImageJobInput,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        dependency_data: &HashMap<AssetId, SingleObject>,
        job_api: &dyn JobApi,
    ) -> GpuImageJobOutput {
        //
        // Read asset properties
        //
        let data_container = DataContainer::new_dataset(data_set, schema_set, input.asset_id);
        let x = GpuImageAssetRecord::default();
        let compressed = x.compress().get(&data_container).unwrap();

        //
        // Read imported data
        //
        let imported_data = &dependency_data[&input.asset_id];
        let data_container = DataContainer::new_single_object(&imported_data, schema_set);
        let x = GpuImageImportedDataRecord::new(PropertyPath::default());

        let image_bytes = x.image_bytes().get(&data_container).unwrap().clone();
        let width = x.width().get(&data_container).unwrap();
        let height = x.height().get(&data_container).unwrap();

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
            log::debug!("Not compressing texture");
            image_bytes
        };

        //
        // Create the processed data
        //
        let processed_data = GpuImageAssetData {
            image_bytes,
            width,
            height,
        };

        //
        // Serialize and return
        //
        job_system::produce_asset(job_api, input.asset_id, processed_data);

        GpuImageJobOutput {}
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "da6760e7-5b24-43b4-830d-6ee4515096b8"]
pub struct GpuImageBuilder {}

impl Builder for GpuImageBuilder {
    fn asset_type(&self) -> &'static str {
        GpuImageAssetRecord::schema_name()
    }

    fn start_jobs(
        &self,
        asset_id: AssetId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        job_api: &dyn JobApi,
    ) {
        let data_container = DataContainer::new_dataset(data_set, schema_set, asset_id);
        let x = GpuImageAssetRecord::default();
        let compressed = x.compress().get(&data_container).unwrap();

        //Future: Might produce jobs per-platform
        job_system::enqueue_job::<GpuImageJobProcessor>(
            data_set,
            schema_set,
            job_api,
            GpuImageJobInput {
                asset_id,
                compressed,
            },
        );
    }
}

pub struct GpuImageAssetPlugin;

impl AssetPlugin for GpuImageAssetPlugin {
    fn setup(
        _schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistryBuilder,
        builder_registry: &mut BuilderRegistryBuilder,
        job_processor_registry: &mut JobProcessorRegistryBuilder,
    ) {
        importer_registry.register_handler::<GpuImageImporter>();
        builder_registry.register_handler::<GpuImageBuilder>();
        job_processor_registry.register_job_processor::<GpuImageJobProcessor>();
    }
}
