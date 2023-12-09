use crate::data_set::DataSetResult;
use crate::{AssetId, DataSet, NullOverride, OverrideBehavior, SchemaSet, SingleObject, Value};
use std::sync::Arc;
use uuid::Uuid;

trait DataContainerRead {
    fn resolve_property(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value>;

    fn get_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride>;

    fn resolve_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride>;

    fn resolve_dynamic_array(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>>;

    fn resolve_map(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>>;

    fn get_override_behavior(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<OverrideBehavior>;
}

trait DataContainerWrite {
    fn set_null_override(
        &mut self,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) -> DataSetResult<()>;

    fn set_property_override(
        &mut self,
        path: impl AsRef<str>,
        value: Option<Value>,
    ) -> DataSetResult<Option<Value>>;

    fn set_override_behavior(
        &mut self,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) -> DataSetResult<()>;

    fn add_dynamic_array_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid>;

    fn add_map_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid>;
}

/// Provides a read-only view into a DataSet or SingleObject. A schema can be used to write into
/// both forms.
#[derive(Clone)]
pub enum DataContainerRef<'a> {
    DataSet(&'a DataSet, &'a SchemaSet, AssetId),
    SingleObjectRef(&'a SingleObject, &'a SchemaSet),
    SingleObjectArc(Arc<SingleObject>, &'a SchemaSet),
}

impl<'a> DataContainerRef<'a> {
    pub fn from_single_object(
        single_object: &'a SingleObject,
        schema_set: &'a SchemaSet,
    ) -> Self {
        DataContainerRef::SingleObjectRef(single_object, schema_set)
    }

    pub fn from_single_object_arc(
        single_object: Arc<SingleObject>,
        schema_set: &'a SchemaSet,
    ) -> Self {
        DataContainerRef::SingleObjectArc(single_object, schema_set)
    }

    pub fn from_dataset(
        data_set: &'a DataSet,
        schema_set: &'a SchemaSet,
        asset_id: AssetId,
    ) -> Self {
        DataContainerRef::DataSet(data_set, schema_set, asset_id)
    }

    pub fn schema_set(&self) -> &SchemaSet {
        match *self {
            DataContainerRef::DataSet(_, schema_set, _) => schema_set,
            DataContainerRef::SingleObjectRef(_, schema_set) => schema_set,
            DataContainerRef::SingleObjectArc(_, schema_set) => schema_set,
        }
    }

    pub fn resolve_property(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value> {
        match self {
            DataContainerRef::DataSet(data_set, schema_set, asset_id) => {
                data_set.resolve_property(schema_set, *asset_id, path)
            }
            DataContainerRef::SingleObjectRef(single_object, schema_set) => {
                single_object.resolve_property(schema_set, path)
            }
            DataContainerRef::SingleObjectArc(single_object, schema_set) => {
                single_object.resolve_property(schema_set, path)
            }
        }
    }

    pub fn get_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        match self {
            DataContainerRef::DataSet(data_set, schema_set, asset_id) => {
                data_set.get_null_override(schema_set, *asset_id, path)
            }
            DataContainerRef::SingleObjectRef(single_object, schema_set) => {
                single_object.get_null_override(schema_set, path)
            }
            DataContainerRef::SingleObjectArc(single_object, schema_set) => {
                single_object.get_null_override(schema_set, path)
            }
        }
    }

    pub fn resolve_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        match self {
            DataContainerRef::DataSet(data_set, schema_set, asset_id) => {
                data_set.resolve_null_override(schema_set, *asset_id, path)
            }
            DataContainerRef::SingleObjectRef(single_object, schema_set) => {
                single_object.resolve_null_override(schema_set, path)
            }
            DataContainerRef::SingleObjectArc(single_object, schema_set) => {
                single_object.resolve_null_override(schema_set, path)
            }
        }
    }

    pub fn resolve_dynamic_array(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        match self {
            DataContainerRef::DataSet(data_set, schema_set, asset_id) => {
                data_set.resolve_dynamic_array(schema_set, *asset_id, path)
            }
            DataContainerRef::SingleObjectRef(single_object, schema_set) => {
                single_object.resolve_dynamic_array(schema_set, path)
            }
            DataContainerRef::SingleObjectArc(single_object, schema_set) => {
                single_object.resolve_dynamic_array(schema_set, path)
            }
        }
    }

    pub fn resolve_map(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        match self {
            DataContainerRef::DataSet(data_set, schema_set, asset_id) => {
                data_set.resolve_map(schema_set, *asset_id, path)
            }
            DataContainerRef::SingleObjectRef(single_object, schema_set) => {
                single_object.resolve_map(schema_set, path)
            }
            DataContainerRef::SingleObjectArc(single_object, schema_set) => {
                single_object.resolve_map(schema_set, path)
            }
        }
    }

    pub fn get_override_behavior(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<OverrideBehavior> {
        match self {
            DataContainerRef::DataSet(data_set, schema_set, asset_id) => {
                data_set.get_override_behavior(schema_set, *asset_id, path)
            }
            DataContainerRef::SingleObjectRef(_, _) => Ok(OverrideBehavior::Replace),
            DataContainerRef::SingleObjectArc(_, _) => Ok(OverrideBehavior::Replace),
        }
    }
}

impl<'a> DataContainerRead for DataContainerRef<'a> {
    fn resolve_property(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value> {
        self.resolve_property(path)
    }

    fn get_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.get_null_override(path)
    }

    fn resolve_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.resolve_null_override(path)
    }

    fn resolve_dynamic_array(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.resolve_dynamic_array(path)
    }

    fn resolve_map(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.resolve_dynamic_array(path)
    }

    fn get_override_behavior(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<OverrideBehavior> {
        self.get_override_behavior(path)
    }
}

/// Provides a read/write view into a DataSet or SingleObject. A schema can be used to write into
/// both forms.
pub enum DataContainerRefMut<'a> {
    DataSet(&'a mut DataSet, &'a SchemaSet, AssetId),
    SingleObject(&'a mut SingleObject, &'a SchemaSet),
}

impl<'a> DataContainerRefMut<'a> {
    pub fn from_single_object(
        single_object: &'a mut SingleObject,
        schema_set: &'a SchemaSet,
    ) -> Self {
        DataContainerRefMut::SingleObject(single_object, schema_set)
    }

    pub fn from_dataset(
        data_set: &'a mut DataSet,
        schema_set: &'a SchemaSet,
        asset_id: AssetId,
    ) -> Self {
        DataContainerRefMut::DataSet(data_set, schema_set, asset_id)
    }

    pub fn read(&'a self) -> DataContainerRef<'a> {
        match &*self {
            DataContainerRefMut::DataSet(a, b, c) => DataContainerRef::DataSet(a, b, *c),
            DataContainerRefMut::SingleObject(a, b) => DataContainerRef::SingleObjectRef(a, b),
        }
    }

    pub fn resolve_property(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value> {
        // We can't simply call read().resolve_property() because rust can't prove the borrowing safety
        match self {
            DataContainerRefMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.resolve_property(schema_set, *asset_id, path)
            }
            DataContainerRefMut::SingleObject(single_object, schema_set) => {
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
            DataContainerRefMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.set_null_override(schema_set, *asset_id, path, null_override)
            }
            DataContainerRefMut::SingleObject(single_object, schema_set) => {
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

    pub fn resolve_map(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.read().resolve_map(path)
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
            DataContainerRefMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.set_property_override(schema_set, *asset_id, path, value)
            }
            DataContainerRefMut::SingleObject(single_object, schema_set) => {
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
            DataContainerRefMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.set_override_behavior(schema_set, *asset_id, path, behavior)
            }
            DataContainerRefMut::SingleObject(_, _) => Ok(()),
        }
    }

    pub fn add_dynamic_array_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        match self {
            DataContainerRefMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.add_dynamic_array_override(schema_set, *asset_id, path)
            }
            DataContainerRefMut::SingleObject(single_object, schema_set) => {
                single_object.add_dynamic_array_override(schema_set, path)
            }
        }
    }

    pub fn add_map_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        match self {
            DataContainerRefMut::DataSet(data_set, schema_set, asset_id) => {
                data_set.add_map_override(schema_set, *asset_id, path)
            }
            DataContainerRefMut::SingleObject(single_object, schema_set) => {
                single_object.add_map_override(schema_set, path)
            }
        }
    }
}

impl<'a> DataContainerRead for DataContainerRefMut<'a> {
    fn resolve_property(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value> {
        self.resolve_property(path)
    }

    fn get_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.get_null_override(path)
    }

    fn resolve_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.resolve_null_override(path)
    }

    fn resolve_dynamic_array(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.resolve_dynamic_array(path)
    }

    fn resolve_map(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.resolve_map(path)
    }

    fn get_override_behavior(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<OverrideBehavior> {
        self.get_override_behavior(path)
    }
}

impl<'a> DataContainerWrite for DataContainerRefMut<'a> {
    fn set_null_override(
        &mut self,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) -> DataSetResult<()> {
        self.set_null_override(path, null_override)
    }

    fn set_property_override(
        &mut self,
        path: impl AsRef<str>,
        value: Option<Value>,
    ) -> DataSetResult<Option<Value>> {
        self.set_property_override(path, value)
    }

    fn set_override_behavior(
        &mut self,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) -> DataSetResult<()> {
        self.set_override_behavior(path, behavior)
    }

    fn add_dynamic_array_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        self.add_dynamic_array_override(path)
    }

    fn add_map_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        self.add_map_override(path)
    }
}

