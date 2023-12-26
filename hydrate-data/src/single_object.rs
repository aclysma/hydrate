use crate::{
    DataSetError, DataSetResult, HashMap, OrderedSet, SchemaFingerprint, SchemaRecord, Value,
};
use crate::{NullOverride, SchemaSet};
use hydrate_schema::Schema;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::string::ToString;
use uuid::Uuid;

/// A simplified container of data. Can be used to produce a set of properties and be merged into
/// a data set later, or be serialized by itself. Still support schema migration.
#[derive(Clone, Debug)]
pub struct SingleObject {
    schema: SchemaRecord,
    properties: HashMap<String, Value>,
    property_null_overrides: HashMap<String, NullOverride>,
    dynamic_collection_entries: HashMap<String, OrderedSet<Uuid>>,
}

impl Hash for SingleObject {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        let schema = &self.schema;

        schema.fingerprint().hash(state);

        // properties
        let mut properties_hash = 0;
        for (key, value) in &self.properties {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            key.hash(&mut inner_hasher);
            value.hash(&mut inner_hasher);
            properties_hash = properties_hash ^ inner_hasher.finish();
        }
        properties_hash.hash(state);

        // property_null_overrides
        let mut property_null_overrides_hash = 0;
        for (key, value) in &self.property_null_overrides {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            key.hash(&mut inner_hasher);
            value.hash(&mut inner_hasher);
            property_null_overrides_hash = property_null_overrides_hash ^ inner_hasher.finish();
        }
        property_null_overrides_hash.hash(state);

        // dynamic_collection_entries
        let mut dynamic_collection_entries_hash = 0;
        for (key, value) in &self.dynamic_collection_entries {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            key.hash(&mut inner_hasher);

            let mut uuid_set_hash = 0;
            for id in value {
                let mut inner_hasher2 = siphasher::sip::SipHasher::default();
                id.hash(&mut inner_hasher2);
                uuid_set_hash = uuid_set_hash ^ inner_hasher2.finish();
            }
            uuid_set_hash.hash(&mut inner_hasher);

            dynamic_collection_entries_hash =
                dynamic_collection_entries_hash ^ inner_hasher.finish();
        }
        dynamic_collection_entries_hash.hash(state);
    }
}

impl SingleObject {
    pub fn new(schema: &SchemaRecord) -> Self {
        SingleObject {
            schema: schema.clone(),
            properties: Default::default(),
            property_null_overrides: Default::default(),
            dynamic_collection_entries: Default::default(),
        }
    }

    pub fn restore(
        schema_set: &SchemaSet,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        dynamic_collection_entries: HashMap<String, OrderedSet<Uuid>>,
    ) -> SingleObject {
        let schema = schema_set.schemas().get(&schema).unwrap();
        let schema_record = schema.as_record().cloned().unwrap();
        SingleObject {
            schema: schema_record,
            properties,
            property_null_overrides,
            dynamic_collection_entries,
        }
    }

    pub fn schema(&self) -> &SchemaRecord {
        &self.schema
    }

    pub fn properties(&self) -> &HashMap<String, Value> {
        &self.properties
    }

    pub fn property_null_overrides(&self) -> &HashMap<String, NullOverride> {
        &self.property_null_overrides
    }

    pub fn dynamic_collection_entries(&self) -> &HashMap<String, OrderedSet<Uuid>> {
        &self.dynamic_collection_entries
    }

    /// Gets if the property has a null override associated with it An error will be returned if
    /// the schema doesn't exist or if this field is not nullable
    pub fn get_null_override(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if property_schema.is_nullable() {
            // Not existing in the map implies that it is unset
            Ok(self
                .property_null_overrides
                .get(path.as_ref())
                .copied()
                .unwrap_or(NullOverride::Unset))
        } else {
            Err(DataSetError::InvalidSchema)?
        }
    }

