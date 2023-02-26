mod dynamic_array;

use std::marker::PhantomData;
pub use dynamic_array::*;
use crate::{DataContainer, DataContainerMut, DataSetError, DataSetResult, DataSetView, DataSetViewMut, Value};
use crate::value::ValueEnum;

#[derive(Default)]
pub struct PropertyPath(String);

impl PropertyPath {
    pub fn push(&self, str: &str) -> PropertyPath {
        if self.0.is_empty() {
            PropertyPath(str.to_string())
        } else if str.is_empty() {
            PropertyPath(self.0.to_string())
        } else {
            PropertyPath(format!("{}.{}", self.0, str))
        }
    }

    pub fn path(&self) -> &str {
        &self.0
    }
}

trait ReadSource {
    fn resolve_property(&self, path: impl AsRef<str>) -> Option<Value>;
}

impl<'a> ReadSource for DataContainer<'a> {
    fn resolve_property(&self, path: impl AsRef<str>) -> Option<Value> {
        self.resolve_property(path)
    }
}

impl<'a> ReadSource for DataContainerMut<'a> {
    fn resolve_property(&self, path: impl AsRef<str>) -> Option<Value> {
        self.resolve_property(path)
    }
}




pub trait Field {
    fn new(property_path: PropertyPath) -> Self;
}

pub trait Enum: Sized {
    fn to_symbol_name(&self) -> &'static str;
    fn from_symbol_name(str: &str) -> Option<Self>;
}



pub struct EnumField<T: Enum>(PropertyPath, PhantomData<T>);

impl<T: Enum> Field for EnumField<T> {
    fn new(property_path: PropertyPath) -> Self {
        EnumField(property_path, PhantomData::default())
    }
}

impl<T: Enum> EnumField<T> {
    pub fn get(&self, data_set_view: &DataContainer) -> DataSetResult<T> {
        let e = data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?;
        T::from_symbol_name(e.as_enum().unwrap().symbol_name()).ok_or(DataSetError::UnexpectedEnumSymbol)
    }

    pub fn set(&self, data_set_view: &mut DataContainerMut, value: T) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::Enum(ValueEnum::new(value.to_symbol_name().to_string())))
    }
}





pub struct NullableField<T: Field>(pub PropertyPath, PhantomData<T>);

impl<T: Field> Field for NullableField<T> {
    fn new(property_path: PropertyPath) -> Self {
        NullableField(property_path, PhantomData::default())
    }
}

impl<T: Field> NullableField<T> {
    pub fn get(&self, data_set_view: &DataSetView) -> Option<T> {
        if data_set_view.resolve_is_null(self.0.path()) == Some(false) {
            Some(T::new(self.0.push("value")))
        } else {
            None
        }
    }

    // set_is_null

    // is_null

    // pub fn set(&self, data_set_view: &mut DataSetViewMut, value: f32) -> DataSetResult<()> {
    //     //TODO: This is wrong
    //     //data_set_view.set_property_override(self.0.path(), Value::F32(value))
    // }
}

// Getter, Accessor, Field, Member, Prop, Property, Value, Schema, Path,
// consider how containers and nullable works?
//
// Use Field/Record/Enum/Fixed terminology?
pub struct F32Field(pub PropertyPath);

impl Field for F32Field {
    fn new(property_path: PropertyPath) -> Self {
        F32Field(property_path)
    }
}

impl F32Field {
    pub fn get(&self, data_set_view: &DataContainer) -> DataSetResult<f32> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_f32().unwrap())
    }

    pub fn set(&self, data_set_view: &mut DataContainerMut, value: f32) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::F32(value))
    }
}

pub struct F64Field(pub PropertyPath);

impl Field for F64Field {
    fn new(property_path: PropertyPath) -> Self {
        F64Field(property_path)
    }
}

impl F64Field {
    pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<f64> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_f64().unwrap())
    }

    pub fn set(&self, data_set_view: &mut DataSetViewMut, value: f64) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::F64(value))
    }
}

pub struct I32Field(pub PropertyPath);

impl Field for I32Field {
    fn new(property_path: PropertyPath) -> Self {
        I32Field(property_path)
    }
}

impl I32Field {
    pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<i32> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_i32().unwrap())
    }

    pub fn set(&self, data_set_view: &mut DataSetViewMut, value: i32) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::I32(value))
    }
}

pub struct U32Field(pub PropertyPath);

impl Field for U32Field {
    fn new(property_path: PropertyPath) -> Self {
        U32Field(property_path)
    }
}

impl U32Field {
    pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<u32> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_u32().unwrap())
    }

    pub fn set(&self, data_set_view: &mut DataSetViewMut, value: u32) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::U32(value))
    }
}


pub struct BooleanField(pub PropertyPath);

impl Field for BooleanField {
    fn new(property_path: PropertyPath) -> Self {
        BooleanField(property_path)
    }
}

impl BooleanField {
    pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<bool> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_boolean().unwrap())
    }

    pub fn set(&self, data_set_view: &mut DataSetViewMut, value: bool) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::Boolean(value))
    }
}


pub struct StringField(pub PropertyPath);

impl Field for StringField {
    fn new(property_path: PropertyPath) -> Self {
        StringField(property_path)
    }
}

impl StringField {
    pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<String> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_string().unwrap().to_string())
    }

    pub fn set(&self, data_set_view: &mut DataSetViewMut, value: String) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::String(value))
    }
}

