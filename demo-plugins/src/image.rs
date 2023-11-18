pub use super::*;
use ::image::GenericImageView;
use std::path::Path;

use super::generated::{
    GpuImageAssetAccessor, GpuImageAssetOwned, GpuImageAssetReader, GpuImageImportedDataAccessor,
    GpuImageImportedDataOwned, GpuImageImportedDataReader, GpuImageImportedDataWriter,
};
use demo_types::image::*;
use hydrate_data::{RecordBuilder, RecordOwned};
use hydrate_model::pipeline::{ImportContext, ScanContext};
use hydrate_pipeline::{
    job_system, AssetId, BuilderContext, BuilderRegistryBuilder, DataContainerRef,
    DataContainerRefMut, DataSet, EnumerateDependenciesContext, FieldAccessor, HashMap,
    ImportableAsset, ImporterRegistry, ImporterRegistryBuilder, JobApi, JobEnumeratedDependencies,
    JobInput, JobOutput, JobProcessor, JobProcessorRegistryBuilder, PropertyPath, RecordAccessor,
    RunContext, SchemaLinker, SchemaSet, SingleObject,
};
use hydrate_pipeline::{AssetPlugin, Builder};
use hydrate_pipeline::{ImportedImportable, Importer, ScannedImportable};
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
        context: ScanContext,
    ) -> Vec<ScannedImportable> {
        let asset_type = context
            .schema_set
            .find_named_type(GpuImageAssetAccessor::schema_name())
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
        context: ImportContext,
    ) -> HashMap<Option<String>, ImportedImportable> {
        //
        // Read the file
        //
        let decoded_image = ::image::open(context.path).unwrap();

        let (width, height) = decoded_image.dimensions();
        let image_bytes = decoded_image.into_rgba8().to_vec();

        //
        // Create import data
        //
        let import_data = GpuImageImportedDataOwned::new_builder(context.schema_set);
        import_data.image_bytes().set(image_bytes).unwrap();
        import_data.width().set(width).unwrap();
        import_data.height().set(height).unwrap();

        //
        // Create the default asset
        //
        let default_asset = GpuImageAssetOwned::new_builder(context.schema_set);
        default_asset.compress().set(false).unwrap();

        //
        // Return the created assets
        //
        let mut imported_assets = HashMap::default();
        imported_assets.insert(
            None,
            ImportedImportable {
                file_references: Default::default(),
                import_data: Some(import_data.into_inner().unwrap()),
                default_asset: Some(default_asset.into_inner().unwrap()),
            },
        );
        imported_assets
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
        context: EnumerateDependenciesContext<Self::InputT>,
    ) -> JobEnumeratedDependencies {
        // No dependencies
        JobEnumeratedDependencies {
            import_data: vec![context.input.asset_id],
            upstream_jobs: Vec::default(),
        }
    }

    fn run(
        &self,
        context: RunContext<Self::InputT>,
    ) -> GpuImageJobOutput {
        //
        // Read asset properties
        //
        let asset = context
            .asset::<GpuImageAssetReader>(context.input.asset_id)
            .unwrap();
        let compressed = asset.compress().get().unwrap();

        //
        // Read imported data
        //
        let imported_data = context
            .imported_data::<GpuImageImportedDataReader>(context.input.asset_id)
            .unwrap();
        let image_bytes_reader = imported_data.image_bytes();
        let image_bytes = image_bytes_reader.get().unwrap();
        let width = imported_data.width().get().unwrap();
        let height = imported_data.height().get().unwrap();

        //
        // Compress the image, or just return the raw image bytes
        //
        let image_bytes = if compressed {
            profiling::scope!("Compressing Image");
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
            image_bytes.clone()
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
        context.produce_default_artifact(context.input.asset_id, processed_data);

        GpuImageJobOutput {}
    }
}

#[derive(TypeUuid, Default)]
#[uuid = "da6760e7-5b24-43b4-830d-6ee4515096b8"]
pub struct GpuImageBuilder {}

impl Builder for GpuImageBuilder {
    fn asset_type(&self) -> &'static str {
        GpuImageAssetAccessor::schema_name()
    }

    fn start_jobs(
        &self,
        context: BuilderContext,
    ) {
        let data_container =
            DataContainerRef::from_dataset(context.data_set, context.schema_set, context.asset_id);
        let x = GpuImageAssetAccessor::default();
        let compressed = x.compress().get(data_container).unwrap();

        //Future: Might produce jobs per-platform
        context.enqueue_job::<GpuImageJobProcessor>(
            context.data_set,
            context.schema_set,
            context.job_api,
            GpuImageJobInput {
                asset_id: context.asset_id,
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
