use hydrate_data::{DataContainer, DataSet, DataSetEntry, DataSetResult, DataSetView, DataSetViewMut, ObjectId, SchemaSet, Value};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use hydrate_base::{AssetUuid, Handle};
use super::simple_data_gen_from_schema::*;

//
// Hand-implemented helper code for the schema -> engine data conversion
//
trait Load<T> {
    fn load(&self, data_set_view: &DataContainer) -> DataSetResult<T>;
}

impl Load<[f32; 3]> for Vec3Record {
    fn load(&self, data_container: &DataContainer) -> DataSetResult<[f32; 3]> {
        Ok([self.x().get(data_container)?, self.y().get(data_container)?, self.z().get(data_container)?])
    }
}

impl Load<[f32; 4]> for Vec4Record {
    fn load(&self, data_container: &DataContainer) -> DataSetResult<[f32; 4]> {
        Ok([self.x().get(data_container)?, self.y().get(data_container)?, self.z().get(data_container)?, self.w().get(data_container)?])
    }
}


// impl Into<[f32; 3]> for Vec3FromSchema {
//     fn into(self) -> [f32; 3] {
//         [self.x, self.y, self.z]
//     }
// }
// impl Into<[f32; 4]> for Vec4FromSchema {
//     fn into(self) -> [f32; 4] {
//         [self.x, self.y, self.z, self.w]
//     }
// }

//
// Engine-specific types start here
//

#[derive(Serialize, Deserialize, TypeUuid, Debug)]
#[uuid = "7132d33e-9bbc-4fb1-b857-17962afd44b8"]
pub struct TransformRef {
    pub transform: Handle<Transform>
}

impl DataSetEntry for TransformRef {
    fn from_data_set(
        data_set_view: &DataContainer,
    ) -> Self {
        let object_id = data_set_view.resolve_property("transform").unwrap().as_object_ref().unwrap();

        let asset_id = AssetUuid(*object_id.as_uuid().as_bytes());

        //TODO: Verify type?
        let handle = hydrate_base::handle::make_handle::<Transform>(asset_id);

        TransformRef {
            transform: handle
        }
    }
}

#[derive(Serialize, Deserialize, TypeUuid, Debug)]
#[uuid = "da334afa-7af9-4894-8b7e-29defe202e90"]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl DataSetEntry for Transform {
    fn from_data_set(
        data_container: &DataContainer,
    ) -> Self {
        let transform = TransformRecord::default();

        Transform {
            position: transform.position().load(data_container).unwrap(),
            rotation: transform.rotation().load(data_container).unwrap(),
            scale: transform.scale().load(data_container).unwrap(),
        }


        // data_set_view.push_property_path("position");
        // let position = Vec3FromSchema::load(data_set_view);
        // data_set_view.pop_property_path();
        //
        // data_set_view.push_property_path("rotation");
        // let rotation = Vec4FromSchema::load(data_set_view);
        // data_set_view.pop_property_path();
        //
        // data_set_view.push_property_path("scale");
        // let scale = Vec3FromSchema::load(data_set_view);
        // data_set_view.pop_property_path();

        //let test_field = data_set.resolve_property(schema, object_id, "test_ref").unwrap().as_object_ref().unwrap();
        // Create handle passing the ObjectId?

        // Transform {
        //     position,
        //     rotation,
        //     scale,
        // }
    }
}

// What if we had a way to "bind" raw rust structs to fields? Needs to know how to read and write,
// but we really just need to provide method of getting a ref and mutable ref to individual fields

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "df64f515-7e2f-47c2-b4d3-17ec7f2e63c7"]
pub struct AllFields {
    pub boolean: bool,
    pub int32: i32,
    pub int64: i64,
}

impl DataSetEntry for AllFields {
    fn from_data_set(
        data_container: &DataContainer,
    ) -> Self {
        let boolean = data_container
            .resolve_property("boolean")
            .unwrap()
            .as_boolean()
            .unwrap();
        let int32 = data_container
            .resolve_property("int32")
            .unwrap()
            .as_i32()
            .unwrap();
        let int64 = data_container
            .resolve_property("int64")
            .unwrap()
            .as_i64()
            .unwrap();

        AllFields {
            boolean,
            int32,
            int64,
        }
    }
}
