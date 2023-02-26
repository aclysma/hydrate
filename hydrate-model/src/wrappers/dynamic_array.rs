use std::ops::{Deref, Index, IndexMut};
use uuid::Uuid;
use core::slice::SliceIndex;
use crate::{DataSetResult, DataSetView, DataSetViewMut, OverrideBehavior};

trait LoadStore {
    fn load(data_set_view: &mut DataSetView) -> Self;
    fn store(&self, data_set_view: &mut DataSetViewMut) -> DataSetResult<()>;
}

struct DynamicArray<T: LoadStore> {
    values: Vec<T>,
    entry_uuids: Vec<Uuid>,
    was_cleared: bool,
}

impl<T: LoadStore, I: SliceIndex<[T]>> Index<I> for DynamicArray<T> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.values[index]
    }
}

impl<T: LoadStore, I: SliceIndex<[T]>> IndexMut<I> for DynamicArray<T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.values[index]
    }
}

impl<T: LoadStore> DynamicArray<T> {
    pub fn load(data_set_view: &mut DataSetView) -> Self {
        let entry_uuids: Vec<Uuid> = data_set_view.resolve_dynamic_array("").into_iter().copied().collect();
        let mut values = Vec::with_capacity(entry_uuids.len());

        for entry_uuid in &entry_uuids {
            data_set_view.push_property_path(&entry_uuid.to_string());
            let value = T::load(data_set_view);
            values.push(value);
            data_set_view.pop_property_path();
        }

        DynamicArray {
            values,
            entry_uuids,
            was_cleared: false
        }
    }

    fn store(&self, data_set_view: &mut DataSetViewMut) -> DataSetResult<()> {
        if self.was_cleared {
            data_set_view.set_override_behavior("", OverrideBehavior::Replace);
        }

        for (entry_uuid, value) in self.entry_uuids.iter().zip(&self.values) {
            data_set_view.push_property_path(&entry_uuid.to_string());
            let result = value.store(data_set_view);
            data_set_view.pop_property_path();
            result?;
        }

        Ok(())
    }

    pub fn values(&self) -> &Vec<T> {
        &self.values
    }

    // This is a one-way operation, the only way to restore is to reload from the dataset
    pub fn clear(&mut self) {
        self.values.clear();
        self.entry_uuids.clear();
        self.was_cleared = false;
    }

    // This works in both append and replace mode. A null entry UUID means to add a new entry
    pub fn push(&mut self, value: T) {
        self.values.push(value);
        self.entry_uuids.push(Uuid::default());
    }
}
