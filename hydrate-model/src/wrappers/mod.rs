mod dynamic_array;
pub use dynamic_array::*;
use crate::{DataSetError, DataSetResult, DataSetView, DataSetViewMut, Value};

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

// Getter, Accessor, Field, Member, Prop, Property, Value, Schema, Path,
// consider how containers and nullable works?
//
// Use Field/Record/Enum/Fixed terminology?
pub struct F32Field(PropertyPath);

impl F32Field {
    pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<f32> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_f32().unwrap())
    }

    pub fn set(&self, data_set_view: &mut DataSetViewMut, value: f32) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::F32(value))
    }
}

pub struct F64Field(PropertyPath);

impl F64Field {
    pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<f64> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_f64().unwrap())
    }

    pub fn set(&self, data_set_view: &mut DataSetViewMut, value: f64) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::F64(value))
    }
}

pub struct I32Field(PropertyPath);

impl I32Field {
    pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<i32> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_i32().unwrap())
    }

    pub fn set(&self, data_set_view: &mut DataSetViewMut, value: i32) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::I32(value))
    }
}

pub struct U32Field(PropertyPath);

impl U32Field {
    pub fn get(&self, data_set_view: &DataSetView) -> DataSetResult<u32> {
        Ok(data_set_view.resolve_property(self.0.path()).ok_or(DataSetError::PathParentIsNull)?.as_u32().unwrap())
    }

    pub fn set(&self, data_set_view: &mut DataSetViewMut, value: u32) -> DataSetResult<()> {
        data_set_view.set_property_override(self.0.path(), Value::U32(value))
    }
}
