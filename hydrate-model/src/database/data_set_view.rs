use std::ops::Deref;
use uuid::Uuid;
use crate::{DataSet, ObjectId, OverrideBehavior, Schema, SchemaRecord, SchemaSet, SingleObject, Value};
use crate::database::data_set::DataSetResult;

pub fn do_push_property_path(property_path_stack: &mut Vec<String>, property_path: &mut String, path: &str) {
    property_path_stack.push(path.to_string());
    // Only push a separating dot if there is already a path
    if !property_path.is_empty() {
        property_path.push('.');
    }
    property_path.push_str(path);
}

pub fn do_pop_property_path(property_path_stack: &mut Vec<String>, property_path: &mut String) {
    let fragment = property_path_stack.pop().unwrap();
    property_path.truncate(property_path.len() - fragment.len());
    // Separating dot does not need to be popped for the first pushed path fragment, in this
    // case we will already have an empty string
    if !property_path.is_empty() {
        property_path.pop().unwrap();
    }
}

fn join_path_and_field(property_path: &str, field_name: &str) -> String {
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

//TODO: Make these impl Read and Write?

pub enum DataContainer<'a> {
    DataSet(&'a DataSet, &'a SchemaSet, ObjectId),
    SingleObject(&'a SingleObject, &'a SchemaSet)
}

impl<'a> DataContainer<'a> {
    pub fn new_single_object(single_object: &'a SingleObject, schema_set: &'a SchemaSet) -> Self {
        DataContainer::SingleObject(single_object, schema_set)
    }

    pub fn new_dataset(data_set: &'a DataSet, schema_set: &'a SchemaSet, object_id: ObjectId) -> Self {
        DataContainer::DataSet(data_set, schema_set, object_id)
    }

    pub fn schema_set(&self) -> &SchemaSet {
        match *self {
            DataContainer::DataSet(_, schema_set, _) => schema_set,
            DataContainer::SingleObject(_, schema_set) => schema_set
        }
    }

    pub fn resolve_property(&self, path: impl AsRef<str>) -> Option<&Value> {
        match *self {
            DataContainer::DataSet(data_set, schema_set, object_id) => data_set.resolve_property(schema_set, object_id, path),
            DataContainer::SingleObject(single_object, schema_set) => single_object.resolve_property(schema_set, path)
        }
    }

    pub fn resolve_is_null(&self, path: impl AsRef<str>) -> Option<bool> {
        match *self {
            DataContainer::DataSet(data_set, schema_set, object_id) => data_set.resolve_is_null(schema_set, object_id, path),
            DataContainer::SingleObject(single_object, schema_set) => single_object.resolve_is_null(schema_set, path)
        }
    }

    pub fn resolve_dynamic_array(&self, path: impl AsRef<str>) -> Box<[Uuid]> {
        match *self {
            DataContainer::DataSet(data_set, schema_set, object_id) => data_set.resolve_dynamic_array(schema_set, object_id, path),
            DataContainer::SingleObject(single_object, schema_set) => single_object.resolve_dynamic_array(schema_set, path)
        }
    }

    pub fn get_override_behavior2(&self, path: impl AsRef<str>) -> OverrideBehavior {
        match *self {
            DataContainer::DataSet(data_set, schema_set, object_id) => data_set.get_override_behavior(schema_set, object_id, path),
            DataContainer::SingleObject(single_object, schema_set) => OverrideBehavior::Replace,
        }
    }


    // pub fn schema(&self, field_name: &str) {
    //     self.object_schema.find_property_schema()
    // }
}


pub enum DataContainerMut<'a> {
    DataSet(&'a mut DataSet, &'a SchemaSet, ObjectId),
    SingleObject(&'a mut SingleObject, &'a SchemaSet)
}

impl<'a> DataContainerMut<'a> {
    pub fn new_single_object(single_object: &'a mut SingleObject, schema_set: &'a SchemaSet) -> Self {
        DataContainerMut::SingleObject(single_object, schema_set)
    }

    pub fn new_dataset(data_set: &'a mut DataSet, schema_set: &'a SchemaSet, object_id: ObjectId) -> Self {
        DataContainerMut::DataSet(data_set, schema_set, object_id)
    }

