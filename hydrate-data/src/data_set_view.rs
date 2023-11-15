use crate::data_set::DataSetResult;
use crate::{DataSet, NullOverride, AssetId, OverrideBehavior, SchemaSet, SingleObject, Value};
use uuid::Uuid;
/*
pub fn do_push_property_path(
    property_path_stack: &mut Vec<String>,
    property_path: &mut String,
    path: &str,
) {
    property_path_stack.push(path.to_string());
    // Only push a separating dot if there is already a path
    if !property_path.is_empty() {
        property_path.push('.');
    }
    property_path.push_str(path);
}

pub fn do_pop_property_path(
    property_path_stack: &mut Vec<String>,
    property_path: &mut String,
) {
    let fragment = property_path_stack.pop().unwrap();
    property_path.truncate(property_path.len() - fragment.len());
    // Separating dot does not need to be popped for the first pushed path fragment, in this
    // case we will already have an empty string
    if !property_path.is_empty() {
        property_path.pop().unwrap();
    }
}

fn join_path_and_field(
    property_path: &str,
    field_name: &str,
) -> String {
    if property_path.is_empty() {
        field_name.to_string()
    } else {
        if field_name.is_empty() {
            property_path.to_string()
        } else {
            format!("{}.{}", property_path, field_name)
        }
    }
}
*/
//TODO: Make these impl Read and Write?

/// Provides a read-only view into a DataSet or SingleObject. A schema can be used to write into
/// both forms.
pub enum DataContainer<'a> {
    DataSet(&'a DataSet, &'a SchemaSet, AssetId),
    SingleObject(&'a SingleObject, &'a SchemaSet),
}

impl<'a> DataContainer<'a> {
    pub fn from_single_object(
        single_object: &'a SingleObject,
        schema_set: &'a SchemaSet,
    ) -> Self {
        DataContainer::SingleObject(single_object, schema_set)
    }

    pub fn from_dataset(
        data_set: &'a DataSet,
        schema_set: &'a SchemaSet,
        asset_id: AssetId,
    ) -> Self {
        DataContainer::DataSet(data_set, schema_set, asset_id)
    }

    pub fn schema_set(&self) -> &SchemaSet {
        match *self {
            DataContainer::DataSet(_, schema_set, _) => schema_set,
            DataContainer::SingleObject(_, schema_set) => schema_set,
        }
    }

    pub fn resolve_property(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value> {
        match *self {
            DataContainer::DataSet(data_set, schema_set, asset_id) => {
                data_set.resolve_property(schema_set, asset_id, path)
            }
            DataContainer::SingleObject(single_object, schema_set) => {
                single_object.resolve_property(schema_set, path)
            }
        }
    }

    pub fn get_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        match *self {
            DataContainer::DataSet(data_set, schema_set, asset_id) => {
                data_set.get_null_override(schema_set, asset_id, path)
            }
            DataContainer::SingleObject(single_object, schema_set) => {
                single_object.get_null_override(schema_set, path)
            }
        }
    }

    pub fn resolve_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        match *self {
            DataContainer::DataSet(data_set, schema_set, asset_id) => {
                data_set.resolve_null_override(schema_set, asset_id, path)
            }
            DataContainer::SingleObject(single_object, schema_set) => {
                single_object.resolve_null_override(schema_set, path)
            }
        }
    }

    pub fn resolve_dynamic_array(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        match *self {
            DataContainer::DataSet(data_set, schema_set, asset_id) => {
                data_set.resolve_dynamic_array(schema_set, asset_id, path)
            }
            DataContainer::SingleObject(single_object, schema_set) => {
                single_object.resolve_dynamic_array(schema_set, path)
            }
        }
    }

    pub fn get_override_behavior(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<OverrideBehavior> {
        match *self {
            DataContainer::DataSet(data_set, schema_set, asset_id) => {
                data_set.get_override_behavior(schema_set, asset_id, path)
            }
            DataContainer::SingleObject(_, _) => Ok(OverrideBehavior::Replace),
        }
    }
}

/// Provides a read/write view into a DataSet or SingleObject. A schema can be used to write into
/// both forms.
pub enum DataContainerMut<'a> {
    DataSet(&'a mut DataSet, &'a SchemaSet, AssetId),
    SingleObject(&'a mut SingleObject, &'a SchemaSet),
}

impl<'a> DataContainerMut<'a> {
    pub fn from_single_object(
        single_object: &'a mut SingleObject,
        schema_set: &'a SchemaSet,
    ) -> Self {
        DataContainerMut::SingleObject(single_object, schema_set)
    }

    pub fn from_dataset(
        data_set: &'a mut DataSet,
        schema_set: &'a SchemaSet,
        asset_id: AssetId,
    ) -> Self {
        DataContainerMut::DataSet(data_set, schema_set, asset_id)
    }

    fn read(&'a self) -> DataContainer<'a> {
        match &*self {
            DataContainerMut::DataSet(a, b, c) => DataContainer::DataSet(a, b, *c),
            DataContainerMut::SingleObject(a, b) => DataContainer::SingleObject(a, b),
        }
    }

    pub fn resolve_property(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value> {
        match self {
            DataContainerMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.resolve_property(schema_set, *asset_id, path)
            }
            DataContainerMut::SingleObject(single_object, schema_set) => {
                single_object.resolve_property(schema_set, path)
            }
        }
    }

    pub fn get_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.read().get_null_override(path)
    }

    pub fn set_null_override(
        &mut self,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) -> DataSetResult<()> {
        match self {
            DataContainerMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.set_null_override(schema_set, *asset_id, path, null_override)
            }
            DataContainerMut::SingleObject(single_object, schema_set) => {
                single_object.set_null_override(schema_set, path, null_override)
            }
        }
    }

    pub fn resolve_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.read().resolve_null_override(path)
    }

    pub fn resolve_dynamic_array(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.read().resolve_dynamic_array(path)
    }

    pub fn get_override_behavior(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<OverrideBehavior> {
        self.read().get_override_behavior(path)
    }

    pub fn set_property_override(
        &mut self,
        path: impl AsRef<str>,
        value: Option<Value>,
    ) -> DataSetResult<Option<Value>> {
        match self {
            DataContainerMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.set_property_override(schema_set, *asset_id, path, value)
            }
            DataContainerMut::SingleObject(single_object, schema_set) => {
                single_object.set_property_override(schema_set, path, value)
            }
        }
    }

    pub fn set_override_behavior(
        &mut self,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) -> DataSetResult<()> {
        match self {
            DataContainerMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.set_override_behavior(schema_set, *asset_id, path, behavior)
            }
            DataContainerMut::SingleObject(_, _) => { Ok(()) }
        }
    }

    pub fn add_dynamic_array_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        match self {
            DataContainerMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.add_dynamic_array_override(schema_set, *asset_id, path)
            }
            DataContainerMut::SingleObject(single_object, schema_set) => {
                single_object.add_dynamic_array_override(schema_set, path)
            }
        }
    }
}
