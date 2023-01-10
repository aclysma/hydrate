use std::marker::PhantomData;
use nexdb::{DataSet, HashMap, ObjectId, SchemaSet, SingleObject};
use crate::pipeline::Builder;

trait SimpleData: Sized {
    fn schema_name() -> &'static str;
    fn from_data_set(
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
    ) -> Self;
}


#[derive(Default)]
pub struct SimpleDataBuilder<T: SimpleData> {
    phantom_data: PhantomData<T>
}

impl<T: SimpleData> Builder for SimpleDataBuilder<T> {
    fn asset_type(&self) -> &'static str {
        T::schema_name()
    }

    fn dependencies(&self, asset_id: ObjectId, data_set: &DataSet, schema: &SchemaSet) -> Vec<ObjectId> {
        vec![asset_id]
    }

    fn build_asset(
        &self,
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
        _dependency_data: &HashMap<ObjectId, SingleObject>
    ) -> Vec<u8> {
        let data = T::from_data_set(asset_id, data_set, schema);

        let serialized = bincode::serialize(&data).unwrap();
        serialized
    }
}