    fn read(&'a self) -> DataContainer<'a> {
        match &*self {
            DataContainerMut::DataSet(a, b, c) => DataContainer::DataSet(a, b, *c),
            DataContainerMut::SingleObject(a, b) => DataContainer::SingleObject(a, b),
        }
    }

    pub fn resolve_property(&self, path: impl AsRef<str>) -> Option<&Value> {
        match self {
            DataContainerMut::DataSet(data_set, schema_set, object_id) => data_set.resolve_property(schema_set, *object_id, path),
            DataContainerMut::SingleObject(single_object, schema_set) => single_object.resolve_property(schema_set, path)
        }
    }

    pub fn resolve_is_null(&self, path: impl AsRef<str>) -> Option<bool> {
        self.read().resolve_is_null(path)
    }

    pub fn resolve_dynamic_array(&self, path: impl AsRef<str>) -> Box<[Uuid]> {
        self.read().resolve_dynamic_array(path)
    }

    pub fn get_override_behavior2(&self, path: impl AsRef<str>) -> OverrideBehavior {
        self.read().get_override_behavior2(path)
    }

    pub fn set_property_override(&mut self, path: impl AsRef<str>, value: Value) -> DataSetResult<()> {
        match self {
            DataContainerMut::DataSet(data_set, schema_set, object_id) => data_set.set_property_override(schema_set, *object_id, path, value),
            DataContainerMut::SingleObject(single_object, schema_set) => single_object.set_property_override(schema_set, path, value),
        }
    }

    pub fn set_override_behavior(&mut self, path: impl AsRef<str>, behavior: OverrideBehavior) {
        match self {
            DataContainerMut::DataSet(data_set, schema_set, object_id) => data_set.set_override_behavior(schema_set, *object_id, path, behavior),
            DataContainerMut::SingleObject(_, _) => {}
        }
    }
}


// pub struct SingleObjectView<'a> {
//     data_set: &'a SingleObject,
//     schema_set: &'a SchemaSet,
//     object_id: ObjectId,
//     property_path_stack: Vec<String>,
//     //object_schema: SchemaRecord,
//     //schema_record_stack: Vec<Schema>,
//     property_path: String,
//     //property_path: String,
// }
//
// impl<'a> SingleObjectView<'a> {
//     pub fn new(data_set: &'a SingleObject, schema_set: &'a SchemaSet, object_id: ObjectId) -> Self {
//         //let object_schema = data_set.object_schema(object_id).unwrap().clone();
//         //object_schema.fin
//         //let schema_record_stack = vec![Schema::NamedType()object_schema.clone()];
//
//         SingleObjectView {
//             data_set,
//             schema_set,
//             object_id,
//             property_path_stack: Default::default(),
//             //object_schema,
//             property_path: Default::default()
//         }
//     }
//
//     pub fn schema_set(&self) -> &SchemaSet {
//         self.schema_set
//     }
//
//     pub fn push_property_path(&mut self, path: &str) {
//         do_push_property_path(&mut self.property_path_stack, &mut self.property_path, path);
//     }
//
//     pub fn pop_property_path(&mut self) {
//         do_pop_property_path(&mut self.property_path_stack, &mut self.property_path);
//     }
//
//     pub fn resolve_property(&self, field_name: &str) -> Option<Value> {
//         self.data_set.resolve_property(self.schema_set, join_path_and_field(&self.property_path, field_name))
//     }
//
//     pub fn resolve_is_null(&self, field_name: &str) -> Option<bool> {
//         self.data_set.resolve_is_null(self.schema_set, join_path_and_field(&self.property_path, field_name))
//     }
//
//     pub fn resolve_dynamic_array(&self, field_name: &str) -> Box<[Uuid]> {
//         self.data_set.resolve_dynamic_array(self.schema_set, join_path_and_field(&self.property_path, field_name))
//     }
//
//     pub fn get_override_behavior(&self, field_name: &str) -> OverrideBehavior {
//         self.data_set.get_override_behavior(self.schema_set, join_path_and_field(&self.property_path, field_name))
//     }
//
//     // pub fn schema(&self, field_name: &str) {
//     //     self.object_schema.find_property_schema()
//     // }
// }



pub struct DataSetView<'a> {
    data_container: DataContainer<'a>,
    property_path_stack: Vec<String>,
    //object_schema: SchemaRecord,
    //schema_record_stack: Vec<Schema>,
    property_path: String,
    //property_path: String,
}

impl<'a> DataSetView<'a> {
    pub fn new(data_container: DataContainer<'a>) -> Self {
        //let object_schema = data_set.object_schema(object_id).unwrap().clone();
        //object_schema.fin
        //let schema_record_stack = vec![Schema::NamedType()object_schema.clone()];

        DataSetView {
            data_container,
            property_path_stack: Default::default(),
            //object_schema,
            property_path: Default::default()
        }
    }

    pub fn schema_set(&self) -> &SchemaSet {
        self.data_container.schema_set()
    }

    pub fn push_property_path(&mut self, path: &str) {
        do_push_property_path(&mut self.property_path_stack, &mut self.property_path, path);
    }

    pub fn pop_property_path(&mut self) {
        do_pop_property_path(&mut self.property_path_stack, &mut self.property_path);
    }

    pub fn resolve_property(&self, field_name: &str) -> Option<&Value> {
        self.data_container.resolve_property(join_path_and_field(&self.property_path, field_name))
    }

    pub fn resolve_is_null(&self, field_name: &str) -> Option<bool> {
        self.data_container.resolve_is_null(join_path_and_field(&self.property_path, field_name))
    }

    pub fn resolve_dynamic_array(&self, field_name: &str) -> Box<[Uuid]> {
        self.data_container.resolve_dynamic_array(join_path_and_field(&self.property_path, field_name))
    }

    pub fn get_override_behavior2(&self, field_name: &str) -> OverrideBehavior {
        self.data_container.get_override_behavior2(join_path_and_field(&self.property_path, field_name))
    }

    // pub fn schema(&self, field_name: &str) {
    //     self.object_schema.find_property_schema()
    // }
}


pub struct DataSetViewMut<'a> {
    data_container: DataContainerMut<'a>,
    schema_set: &'a SchemaSet,
    object_id: ObjectId,
    property_path_stack: Vec<String>,
    //object_schema: SchemaRecord,
    //schema_record_stack: Vec<Schema>,
    property_path: String,
    //property_path: String,
}

impl<'a> DataSetViewMut<'a> {
    pub fn push_property_path(&mut self, path: &str) {
        do_push_property_path(&mut self.property_path_stack, &mut self.property_path, path);
    }

    pub fn pop_property_path(&mut self) {
        do_pop_property_path(&mut self.property_path_stack, &mut self.property_path);
    }

    pub fn set_property_override(&mut self, field_name: &str, value: Value) -> DataSetResult<()> {
        self.data_container.set_property_override(join_path_and_field(&self.property_path, field_name), value)
    }

    pub fn set_override_behavior(&mut self, field_name: &str, behavior: OverrideBehavior) {
        self.data_container.set_override_behavior(join_path_and_field(&self.property_path, field_name), behavior)
    }
}