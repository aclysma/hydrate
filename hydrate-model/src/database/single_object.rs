use crate::{DataSetError, DataSetResult, HashMap, HashMapKeys, HashSet, HashSetIter, Schema, SchemaFingerprint, SchemaNamedType, SchemaRecord, Value};
use crate::{NullOverride, SchemaSet};
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::string::ToString;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SingleObject {
    pub(crate) schema: SchemaRecord,
    pub(crate) properties: HashMap<String, Value>,
    pub(crate) property_null_overrides: HashMap<String, NullOverride>,
    pub(crate) dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
}

impl Hash for SingleObject {
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    ) {
        let schema = &self.schema;

        use std::hash::{Hash, Hasher};

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

        // dynamic_array_entries
        let mut dynamic_array_entries_hash = 0;
        for (key, value) in &self.dynamic_array_entries {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            key.hash(&mut inner_hasher);

            let mut uuid_set_hash = 0;
            for id in value {
                let mut inner_hasher2 = siphasher::sip::SipHasher::default();
                id.hash(&mut inner_hasher2);
                uuid_set_hash = uuid_set_hash ^ inner_hasher2.finish();
            }
            uuid_set_hash.hash(&mut inner_hasher);

            dynamic_array_entries_hash = dynamic_array_entries_hash ^ inner_hasher.finish();
        }
        dynamic_array_entries_hash.hash(state);
    }
}

impl SingleObject {
    pub fn new(schema: &SchemaRecord) -> Self {
        SingleObject {
            schema: schema.clone(),
            properties: Default::default(),
            property_null_overrides: Default::default(),
            dynamic_array_entries: Default::default(),
        }
    }

    pub(crate) fn restore(
        schema_set: &SchemaSet,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
    ) -> SingleObject {
        let schema = schema_set.schemas().get(&schema).unwrap();
        let schema_record = schema.as_record().cloned().unwrap();
        SingleObject {
            schema: schema_record,
            properties,
            property_null_overrides,
            dynamic_array_entries,
        }
    }

    pub fn schema(&self) -> &SchemaRecord {
        &self.schema
    }

    pub fn get_null_override(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> Option<NullOverride> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

        if property_schema.is_nullable() {
            self.property_null_overrides.get(path.as_ref()).copied()
        } else {
            None
        }
    }

    pub fn set_null_override(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

        if property_schema.is_nullable() {
            self.property_null_overrides
                .insert(path.as_ref().to_string(), null_override);
        }
    }

    pub fn remove_null_override(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

        if property_schema.is_nullable() {
            self.property_null_overrides.remove(path.as_ref());
        }
    }

    // None return means the property can't be resolved, maybe because something higher in
    // property hierarchy is null or non-existing
    pub fn resolve_is_null(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> Option<bool> {
        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        //TODO: Only allow getting values that exist, in particular, dynamic array overrides

        let property_schema = super::property_schema_and_path_ancestors_to_check(
            &self.schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )
        .unwrap();

        if !property_schema.is_nullable() {
            return None;
        }

        for checked_property in &nullable_ancestors {
            if self.resolve_is_null(schema_set, checked_property) != Some(false) {
                return None;
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return None;
            }
        }

        // Recursively look for a null override
        if let Some(value) = self.property_null_overrides.get(path.as_ref()) {
            return Some(*value == NullOverride::SetNull);
        }

        //TODO: Return schema default value
        Some(true)
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
        value: Value,
    ) -> DataSetResult<()> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

        //TODO: Should we check for null in path ancestors?
        //TODO: Only allow setting on values that exist, in particular, dynamic array overrides
        if !value.matches_schema(&property_schema, schema_set.schemas()) {
            log::debug!(
                "Value {:?} doesn't match schema {:?}",
                value,
                property_schema
            );
            return Err(DataSetError::ValueDoesNotMatchSchema);
        }

        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        let _property_schema = super::property_schema_and_path_ancestors_to_check(
            &self.schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )
        .unwrap();

        for checked_property in &nullable_ancestors {
            if self.resolve_is_null(schema_set, checked_property) != Some(false) {
                return Err(DataSetError::PathParentIsNull);
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return Err(DataSetError::PathDynamicArrayEntryDoesNotExist);
            }
        }

        self.properties.insert(path.as_ref().to_string(), value);
        Ok(())
    }

    pub fn remove_property_override(
        &mut self,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        self.properties.remove(path.as_ref())
    }

    pub fn resolve_property(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> Option<&Value> {
        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        //TODO: Only allow getting values that exist, in particular, dynamic array overrides

        let property_schema = super::property_schema_and_path_ancestors_to_check(
            &self.schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )
        .unwrap();

        for checked_property in &nullable_ancestors {
            if self.resolve_is_null(schema_set, checked_property) != Some(false) {
                return None;
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return None;
            }
        }

        if let Some(value) = self.properties.get(path.as_ref()) {
            return Some(value);
        }

        //TODO: Return schema default value
        Some(Value::default_for_schema(&property_schema, schema_set.schemas()))
    }

    pub fn get_dynamic_array_overrides(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> Option<HashSetIter<Uuid>> {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("get_dynamic_array_overrides only allowed on dynamic arrays");
        }

        if let Some(overrides) = self.dynamic_array_entries.get(path.as_ref()) {
            Some(overrides.iter())
        } else {
            None
        }
    }

    pub fn add_dynamic_array_override(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> Uuid {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("add_dynamic_array_override only allowed on dynamic arrays");
        }

        let entry = self
            .dynamic_array_entries
            .entry(path.as_ref().to_string())
            .or_insert(Default::default());
        let new_uuid = Uuid::new_v4();
        let already_existed = !entry.insert(new_uuid);
        if already_existed {
            panic!("Already existed")
        }
        new_uuid
    }

    pub fn remove_dynamic_array_override(
        &mut self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) {
        let property_schema = self
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("remove_dynamic_array_override only allowed on dynamic arrays");
        }

        if let Some(override_list) = self.dynamic_array_entries.get_mut(path.as_ref()) {
            if !override_list.remove(&element_id) {
                panic!("Could not find override")
            }
        }
    }

    pub fn do_resolve_dynamic_array(
        &self,
        path: &str,
        resolved_entries: &mut Vec<Uuid>,
    ) {
        if let Some(entries) = self.dynamic_array_entries.get(path) {
            for entry in entries {
                resolved_entries.push(*entry);
            }
        }
    }

    pub fn resolve_dynamic_array(
        &self,
        schema_set: &SchemaSet,
        path: impl AsRef<str>,
    ) -> Box<[Uuid]> {
        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        let property_schema = super::property_schema_and_path_ancestors_to_check(
            &self.schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        );
        if property_schema.is_none() {
            panic!("dynamic array not found");
        }

        for checked_property in &nullable_ancestors {
            if self.resolve_is_null(schema_set, checked_property) != Some(false) {
                return vec![].into_boxed_slice();
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return vec![].into_boxed_slice();
            }
        }

        let mut resolved_entries = vec![];
        self.do_resolve_dynamic_array(path.as_ref(), &mut resolved_entries);
        resolved_entries.into_boxed_slice()
    }
}
