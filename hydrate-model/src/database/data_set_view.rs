use uuid::Uuid;
use crate::{DataSet, ObjectId, Schema, SchemaRecord, SchemaSet, Value};
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

pub struct DataSetView<'a> {
    data_set: &'a DataSet,
    schema_set: &'a SchemaSet,
    object_id: ObjectId,
    property_path_stack: Vec<String>,
    //object_schema: SchemaRecord,
    //schema_record_stack: Vec<Schema>,
    property_path: String,
    //property_path: String,
}

impl<'a> DataSetView<'a> {
    pub fn new(data_set: &'a DataSet, schema_set: &'a SchemaSet, object_id: ObjectId) -> Self {
        //let object_schema = data_set.object_schema(object_id).unwrap().clone();
        //object_schema.fin
        //let schema_record_stack = vec![Schema::NamedType()object_schema.clone()];

        DataSetView {
            data_set,
            schema_set,
            object_id,
            property_path_stack: Default::default(),
            //object_schema,
            property_path: Default::default()
        }
    }

    pub fn push_property_path(&mut self, path: &str) {
        do_push_property_path(&mut self.property_path_stack, &mut self.property_path, path);
    }

    pub fn pop_property_path(&mut self) {
        do_pop_property_path(&mut self.property_path_stack, &mut self.property_path);
    }

    pub fn resolve_property(&self, field_name: &str) -> Option<Value> {
        if self.property_path.is_empty() {
            self.data_set.resolve_property(self.schema_set, self.object_id, field_name)
        } else {
            if field_name.is_empty() {
                self.data_set.resolve_property(self.schema_set, self.object_id, &format!("{}", self.property_path))
            } else {
                self.data_set.resolve_property(self.schema_set, self.object_id, &format!("{}.{}", self.property_path, field_name))
            }
        }
    }

    pub fn resolve_is_null(&self, field_name: &str) -> Option<bool> {
        if self.property_path.is_empty() {
            self.data_set.resolve_is_null(self.schema_set, self.object_id, field_name)
        } else {
            if field_name.is_empty() {
                self.data_set.resolve_is_null(self.schema_set, self.object_id, &format!("{}", self.property_path))
            } else {
                self.data_set.resolve_is_null(self.schema_set, self.object_id, &format!("{}.{}", self.property_path, field_name))
            }
        }
    }

    pub fn resolve_dynamic_array(&self, field_name: &str) -> Box<[Uuid]> {
        if self.property_path.is_empty() {
            self.data_set.resolve_dynamic_array(self.schema_set, self.object_id, field_name)
        } else {
            if field_name.is_empty() {
                self.data_set.resolve_dynamic_array(self.schema_set, self.object_id, &format!("{}", self.property_path))
            } else {
                self.data_set.resolve_dynamic_array(self.schema_set, self.object_id, &format!("{}.{}", self.property_path, field_name))
            }
        }
    }

    // pub fn schema(&self, field_name: &str) {
    //     self.object_schema.find_property_schema()
    // }
}


pub struct DataSetViewMut<'a> {
    data_set: &'a mut DataSet,
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
        if self.property_path.is_empty() {
            self.data_set.set_property_override(self.schema_set, self.object_id, field_name, value)
        } else {
            if field_name.is_empty() {
                self.data_set.set_property_override(self.schema_set, self.object_id, &format!("{}", self.property_path), value)
            } else {
                self.data_set.set_property_override(self.schema_set, self.object_id, &format!("{}.{}", self.property_path, field_name), value)
            }
        }
    }
}