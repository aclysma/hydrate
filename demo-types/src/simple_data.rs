use hydrate_model::{DataSet, DataSetEntry, DataSetResult, DataSetView, DataSetViewMut, ObjectId, SchemaSet, Value};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use hydrate_base::{AssetUuid, Handle};


//
// In theory this is the auto-generated code
//
pub struct Vec3FromSchema {
    x: f32,
    y: f32,
    z: f32
}

impl Vec3FromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let x = data_set_view.resolve_property("x").unwrap().as_f32().unwrap();
        let y = data_set_view.resolve_property("y").unwrap().as_f32().unwrap();
        let z = data_set_view.resolve_property("z").unwrap().as_f32().unwrap();

        Vec3FromSchema {
            x,
            y,
            z
        }
    }

    pub fn store(&mut self, data_set_view: &mut DataSetViewMut) -> DataSetResult<()> {
        data_set_view.set_property_override("x", Value::F32(self.x))?;
        data_set_view.set_property_override("y", Value::F32(self.y))?;
        data_set_view.set_property_override("z", Value::F32(self.z))?;
        Ok(())
    }
}

pub struct Vec4FromSchema {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Vec4FromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let x = data_set_view.resolve_property("x").unwrap().as_f32().unwrap();
        let y = data_set_view.resolve_property("y").unwrap().as_f32().unwrap();
        let z = data_set_view.resolve_property("z").unwrap().as_f32().unwrap();
        let w = data_set_view.resolve_property("w").unwrap().as_f32().unwrap();

        Vec4FromSchema {
            x,
            y,
            z,
            w
        }
    }

    pub fn store(&mut self, data_set_view: &mut DataSetViewMut) -> DataSetResult<()> {
        data_set_view.set_property_override("x", Value::F32(self.x))?;
        data_set_view.set_property_override("y", Value::F32(self.y))?;
        data_set_view.set_property_override("z", Value::F32(self.z))?;
        data_set_view.set_property_override("w", Value::F32(self.w))?;
        Ok(())
    }
}

pub struct TransformFromSchema {
    pub position: Vec3FromSchema,
    pub rotation: Vec4FromSchema,
    pub scale: Vec3FromSchema,
}

impl TransformFromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        data_set_view.push_property_path("position");
        let position = Vec3FromSchema::load(data_set_view);
        data_set_view.pop_property_path();

        data_set_view.push_property_path("rotation");
        let rotation = Vec4FromSchema::load(data_set_view);
        data_set_view.pop_property_path();

        data_set_view.push_property_path("scale");
        let scale = Vec3FromSchema::load(data_set_view);
        data_set_view.pop_property_path();

        //let test_field = data_set.resolve_property(schema, object_id, "test_ref").unwrap().as_object_ref().unwrap();
        // Create handle passing the ObjectId?

        TransformFromSchema {
            position,
            rotation,
            scale,
        }
    }

    pub fn store(&mut self, data_set_view: &mut DataSetViewMut) -> DataSetResult<()> {
        data_set_view.push_property_path("position");
        self.position.store(data_set_view)?;
        data_set_view.pop_property_path();

        data_set_view.push_property_path("rotation");
        self.rotation.store(data_set_view)?;
        data_set_view.pop_property_path();

        data_set_view.push_property_path("scale");
        self.scale.store(data_set_view)?;
        data_set_view.pop_property_path();

        Ok(())
    }
}

//
// Hand-implemented helper code for the schema -> engine data conversion
//
impl Into<[f32; 3]> for Vec3FromSchema {
    fn into(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}
impl Into<[f32; 4]> for Vec4FromSchema {
    fn into(self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }
}

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
        data_set_view: &mut DataSetView,
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
        data_set_view: &mut DataSetView,
    ) -> Self {
        let data = TransformFromSchema::load(data_set_view);

        Transform {
            position: data.position.into(),
            rotation: data.rotation.into(),
            scale: data.scale.into(),
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

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "df64f515-7e2f-47c2-b4d3-17ec7f2e63c7"]
pub struct AllFields {
    pub boolean: bool,
    pub int32: i32,
    pub int64: i64,
}

impl DataSetEntry for AllFields {
    fn from_data_set(
        data_set_view: &mut DataSetView,
    ) -> Self {
        let boolean = data_set_view
            .resolve_property("boolean")
            .unwrap()
            .as_boolean()
            .unwrap();
        let int32 = data_set_view
            .resolve_property("int32")
            .unwrap()
            .as_i32()
            .unwrap();
        let int64 = data_set_view
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
