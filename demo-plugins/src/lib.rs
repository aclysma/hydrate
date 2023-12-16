mod image;

use std::sync::Arc;
pub use self::image::*;

mod blender_material;
pub use blender_material::*;

mod blender_mesh;
pub use blender_mesh::*;

mod glsl;
pub use glsl::*;

mod simple_data_types;
pub use simple_data_types::*;

mod gltf;
pub use crate::gltf::*;

mod gpu_buffer;
pub use gpu_buffer::*;

mod mesh_adv;
pub use mesh_adv::*;

pub mod generated_wrapper;
pub use generated_wrapper as generated;
use hydrate_pipeline::ThumbnailImage;

mod push_buffer;

mod mesh_util;

mod example_tasks;

fn create_thumbnail_image_from_bytes(bytes: &[u8], image_format: ::image::ImageFormat) -> Arc<ThumbnailImage> {
    let image = ::image::load_from_memory_with_format(bytes, image_format).unwrap();
    use ::image::GenericImageView;
    let width = image.width();
    let height = image.height();
    Arc::new(ThumbnailImage {
        width,
        height,
        pixel_data: image.into_rgba8().into_raw()
    })
}