use hydrate_model::{DataSet, DataSetEntry, HashMap, ObjectId, SchemaLinker, SchemaSet, SingleObject};
use hydrate_pipeline::{AssetPlugin, Builder, BuilderRegistry, ImporterRegistry};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use demo_types::simple_data::*;

pub struct SimpleBincodeDataBuilder<T: DataSetEntry + Sized + Serialize + for<'a> Deserialize<'a>> {
    asset_type: &'static str,
    phantom_data: PhantomData<T>,
}

impl<T: DataSetEntry + Sized + Serialize + for<'a> Deserialize<'a>> SimpleBincodeDataBuilder<T> {
    pub fn new(asset_type: &'static str) -> Self {
        SimpleBincodeDataBuilder {
            asset_type,
            phantom_data: PhantomData::default()
        }
    }
}

impl<T: DataSetEntry + Sized + Serialize + for<'a> Deserialize<'a>> Builder for SimpleBincodeDataBuilder<T> {
    fn asset_type(&self) -> &'static str {
        self.asset_type
    }

    fn build_dependencies(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
    ) -> Vec<ObjectId> {
        vec![asset_id]
    }

    fn build_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
        _dependency_data: &HashMap<ObjectId, SingleObject>,
    ) -> Vec<u8> {
        let data = T::from_data_set(asset_id, data_set, schema);

        let serialized = bincode::serialize(&data).unwrap();
        serialized
    }
}

pub struct SimpleDataAssetPlugin;

impl AssetPlugin for SimpleDataAssetPlugin {
    fn setup(
        schema_linker: &mut SchemaLinker,
        importer_registry: &mut ImporterRegistry,
        builder_registry: &mut BuilderRegistry,
    ) {
        builder_registry.register_handler_instance(schema_linker, SimpleBincodeDataBuilder::<AllFields>::new("AllFields"));
        builder_registry.register_handler_instance(schema_linker, SimpleBincodeDataBuilder::<Transform>::new("Transform"));
    }
}