/// Provides a read/write view into a DataSet or SingleObject. A schema can be used to write into
/// both forms.
pub enum DataContainer {
    SingleObject(SingleObject, SchemaSet),
}

impl DataContainer {
    pub fn into_inner(self) -> SingleObject {
        match self {
            DataContainer::SingleObject(a, _b) => a,
        }
    }

    pub fn from_single_object(
        single_object: SingleObject,
        schema_set: SchemaSet,
    ) -> Self {
        DataContainer::SingleObject(single_object, schema_set)
    }

    pub fn read<'a>(&'a self) -> DataContainerRef<'a> {
        match self {
            DataContainer::SingleObject(a, b) => DataContainerRef::SingleObjectRef(&a, &b),
        }
    }

    pub fn to_mut(&mut self) -> DataContainerRefMut {
        match self {
            DataContainer::SingleObject(a, b) => DataContainerRefMut::SingleObject(a, b),
        }
    }

    pub fn resolve_property(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value> {
        // We can't simply call read().resolve_property() because rust can't prove the borrowing safety
        match self {
            DataContainer::SingleObject(single_object, schema_set) => {
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
            DataContainer::SingleObject(single_object, schema_set) => {
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

    pub fn resolve_map(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.read().resolve_map(path)
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
            DataContainer::SingleObject(single_object, schema_set) => {
                single_object.set_property_override(schema_set, path, value)
            }
        }
    }

    pub fn set_override_behavior(
        &mut self,
        _path: impl AsRef<str>,
        _behavior: OverrideBehavior,
    ) -> DataSetResult<()> {
        match self {
            DataContainer::SingleObject(_, _) => Ok(()),
        }
    }

    pub fn add_dynamic_array_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        match self {
            DataContainer::SingleObject(single_object, schema_set) => {
                single_object.add_dynamic_array_override(schema_set, path)
            }
        }
    }

    pub fn add_map_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        match self {
            DataContainer::SingleObject(single_object, schema_set) => {
                single_object.add_map_override(schema_set, path)
            }
        }
    }
}

impl DataContainerRead for DataContainer {
    fn resolve_property(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<&Value> {
        self.resolve_property(path)
    }

    fn get_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.get_null_override(path)
    }

    fn resolve_null_override(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        self.resolve_null_override(path)
    }

    fn resolve_dynamic_array(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.resolve_dynamic_array(path)
    }

    fn resolve_map(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.resolve_map(path)
    }

    fn get_override_behavior(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<OverrideBehavior> {
        self.get_override_behavior(path)
    }
}

impl DataContainerWrite for DataContainer {
    fn set_null_override(
        &mut self,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) -> DataSetResult<()> {
        self.set_null_override(path, null_override)
    }

    fn set_property_override(
        &mut self,
        path: impl AsRef<str>,
        value: Option<Value>,
    ) -> DataSetResult<Option<Value>> {
        self.set_property_override(path, value)
    }

    fn set_override_behavior(
        &mut self,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) -> DataSetResult<()> {
        self.set_override_behavior(path, behavior)
    }

    fn add_dynamic_array_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        self.add_dynamic_array_override(path)
    }

    fn add_map_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        self.add_map_override(path)
    }
}