    pub fn set_null_override(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) -> DataSetResult<()> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if property_schema.is_nullable() {
            if null_override != NullOverride::Unset {
                self.property_null_overrides
                    .insert(path.as_ref().to_string(), null_override);
            } else {
                self.property_null_overrides.remove(path.as_ref());
            }
            Ok(())
        } else {
            Err(DataSetError::InvalidSchema)?
        }
    }

    fn validate_parent_paths(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<Schema> {
        // Contains the path segments that we need to check for being null
        let mut accessed_nullable_keys = vec![];
        // The containers we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];
        let mut accessed_static_array_keys = vec![];
        let mut accessed_map_keys = vec![];

        //TODO: Only allow getting values that exist, in particular, dynamic array overrides

        let property_schema = super::property_schema_and_path_ancestors_to_check(
            &self.schema,
            &path,
            schema_set.schemas(),
            &mut accessed_nullable_keys,
            &mut accessed_dynamic_array_keys,
            &mut accessed_static_array_keys,
            &mut accessed_map_keys,
        )?;

        // See if this field was contained in any nullables. If any of those were null, return None.
        for checked_property in &accessed_nullable_keys {
            if self.resolve_null_override(schema_set, checked_property)? != NullOverride::SetNonNull
            {
                return Err(DataSetError::PathParentIsNull)?;
            }
        }

        // See if this field was contained in a container. If any of those containers didn't contain
        // this property path, return None
        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_collection_entries =
                self.resolve_dynamic_array_entries(schema_set, path)?;
            if !dynamic_collection_entries
                .contains(&Uuid::from_str(key).map_err(|_| DataSetError::UuidParseError)?)
            {
                return Err(DataSetError::PathDynamicArrayEntryDoesNotExist)?;
            }
        }

        Ok(property_schema)
    }

    // None return means the property can't be resolved, maybe because something higher in
    // property hierarchy is null or non-existing
    pub fn resolve_null_override(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        let property_schema = self.validate_parent_paths(schema_set, path.as_ref())?;

        // This field is not nullable, return an error
        if !property_schema.is_nullable() {
            return Err(DataSetError::InvalidSchema)?;
        }

        Ok(self
            .property_null_overrides
            .get(path.as_ref())
            .copied()
            .unwrap_or(NullOverride::Unset))
    }

    pub fn has_property_override(
        &self,
        path: impl AsRef<str>,
    ) -> bool {
        self.get_property_override(path).is_some()
    }

    // Just gets if this object has a property without checking prototype chain for fallback or returning a default
    // Returning none means it is not overridden
    pub fn get_property_override(
        &self,
        path: impl AsRef<str>,
    ) -> Option<&Value> {
        self.properties.get(path.as_ref())
    }

    // Just sets a property on this object, making it overridden, or replacing the existing override
    pub fn set_property_override(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
        value: Option<Value>,
    ) -> DataSetResult<Option<Value>> {
        let property_schema = self.validate_parent_paths(schema_set, path.as_ref())?;

        if let Some(value) = &value {
            if !value.matches_schema(&property_schema, schema_set.schemas()) {
                log::debug!(
                    "Value {:?} doesn't match schema {:?}",
                    value,
                    property_schema
                );
                return Err(DataSetError::ValueDoesNotMatchSchema)?;
            }
        }

        let old_value = if let Some(value) = value {
            self.properties.insert(path.as_ref().to_string(), value)
        } else {
            self.properties.remove(path.as_ref())
        };
        Ok(old_value)
    }

    pub fn resolve_property<'a>(
        &'a self,
        schema_set: &'a SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<&'a Value> {
        let property_schema = self.validate_parent_paths(schema_set, path.as_ref())?;

        if let Some(value) = self.properties.get(path.as_ref()) {
            return Ok(value);
        }

        Ok(Value::default_for_schema(&property_schema, schema_set))
    }

    fn get_dynamic_collection_entries(
        &self,
        path: impl AsRef<str>,
    ) -> DataSetResult<std::slice::Iter<Uuid>> {
        if let Some(overrides) = self.dynamic_collection_entries.get(path.as_ref()) {
            Ok(overrides.iter())
        } else {
            Ok(std::slice::Iter::default())
        }
    }

    pub fn get_dynamic_array_entries(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<std::slice::Iter<Uuid>> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_dynamic_array() {
            return Err(DataSetError::InvalidSchema)?;
        }

        self.get_dynamic_collection_entries(path)
    }

    pub fn get_map_entries(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<std::slice::Iter<Uuid>> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_map() {
            return Err(DataSetError::InvalidSchema)?;
        }

        self.get_dynamic_collection_entries(path)
    }

    fn add_dynamic_collection_entry(
        &mut self,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        let entry = self
            .dynamic_collection_entries
            .entry(path.as_ref().to_string())
            .or_insert(Default::default());
        let new_uuid = Uuid::new_v4();
        let newly_inserted = entry.try_insert_at_end(new_uuid);
        if !newly_inserted {
            panic!("Created a new random UUID but it matched an existing UUID");
        }
        Ok(new_uuid)
    }

    pub fn add_dynamic_array_entry(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_dynamic_array() {
            return Err(DataSetError::InvalidSchema)?;
        }

        self.add_dynamic_collection_entry(path)
    }

    pub fn add_map_entry(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_map() {
            return Err(DataSetError::InvalidSchema)?;
        }

        self.add_dynamic_collection_entry(path)
    }

    pub fn insert_dynamic_array_entry(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
        index: usize,
        entry_uuid: Uuid,
    ) -> DataSetResult<()> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_dynamic_array() {
            return Err(DataSetError::InvalidSchema)?;
        }

        if !property_schema.is_dynamic_array() {
            return Err(DataSetError::InvalidSchema)?;
        }

        let entry = self
            .dynamic_collection_entries
            .entry(path.as_ref().to_string())
            .or_insert(Default::default());
        if entry.try_insert_at_position(index, entry_uuid) {
            Ok(())
        } else {
            Err(DataSetError::DuplicateEntryKey)?
        }
    }

    fn remove_dynamic_collection_entry(
        &mut self,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) -> DataSetResult<bool> {
        if let Some(override_list) = self.dynamic_collection_entries.get_mut(path.as_ref()) {
            // Return if the override existed or not
            let was_removed = override_list.remove(&element_id);
            Ok(was_removed)
        } else {
            // The override didn't exist
            Ok(false)
        }
    }

    pub fn remove_dynamic_array_entry(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) -> DataSetResult<bool> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_dynamic_array() {
            return Err(DataSetError::InvalidSchema)?;
        }

        self.remove_dynamic_collection_entry(path, element_id)
    }

    pub fn remove_map_entry(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) -> DataSetResult<bool> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_map() {
            return Err(DataSetError::InvalidSchema)?;
        }

        self.remove_dynamic_collection_entry(path, element_id)
    }

    fn do_resolve_dynamic_collection_entries(
        &self,
        path: &str,
        resolved_entries: &mut Vec<Uuid>,
    ) {
        if let Some(entries) = self.dynamic_collection_entries.get(path) {
            for entry in entries {
                resolved_entries.push(*entry);
            }
        }
    }

    pub fn resolve_dynamic_array_entries(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        let property_schema = self.validate_parent_paths(schema_set, path.as_ref())?;
        if !property_schema.is_dynamic_array() {
            return Err(DataSetError::InvalidSchema)?;
        }

        let mut resolved_entries = vec![];
        self.do_resolve_dynamic_collection_entries(path.as_ref(), &mut resolved_entries);
        Ok(resolved_entries.into_boxed_slice())
    }

    pub fn resolve_map_entries(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        let property_schema = self.validate_parent_paths(schema_set, path.as_ref())?;
        if !property_schema.is_map() {
            return Err(DataSetError::InvalidSchema)?;
        }

        let mut resolved_entries = vec![];
        self.do_resolve_dynamic_collection_entries(path.as_ref(), &mut resolved_entries);
        Ok(resolved_entries.into_boxed_slice())
    }
}
