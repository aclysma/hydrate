use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;
use type_uuid::TypeUuid;
use hydrate_base::AssetId;
use hydrate_data::{DataSet, SchemaSet};
use hydrate_schema::HashSet;
use crate::PipelineResult;
use crate::thumbnails::thumbnail_system::ThumbnailImage;

pub struct ThumbnailProviderGatherContext<'a> {
    pub asset_id: AssetId,
    pub data_set: &'a DataSet,
    pub schema_set: &'a SchemaSet,
    pub(crate) import_data_dependencies: &'a Rc<RefCell<&'a mut HashSet<AssetId>>>,
}

impl<'a> ThumbnailProviderGatherContext<'a> {
    pub fn add_import_data_dependency(&self, asset_id: AssetId) {
        self.import_data_dependencies.borrow_mut().insert(asset_id);
    }
}

pub struct ThumbnailProviderRenderContext {
    pub asset_id: AssetId,

}

impl ThumbnailProviderRenderContext {

}


pub trait ThumbnailProvider {
    type GatheredDataT: Hash + 'static;

    fn asset_type(&self) -> &'static str;

    fn version(&self) -> u32;

    // Is given access to all data needed to produce the thumbnail. This call acts as the logic
    // that decides if a thumbnail needs to be refreshed.
    fn gather(&self, context: ThumbnailProviderGatherContext) -> Self::GatheredDataT;

    // Can only read gathered data provided by gather(), import data, and build data. The import/build
    // data has to match the hash of what was requested by gather()
    fn render(&self, context: ThumbnailProviderRenderContext, gathered_data: Self::GatheredDataT) -> PipelineResult<ThumbnailImage>;  // return something?
}
