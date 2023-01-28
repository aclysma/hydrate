use hydrate_model::{DataSet, DataSetEntry, ObjectId, SchemaSet};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl DataSetEntry for Transform {
    fn from_data_set(
        object_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
    ) -> Self {
        let position = [
            data_set
                .resolve_property(schema, object_id, "position.x")
                .unwrap()
                .as_f32()
                .unwrap(),
            data_set
                .resolve_property(schema, object_id, "position.y")
                .unwrap()
                .as_f32()
                .unwrap(),
            data_set
                .resolve_property(schema, object_id, "position.z")
                .unwrap()
                .as_f32()
                .unwrap(),
        ];

        let rotation = [
            data_set
                .resolve_property(schema, object_id, "rotation.x")
                .unwrap()
                .as_f32()
                .unwrap(),
            data_set
                .resolve_property(schema, object_id, "rotation.y")
                .unwrap()
                .as_f32()
                .unwrap(),
            data_set
                .resolve_property(schema, object_id, "rotation.z")
                .unwrap()
                .as_f32()
                .unwrap(),
            data_set
                .resolve_property(schema, object_id, "rotation.w")
                .unwrap()
                .as_f32()
                .unwrap(),
        ];

        let scale = [
            data_set
                .resolve_property(schema, object_id, "scale.x")
                .unwrap()
                .as_f32()
                .unwrap(),
            data_set
                .resolve_property(schema, object_id, "scale.y")
                .unwrap()
                .as_f32()
                .unwrap(),
            data_set
                .resolve_property(schema, object_id, "scale.z")
                .unwrap()
                .as_f32()
                .unwrap(),
        ];

        Transform {
            position,
            rotation,
            scale,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AllFields {
    pub boolean: bool,
    pub int32: i32,
    pub int64: i64,
}

impl DataSetEntry for AllFields {
    fn from_data_set(
        object_id: ObjectId,
        data_set: &DataSet,
        schema: &SchemaSet,
    ) -> Self {
        let boolean = data_set
            .resolve_property(schema, object_id, "boolean")
            .unwrap()
            .as_boolean()
            .unwrap();
        let int32 = data_set
            .resolve_property(schema, object_id, "int32")
            .unwrap()
            .as_i32()
            .unwrap();
        let int64 = data_set
            .resolve_property(schema, object_id, "int64")
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
