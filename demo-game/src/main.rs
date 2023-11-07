use demo_types::gpu_buffer::GpuBufferBuiltData;
use demo_types::image::GpuImageAssetData;
use demo_types::mesh_adv::{
    MeshAdvBufferAssetData, MeshAdvMaterialAssetData, MeshAdvMaterialData, MeshAdvMeshAssetData,
};
use demo_types::simple_data::{Transform, TransformRef};
use hydrate::base::handle::{LoaderInfoProvider, RefOp, SerdeContext};
use hydrate::base::{ArtifactId, LoadHandle};
use hydrate::loader::asset_storage::UpdateAssetResult;
use hydrate::loader::{DynAssetLoader, Handle};
use std::error::Error;
use std::path::PathBuf;
use type_uuid::TypeUuid;

pub fn build_data_source_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../demo-editor/data/build_data"
    ))
}

#[derive(TypeUuid, Clone)]
#[uuid = "3ebc8afd-09d2-427e-b9e9-50a53fcbde84"]
struct GpuImageAsset {
    pub image_bytes: Vec<u8>,
    pub _width: u32,
    pub _height: u32,
}

struct GpuImageLoader;

impl DynAssetLoader<GpuImageAsset> for GpuImageLoader {
    fn update_asset(
        &mut self,
        refop_sender: &crossbeam_channel::Sender<RefOp>,
        loader_info: &dyn LoaderInfoProvider,
        data: &[u8],
        _load_handle: LoadHandle,
        load_op: hydrate::loader::storage::AssetLoadOp,
        _version: u32,
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
        _version: u32,
    ) {
    }

    fn free(
        &mut self,
        _handle: LoadHandle,
    ) {
    }
}

fn main() {
    // Setup logging
    env_logger::Builder::default()
        .write_style(env_logger::WriteStyle::Always)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut loader = hydrate::loader::AssetManager::new(build_data_source_path()).unwrap();
    loader.add_storage_with_loader::<GpuImageAssetData, GpuImageAsset, GpuImageLoader>(Box::new(
        GpuImageLoader,
    ));
    loader.add_storage::<GpuBufferBuiltData>();
    loader.add_storage::<Transform>();
    loader.add_storage::<TransformRef>();
    loader.add_storage::<MeshAdvMeshAssetData>();
    loader.add_storage::<MeshAdvBufferAssetData>();
    loader.add_storage::<MeshAdvMaterialAssetData>();
    loader.add_storage::<MeshAdvMaterialData>();

    let load_handle_transform_ref: Handle<TransformRef> =
        loader.load_asset_path("db:/path_file_system/test");

    // let load_handle_transform_ref: Handle<TransformRef> = loader.load_asset(ArtifactId(
    //     uuid::Uuid::parse_str("798bd93be6d14f459d31d7e689c28c03")
    //         .unwrap()
    //         .as_u128(),
    // ));

    let load_handle_mesh: Handle<MeshAdvMeshAssetData> = loader.load_asset(ArtifactId(
        uuid::Uuid::parse_str("522aaf98-5dc3-4578-a4cc-411ca6c0a826")
            .unwrap()
            .as_u128(),
    ));

    let load_handle_image: Handle<GpuImageAsset> = loader.load_asset(ArtifactId(
        uuid::Uuid::parse_str("fe9a9f83-a7c1-4a00-b61d-17a1dca43717")
            .unwrap()
            .as_u128(),
    ));

    loop {
        std::thread::sleep(std::time::Duration::from_millis(15));
        loader.update();

        let data = load_handle_transform_ref.asset(loader.storage());
        if let Some(data) = data {
            let data_inner = data.transform.asset(loader.storage());
            println!("load_handle_transform_ref loaded {:?}", data);
            println!("load_handle_transform_ref inner loaded {:?}", data_inner);
        } else {
            println!("load_handle_transform_ref not loaded");
        }

        let data = load_handle_mesh.asset(loader.storage());
        if let Some(data) = data {
            let data_full_vb = data
                .vertex_position_buffer
                .as_ref()
                .map(|x| x.asset(loader.storage()).unwrap());
            let data_position_vb = data
                .vertex_position_buffer
                .as_ref()
                .map(|x| x.asset(loader.storage()).unwrap());
            println!("load_handle_mesh loaded {:?}", data.mesh_parts);
            if let Some(data_full_vb) = data_full_vb {
                println!("full vb {:?}", data_full_vb.data.len());
            }

            if let Some(data_position_vb) = data_position_vb {
                println!("position vb {:?}", data_position_vb.data.len());
            }
        } else {
            println!("load_handle_mesh not loaded");
        }

        let data = load_handle_image.asset(loader.storage());
        if let Some(data) = data {
            println!("load_handle_image loaded {:?}", data.image_bytes.len());
        } else {
            println!("load_handle_image not loaded");
        }
    }
}
