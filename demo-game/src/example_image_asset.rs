use demo_types::image::GpuImageAssetData;
use hydrate::base::handle::{LoaderInfoProvider, RefOp, SerdeContext};
use hydrate::base::LoadHandle;
use hydrate::loader::asset_storage::UpdateAssetResult;
use hydrate::loader::DynAssetLoader;
use std::error::Error;
use type_uuid::TypeUuid;

// No real significance to this UUID, other than all assets should have a unique UUID
#[derive(TypeUuid, Clone)]
#[uuid = "3ebc8afd-09d2-427e-b9e9-50a53fcbde84"]
pub struct GpuImageAsset {
    pub image_bytes: Vec<u8>,
    pub _width: u32,
    pub _height: u32,
}

// This is an example asset loader, allowing for custom operations to prepare the asset for use
pub struct GpuImageLoader;

impl DynAssetLoader<GpuImageAsset> for GpuImageLoader {
    fn update_asset(
        &mut self,
        refop_sender: &crossbeam_channel::Sender<RefOp>,
        loader_info: &dyn LoaderInfoProvider,
        data: &[u8],
        _load_handle: LoadHandle,
        load_op: hydrate::loader::storage::AssetLoadOp,
    ) -> Result<
        hydrate::loader::asset_storage::UpdateAssetResult<GpuImageAsset>,
        Box<dyn Error + Send + 'static>,
    > {
        log::debug!("GpuImageLoader update_asset");

        let asset_data = SerdeContext::with(loader_info, refop_sender.clone(), || {
            log::debug!("bincode deserialize");
            let x = bincode::deserialize::<GpuImageAssetData>(data)
                // Coerce into boxed error
                .map_err(|x| -> Box<dyn Error + Send + 'static> { Box::new(x) });
            println!("finished deserialize");
            x
        })?;
        log::debug!("call load_op.complete()");

        load_op.complete();
        log::debug!("return");
        Ok(UpdateAssetResult::Result(GpuImageAsset {
            image_bytes: asset_data.image_bytes,
            _width: asset_data.width,
            _height: asset_data.height,
        }))
    }

    fn commit_asset_version(
        &mut self,
        _handle: LoadHandle,
    ) {
    }

    fn free(
        &mut self,
        _handle: LoadHandle,
    ) {
    }
}