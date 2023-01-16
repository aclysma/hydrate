use crate::{DataSet, ObjectId, SchemaSet, SingleObject};

pub trait SingleObjectEntry {
    fn copy_from_single_object(&mut self, single_object: &SingleObject);
}

pub trait DataSetEntry {
    fn from_data_set(object_id: ObjectId, data_set: &DataSet, schema: &SchemaSet) -> Self;
}

// trait ToSchema {
//
// }