use demo_types::simple_data::*;
use hydrate_base::{AssetUuid, BuiltObjectMetadata};
use hydrate_model::{BuilderRegistryBuilder, DataContainer, DataSet, DataSetView, HashMap, ImporterRegistryBuilder, ObjectId, SchemaLinker, SchemaSet, SingleObject};
use hydrate_model::pipeline::{AssetPlugin, Builder, BuilderRegistry, BuiltAsset, ImporterRegistry};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use type_uuid::TypeUuid;
use crate::generated::{AllFieldsRecord, TransformRecord};

use super::SimpleData;

//
// Implement SimpleBincodeDataBuilder for all SimpleData
//
pub struct SimpleBincodeDataBuilder<T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid> {
    asset_type: &'static str,
    phantom_data: PhantomData<T>,
}

impl<T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid> SimpleBincodeDataBuilder<T> {
    pub fn new(asset_type: &'static str) -> Self {
        SimpleBincodeDataBuilder {
            asset_type,
            phantom_data: PhantomData::default(),
        }
    }
}

impl<T: SimpleData + Sized + Serialize + for<'a> Deserialize<'a> + TypeUuid> Builder
for SimpleBincodeDataBuilder<T>
{
    fn asset_type(&self) -> &'static str {
        self.asset_type
    }

    fn enumerate_dependencies(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
    ) -> Vec<ObjectId> {
        vec![asset_id]
    }

    fn build_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema_set: &SchemaSet,
        _dependency_data: &HashMap<ObjectId, SingleObject>,
    ) -> BuiltAsset {
        let mut data_set_view = DataContainer::new_dataset(&data_set, schema_set, asset_id);
        let data = T::from_data_container(&mut data_set_view);

        let serialized = bincode::serialize(&data).unwrap();
        BuiltAsset {
            metadata: BuiltObjectMetadata {
                dependencies: vec![],
                subresource_count: 0,
                asset_type: uuid::Uuid::from_bytes(T::UUID)
            },
            data: serialized
        }
    }
}

