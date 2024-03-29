pub use super::*;
use ::image::GenericImageView;
use std::sync::Arc;

use super::generated::{GpuImageAssetRecord, GpuImageImportedDataRecord};
use ::image::Rgba;
use demo_types::image::*;
use hydrate_data::Record;
use hydrate_model::pipeline::{ImportContext, ScanContext};
use hydrate_pipeline::{
    AssetId, BuilderContext, JobInput, JobOutput, JobProcessor, PipelineResult, RunContext,
};
use hydrate_pipeline::{AssetPlugin, Builder};
use hydrate_pipeline::{
    AssetPluginSetupContext, Importer, ThumbnailImage, ThumbnailProvider,
    ThumbnailProviderGatherContext, ThumbnailProviderRenderContext,
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
        context: ScanContext,
    ) -> PipelineResult<()> {
        context.add_default_importable::<GpuImageAssetRecord>()?;
        Ok(())
    }

    fn import_file(
        &self,
        context: ImportContext,
    ) -> PipelineResult<()> {
        //
        // Read the file
        //
        let decoded_image = ::image::open(context.path).map_err(|x| x.to_string())?;

        let (width, height) = decoded_image.dimensions();
        let image_bytes = decoded_image.into_rgba8().to_vec();

        //
        // Create import data
        //
        let import_data = GpuImageImportedDataRecord::new_builder(context.schema_set);
        import_data.image_bytes().set(image_bytes)?;
        import_data.width().set(width)?;
        import_data.height().set(height)?;

        //
        // Create the default asset
        //
        let default_asset = GpuImageAssetRecord::new_builder(context.schema_set);
        default_asset.compress().set(false)?;

        //
        // Return the created assets
        //
        context
            .add_default_importable(default_asset.into_inner()?, Some(import_data.into_inner()?));
        Ok(())
    }
}

#[derive(Hash, Serialize, Deserialize)]
pub struct GpuImageJobInput {
    pub asset_id: AssetId,
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

    fn run<'a>(
        &'a self,
        context: &'a RunContext<'a, Self::InputT>,
    ) -> PipelineResult<GpuImageJobOutput> {
        //
        // Read asset properties
        //
        let asset = context.asset::<GpuImageAssetRecord>(context.input.asset_id)?;
        let compressed = asset.compress().get()?;

        //
        // Read imported data
        //
        let imported_data =
            context.imported_data::<GpuImageImportedDataRecord>(context.input.asset_id)?;
        let image_bytes_reader = imported_data.image_bytes();
        let image_bytes = image_bytes_reader.get()?;
        let width = imported_data.width().get()?;
        let height = imported_data.height().get()?;

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
                compressor
                    .process()
                    .map_err(|e| format!("Compressor process() failed {:?}", e))?;
                log::debug!("Compressed texture");
            }
            let compressed_basis_data = Arc::new(compressor.basis_file().to_vec());
            compressed_basis_data
        } else {
            //log::debug!("Not compressing texture");
            (*image_bytes).clone()
        };

        //
        // Create the processed data
        //
        let processed_data = GpuImageAssetData {
            image_bytes: (*image_bytes).clone(),
            width,
            height,
        };

        //
        // Serialize and return
        //
        context.produce_default_artifact(context.input.asset_id, processed_data)?;

        Ok(GpuImageJobOutput {})
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
        context: BuilderContext,
    ) -> PipelineResult<()> {
        //Future: Might produce jobs per-platform
        context.enqueue_job::<GpuImageJobProcessor>(
            context.data_set,
            context.schema_set,
            context.job_api,
            GpuImageJobInput {
                asset_id: context.asset_id,
            },
        )?;

        Ok(())
    }
}

#[derive(Default)]
pub struct GpuImageThumbnailProvider {}

impl ThumbnailProvider for GpuImageThumbnailProvider {
    type GatheredDataT = ();

    fn asset_type(&self) -> &'static str {
        GpuImageAssetRecord::schema_name()
    }

    fn version(&self) -> u32 {
        1
    }

    fn gather(
        &self,
        context: ThumbnailProviderGatherContext,
    ) -> Self::GatheredDataT {
        context.add_import_data_dependency(context.asset_id);
    }

    fn render<'a>(
        &'a self,
        context: &'a ThumbnailProviderRenderContext<'a>,
        _gathered_data: Self::GatheredDataT,
    ) -> PipelineResult<ThumbnailImage> {
        let import_data = context.imported_data::<GpuImageImportedDataRecord>(context.asset_id)?;
        let width = import_data.width().get()?;
        let height = import_data.height().get()?;
        let image_bytes = import_data.image_bytes().get()?.clone();

        let image = ::image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_vec(
            width,
            height,
            (*image_bytes).clone(),
        )
        .unwrap();

        let resize_ratio_x = width as f32 / context.desired_thumbnail_width as f32;
        let resize_ratio_y = height as f32 / context.desired_thumbnail_height as f32;

        let resize_ratio = resize_ratio_x.max(resize_ratio_y);
        let new_size_x = ((width as f32 / resize_ratio).round() as u32).max(1);
        let new_size_y = ((height as f32 / resize_ratio).round() as u32).max(1);

        let resized_image = ::image::imageops::resize(
            &image,
            new_size_x,
            new_size_y,
            ::image::imageops::FilterType::Lanczos3,
        );
        Ok(ThumbnailImage {
            width: resized_image.width(),
            height: resized_image.height(),
            pixel_data: resized_image.into_raw().clone(),
        })
    }
}

pub struct GpuImageAssetPlugin;

impl AssetPlugin for GpuImageAssetPlugin {
    fn setup(context: AssetPluginSetupContext) {
        context
            .importer_registry
            .register_handler::<GpuImageImporter>();
        context
            .builder_registry
            .register_handler::<GpuImageBuilder>();
        context
            .job_processor_registry
            .register_job_processor::<GpuImageJobProcessor>();
        context
            .thumbnail_provider_registry
            .register_thumbnail_provider::<GpuImageThumbnailProvider>();
    }
}
