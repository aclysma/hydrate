use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use nexdb::{DataSet, HashMap, ObjectId, SchemaLinker, SchemaSet, SingleObject};
use pipeline::{AssetPlugin, Builder, BuilderRegistry, ImporterRegistry};

pub trait SimpleData: Sized + Serialize + for<'a> Deserialize<'a> {
    fn schema_name() -> &'static str;

    fn from_data_set(
        asset_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
    ) -> Self;
}


pub struct SimpleDataBuilder<T: SimpleData> {
    phantom_data: PhantomData<T>
}

impl<T: SimpleData> Default for SimpleDataBuilder<T> {
    fn default() -> Self {
        SimpleDataBuilder {
            phantom_data: PhantomData::default()
        }
    }
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


#[derive(Serialize, Deserialize)]
struct Transform {
    position: [f32; 3],
    rotation: [f32; 4],
    scale: [f32; 3],
}

impl<'a> SimpleData for Transform {
    fn schema_name() -> &'static str {
        "Transform"
    }

    fn from_data_set(asset_id: ObjectId, data_set: &DataSet, schema: &SchemaSet) -> Self {
        let position = [
            data_set.resolve_property(schema, asset_id, "position.x").unwrap().as_f32().unwrap(),
            data_set.resolve_property(schema, asset_id, "position.y").unwrap().as_f32().unwrap(),
            data_set.resolve_property(schema, asset_id, "position.z").unwrap().as_f32().unwrap()
        ];

        let rotation = [
            data_set.resolve_property(schema, asset_id, "rotation.x").unwrap().as_f32().unwrap(),
            data_set.resolve_property(schema, asset_id, "rotation.y").unwrap().as_f32().unwrap(),
            data_set.resolve_property(schema, asset_id, "rotation.z").unwrap().as_f32().unwrap(),
            data_set.resolve_property(schema, asset_id, "rotation.w").unwrap().as_f32().unwrap()
        ];

        let scale = [
            data_set.resolve_property(schema, asset_id, "scale.x").unwrap().as_f32().unwrap(),
            data_set.resolve_property(schema, asset_id, "scale.y").unwrap().as_f32().unwrap(),
            data_set.resolve_property(schema, asset_id, "scale.z").unwrap().as_f32().unwrap()
        ];

        Transform {
            position,
            rotation,
            scale,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AllFields {
    boolean: bool,
    int32: i32,
    int64: i64
}

impl<'a> SimpleData for AllFields {
    fn schema_name() -> &'static str {
        "AllFields"
    }

    fn from_data_set(asset_id: ObjectId, data_set: &DataSet, schema: &SchemaSet) -> Self {
        let boolean = data_set.resolve_property(schema, asset_id, "boolean").unwrap().as_boolean().unwrap();
        let int32 = data_set.resolve_property(schema, asset_id, "int32").unwrap().as_i32().unwrap();
        let int64 = data_set.resolve_property(schema, asset_id, "int64").unwrap().as_i64().unwrap();

        AllFields {
            boolean,
            int32,
            int64
        }
    }
}

pub struct SimpleDataAssetPlugin;

impl AssetPlugin for SimpleDataAssetPlugin {
    fn setup(schema_linker: &mut SchemaLinker, importer_registry: &mut ImporterRegistry, builder_registry: &mut BuilderRegistry) {
        builder_registry.register_handler::<SimpleDataBuilder<AllFields>>(schema_linker);
        builder_registry.register_handler::<SimpleDataBuilder<Transform>>(schema_linker);
    }
}
