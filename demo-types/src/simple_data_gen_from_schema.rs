use hydrate_model::{DataSet, DataSetEntry, DataSetResult, DataSetView, DataSetViewMut, F32Field, Field, NullableField, ObjectId, PropertyPath, SchemaSet, Value};
use serde::{Deserialize, Serialize};
use type_uuid::TypeUuid;
use hydrate_base::{AssetUuid, Handle};
use hydrate_base::handle::GenericHandle;

#[derive(Default)]
pub struct Vec3Record(PropertyPath);

impl Field for Vec3Record {
    fn new(property_path: PropertyPath) -> Self {
        Vec3Record(property_path)
    }
}

impl Vec3Record {
    pub fn x(&self) -> F32Field { F32Field::new(self.0.push("x")) }
    pub fn y(&self) -> F32Field { F32Field::new(self.0.push("y")) }
    pub fn z(&self) -> F32Field { F32Field::new(self.0.push("z")) }
}

#[derive(Default)]
pub struct Vec4Record(PropertyPath);

impl Field for Vec4Record {
    fn new(property_path: PropertyPath) -> Self {
        Vec4Record(property_path)
    }
}

impl Vec4Record {
    pub fn x(&self) -> F32Field { F32Field::new(self.0.push("x")) }
    pub fn y(&self) -> F32Field { F32Field::new(self.0.push("y")) }
    pub fn z(&self) -> F32Field { F32Field::new(self.0.push("z")) }
    pub fn w(&self) -> F32Field { F32Field::new(self.0.push("w")) }
}

#[derive(Default)]
pub struct TransformRecord(PropertyPath);

impl Field for TransformRecord {
    fn new(property_path: PropertyPath) -> Self {
        TransformRecord(property_path)
    }
}

impl TransformRecord {
    pub fn position(&self) -> Vec3Record { Vec3Record::new(self.0.push("position")) }
    pub fn rotation(&self) -> Vec4Record { Vec4Record::new(self.0.push("rotation")) }
    pub fn scale(&self) -> Vec3Record { Vec3Record::new(self.0.push("scale")) }
}

#[derive(Default)]
pub struct TestRecord(PropertyPath);

impl TestRecord {
    pub fn position(&self) -> NullableField<F32Field> { NullableField::new(self.0.push("position")) }
}

/*

//
// In theory this is the auto-generated code
//
pub struct Vec3FromSchema {
    pub x: f32,
    pub y: f32,
    pub z: f32
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
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
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
*/
/*
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
*/

// We have strongly typed handles, and we want to generate code using the handles pointing to end-user created types
// end-users can provide their own types to transform the autogenerated schema types into
// Original plan was to allow users to implement Into<T> transforming autogen types to engine types
// Vec3 and Vec4 to end-user math type like glam may be a special case?
// So we have a cyclical dependency problem...
// - Types needing to impl From<T> where T is schema-generated code
// - Schema types needing to have Handle<T> where T is a user-implemented type (that possibly impls From...
//
// So don't see a way around supporting these types being in the same crate
//
// As a second pass, would be nice to be able to tweak the generated code
// - Derives (like Serialize, Debug, etc.)
// - Extra metadata like providing a type UUID
// - Overriding the name of the generated type
//
// We probably still need pure-schema types, even if we generate an "engine" type
//
// We could break the cyclical dependency of schema and engine types by having schema types use
// GenericHandle instead of strongly-typed handle
//
// Maybe this is needed anyways to support reading invalid data in the editor?
//
// The schema types should have ObjectId/AssetId, not handles. So that fixes the cyclic reference
//
// We don't make engine types until the build step, at that point we can call make_handle to convert
// asset id/object id to a typed or untyped handle



/*

// This would allow influence on generation of the schema type and the engine type
{
    type: "Transform",
    derives: "Serialize, Deserialize, Debug",
    uuid: "...",
    rust_name: "Transform",
}

*/

pub struct TestFromSchema {
    pub opt_bool: Option<bool>,
    pub opt_opt_bool: Option<Option<bool>>,
    pub bool_vec: Vec<bool>,
}

impl TestFromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let opt_bool = if !data_set_view.resolve_is_null("opt_bool").unwrap() {
            data_set_view.push_property_path("opt_bool");
            let p = Some(data_set_view.resolve_property("value").unwrap().as_boolean().unwrap());
            data_set_view.pop_property_path();
            p
        } else {
            None
        };

        let opt_opt_bool = if !data_set_view.resolve_is_null("opt_bool").unwrap() {
            data_set_view.push_property_path("opt_bool");

            let value = if !data_set_view.resolve_is_null("value").unwrap() {
                data_set_view.push_property_path("value");
                let value = data_set_view.resolve_property("").unwrap().as_boolean().unwrap();
                data_set_view.pop_property_path();
                Some(value)
            } else {
                None
            };

            data_set_view.pop_property_path();
            Some(value)
        } else {
            None
        };

        let mut bool_vec = Vec::default();
        for item in &*data_set_view.resolve_dynamic_array("bool_vec") {
            data_set_view.push_property_path(&item.to_string());
            let value = data_set_view.resolve_property("").unwrap().as_boolean().unwrap();
            data_set_view.pop_property_path();
            bool_vec.push(value);
        }





        TestFromSchema {
            opt_bool,
            opt_opt_bool,
            bool_vec
        }
    }

    pub fn store(&mut self, data_set_view: &mut DataSetViewMut) -> DataSetResult<()> {
        if let Some(opt_bool) = self.opt_bool {
            data_set_view.set_property_override("opt_bool.value", Value::Boolean(opt_bool))?;
        }

        Ok(())
    }
}

pub struct TransformRefFromSchema {
    pub transform: ObjectId
}

impl TransformRefFromSchema {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let transform = data_set_view.resolve_property("transform").unwrap().as_object_ref().unwrap();

        TransformRefFromSchema {
            transform
        }
    }

    pub fn store(&mut self, data_set_view: &mut DataSetViewMut) -> DataSetResult<()> {
        data_set_view.set_property_override("transform", Value::ObjectRef(self.transform))?;

        Ok(())
    }
}
