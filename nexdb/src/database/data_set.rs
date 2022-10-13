use std::str::FromStr;
use uuid::Uuid;
use crate::{HashMap, HashMapKeys, HashSet, HashSetIter, ObjectId, Schema, SchemaFingerprint, SchemaNamedType, SchemaRecord, Value};
use crate::database::schema_set::SchemaSet;
use crate::value::PropertyValue;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NullOverride {
    SetNull,
    SetNonNull,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OverrideBehavior {
    Append,
    Replace,
}

pub struct DataObjectDelta {

}

#[derive(Clone)]
pub struct DataObjectInfo {
    pub(crate) schema: SchemaRecord, // Will always be a SchemaRecord
    pub(crate) prototype: Option<ObjectId>,
    pub(crate) properties: HashMap<String, Value>,
    pub(crate) property_null_overrides: HashMap<String, NullOverride>,
    pub(crate) properties_in_replace_mode: HashSet<String>,
    pub(crate) dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
}

#[derive(Default)]
pub struct DataSet {
    objects: HashMap<ObjectId, DataObjectInfo>,
}

impl DataSet {
    pub fn all_objects<'a>(&'a self) -> HashMapKeys<'a, ObjectId, DataObjectInfo> {
        self.objects.keys()
    }

    pub(crate) fn objects(&self) -> &HashMap<ObjectId, DataObjectInfo> {
        &self.objects
    }

    pub(crate) fn insert_object(
        &mut self,
        obj_info: DataObjectInfo,
    ) -> ObjectId {
        let id = ObjectId(uuid::Uuid::new_v4().as_u128());
        let old = self.objects.insert(id, obj_info);
        assert!(old.is_none());

        id
    }

    pub fn new_object(
        &mut self,
        schema: &SchemaRecord,
    ) -> ObjectId {
        let obj = DataObjectInfo {
            schema: schema.clone(),
            prototype: None,
            properties: Default::default(),
            property_null_overrides: Default::default(),
            properties_in_replace_mode: Default::default(),
            dynamic_array_entries: Default::default(),
        };

        self.insert_object(obj)
    }

    pub fn new_object_from_prototype(
        &mut self,
        prototype: ObjectId,
    ) -> ObjectId {
        let prototype_info = self.objects.get(&prototype).unwrap();
        let obj = DataObjectInfo {
            schema: prototype_info.schema.clone(),
            prototype: Some(prototype),
            properties: Default::default(),
            property_null_overrides: Default::default(),
            properties_in_replace_mode: Default::default(),
            dynamic_array_entries: Default::default(),
        };

        self.insert_object(obj)
    }

    pub(crate) fn restore_object(
        &mut self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        prototype: Option<ObjectId>,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        properties_in_replace_mode: HashSet<String>,
        dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
    ) {
        let schema = schema_set.schemas().get(&schema).unwrap();
        let schema_record = schema.as_record().cloned().unwrap();
        let obj = DataObjectInfo {
            schema: schema_record,
            prototype,
            properties,
            property_null_overrides,
            properties_in_replace_mode,
            dynamic_array_entries,
        };

        self.objects.insert(object_id, obj);
    }

    // pub(crate) fn restore_object(
    //     &mut self,
    //     object_id: ObjectId,
    //     schema: SchemaFingerprint,
    //     schema_name: String,
    //     prototype: Option<ObjectId>,
    // ) {
    //     let schema = self.schemas.get(&schema).unwrap();
    //     let obj = DatabaseObjectInfo {
    //         schema: schema.clone(),
    //         prototype,
    //         properties: Default::default(),
    //         property_null_overrides: Default::default(),
    //         properties_in_replace_mode: Default::default(),
    //         dynamic_array_entries: Default::default(),
    //     };
    //
    //     self.insert_object(obj);
    // }

    pub fn object_prototype(
        &self,
        object_id: ObjectId
    ) -> Option<ObjectId> {
        let object = self.objects.get(&object_id).unwrap();
        object.prototype
    }

    pub fn object_schema(
        &self,
        object_id: ObjectId,
    ) -> Option<&SchemaRecord> {
        self.objects.get(&object_id).map(|x| &x.schema)
    }

    // pub(crate) fn property_schema(
    //     schema: &SchemaNamedType,
    //     path: impl AsRef<str>,
    //     named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    // ) -> Option<Schema> {
    //     let mut schema = Schema::NamedType(schema.fingerprint());
    //
    //     //TODO: Escape map keys (and probably avoid path strings anyways)
    //     let split_path = path.as_ref().split(".");
    //
    //     // Iterate the path segments to find
    //     for path_segment in split_path {
    //         let s = schema.find_property_schema(path_segment, named_types);
    //         if let Some(s) = s {
    //             schema = s.clone();
    //         } else {
    //             return None;
    //         }
    //     }
    //
    //     Some(schema)
    // }

    fn truncate_property_path(
        path: impl AsRef<str>,
        max_segment_count: usize,
    ) -> String {
        let mut shortened_path = String::default();
        //TODO: Escape map keys (and probably avoid path strings anyways)
        let split_path = path.as_ref().split(".");
        for (i, path_segment) in split_path.enumerate() {
            if i > max_segment_count {
                break;
            }

            if i == 0 {
                shortened_path = path_segment.to_string();
            } else {
                shortened_path = format!("{}.{}", shortened_path, path_segment);
            }
        }

        shortened_path
    }

    fn property_schema_and_path_ancestors_to_check<'a>(
        named_type: &'a SchemaRecord,
        path: impl AsRef<str>,
        named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
        nullable_ancestors: &mut Vec<String>,
        dynamic_array_ancestors: &mut Vec<String>,
        map_ancestors: &mut Vec<String>,
        accessed_dynamic_array_keys: &mut Vec<(String, String)>,
    ) -> Option<Schema> {
        let mut schema = Schema::NamedType(named_type.fingerprint());

        //TODO: Escape map keys (and probably avoid path strings anyways)
        let split_path: Vec<_> = path.as_ref().split(".").collect();

        let mut parent_is_dynamic_array = false;

        for (i, path_segment) in split_path[0..split_path.len() - 1].iter().enumerate() {
            //.as_ref().split(".").enumerate() {
            let s = schema.find_field_schema(path_segment, named_types)?;
            //println!("  next schema {:?}", s);

            // current path needs to be verified as existing
            if parent_is_dynamic_array {
                accessed_dynamic_array_keys.push((
                    Self::truncate_property_path(path.as_ref(), i - 1),
                    path_segment.to_string(),
                ));
            }

            parent_is_dynamic_array = false;

            //if let Some(s) = s {
            // If it's nullable, we need to check for value being null before looking up the prototype chain
            // If it's a map or dynamic array, we need to check for append mode before looking up the prototype chain
            match s {
                Schema::Nullable(_) => {
                    let mut shortened_path = Self::truncate_property_path(path.as_ref(), i);
                    nullable_ancestors.push(shortened_path);
                }
                Schema::DynamicArray(_) => {
                    let mut shortened_path = Self::truncate_property_path(path.as_ref(), i);
                    dynamic_array_ancestors.push(shortened_path.clone());

                    parent_is_dynamic_array = true;
                }
                Schema::Map(_) => {
                    let mut shortened_path = Self::truncate_property_path(path.as_ref(), i);
                    map_ancestors.push(shortened_path);
                }
                _ => {}
            }

            schema = s.clone();
            //} else {
            //    return None;
            //}
        }

        if let Some(last_path_segment) = split_path.last() {
            schema = schema
                .find_field_schema(split_path.last().unwrap(), named_types)?
                .clone();
        }

        Some(schema)
    }

    pub fn get_null_override(
        &self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<NullOverride> {
        let object = self.objects.get(&object_id).unwrap();
        let property_schema = object.schema.find_property_schema(&path, schema_set.schemas()).unwrap();

        if property_schema.is_nullable() {
            object.property_null_overrides.get(path.as_ref()).copied()
        } else {
            None
        }
    }

    pub fn set_null_override(
        &mut self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) {
        let mut object = self.objects.get_mut(&object_id).unwrap();
        let property_schema = object.schema.find_property_schema(&path, schema_set.schemas()).unwrap();

        if property_schema.is_nullable() {
            object.property_null_overrides
                .insert(path.as_ref().to_string(), null_override);
        }
    }

    pub fn remove_null_override(
        &mut self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        let mut object = self.objects.get_mut(&object_id).unwrap();
        let property_schema = object.schema.find_property_schema(&path, schema_set.schemas()).unwrap();

        if property_schema.is_nullable() {
            object.property_null_overrides.remove(path.as_ref());
        }
    }

    // None return means the property can't be resolved, maybe because something higher in
    // property hierarchy is null or non-existing
    pub fn resolve_is_null(
        &self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<bool> {
        let mut object_schema = self.object_schema(object_id).unwrap();

        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        //TODO: Only allow getting values that exist, in particular, dynamic array overrides

        let property_schema = Self::property_schema_and_path_ancestors_to_check(
            object_schema,
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
            if self.resolve_is_null(schema_set, object_id, checked_property) != Some(false) {
                return None;
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, object_id, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return None;
            }
        }

        // Recursively look for a null override
        let mut prototype_id = Some(object_id);
        while let Some(prototype_id_iter) = prototype_id {
            let obj = self.objects.get(&prototype_id_iter).unwrap();

            if let Some(value) = obj.property_null_overrides.get(path.as_ref()) {
                return Some(*value == NullOverride::SetNull);
            }

            prototype_id = obj.prototype;
        }

        //TODO: Return schema default value
        Some(true)
    }

    pub fn has_property_override(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> bool {
        self.get_property_override(object_id, path).is_some()
    }

    // Just gets if this object has a property without checking prototype chain for fallback or returning a default
    // Returning none means it is not overridden
    pub fn get_property_override(
        &self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<&Value> {
        let object = self.objects.get(&object_id).unwrap();
        object.properties.get(path.as_ref())
    }

    // Just sets a property on this object, making it overridden, or replacing the existing override
    pub fn set_property_override(
        &mut self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
        value: Value,
    ) -> bool {
        let mut object_schema = self.object_schema(object_id).unwrap();
        let mut property_schema =
            object_schema.find_property_schema(&path, schema_set.schemas()).unwrap();

        //TODO: Should we check for null in path ancestors?
        //TODO: Only allow setting on values that exist, in particular, dynamic array overrides
        if !value.matches_schema(&property_schema, schema_set.schemas()) {
            log::debug!(
                "Value {:?} doesn't match schema {:?}",
                value, property_schema
            );
            return false;
        }

        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        let property_schema = Self::property_schema_and_path_ancestors_to_check(
            object_schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )
            .unwrap();

        for checked_property in &nullable_ancestors {
            if self.resolve_is_null(schema_set, object_id, checked_property) != Some(false) {
                return false;
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, object_id, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return false;
            }
        }

        let obj = self.objects.get_mut(&object_id).unwrap();
        obj.properties.insert(path.as_ref().to_string(), value);
        true
    }

    pub fn remove_property_override(
        &mut self,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        let mut object = self.objects.get_mut(&object_id).unwrap();
        object.properties.remove(path.as_ref())
    }

    pub fn apply_property_override_to_prototype(
        &mut self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        let object = self.objects.get(&object_id).unwrap();
        let prototype_id = object.prototype;

        if let Some(prototype_id) = prototype_id {
            let v = self.remove_property_override(object_id, path.as_ref());
            if let Some(v) = v {
                self.set_property_override(schema_set, prototype_id, path, v);
            }
        }
    }

    pub fn resolve_property(
        &self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        let mut object_schema = self.object_schema(object_id).unwrap();

        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        //TODO: Only allow getting values that exist, in particular, dynamic array overrides

        let property_schema = Self::property_schema_and_path_ancestors_to_check(
            object_schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )
            .unwrap();

        for checked_property in &nullable_ancestors {
            if self.resolve_is_null(schema_set, object_id, checked_property) != Some(false) {
                return None;
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, object_id, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return None;
            }
        }

        let mut prototype_id = Some(object_id);
        while let Some(prototype_id_iter) = prototype_id {
            let obj = self.objects.get(&prototype_id_iter).unwrap();

            if let Some(value) = obj.properties.get(path.as_ref()) {
                return Some(value.clone());
            }

            prototype_id = obj.prototype;
        }

        //TODO: Return schema default value
        Some(Value::default_for_schema(&property_schema, schema_set.schemas()).clone())
    }

    pub fn get_dynamic_array_overrides<'a>(
        &'a self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<HashSetIter<'a, Uuid>> {
        let object = self.objects.get(&object_id).unwrap();
        let property_schema = object.schema.find_property_schema(&path, schema_set.schemas()).unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("get_dynamic_array_overrides only allowed on dynamic arrays");
        }

        let object = self.objects.get(&object_id).unwrap();
        if let Some(overrides) = object.dynamic_array_entries.get(path.as_ref()) {
            Some(overrides.iter())
        } else {
            None
        }
    }

    pub fn add_dynamic_array_override(
        &mut self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Uuid {
        let mut object = self.objects.get_mut(&object_id).unwrap();
        let property_schema = object.schema.find_property_schema(&path, schema_set.schemas()).unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("add_dynamic_array_override only allowed on dynamic arrays");
        }

        let entry = object
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
        object_id: ObjectId,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) {
        let object = self.objects.get_mut(&object_id).unwrap();
        let property_schema = object.schema.find_property_schema(&path, schema_set.schemas()).unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("remove_dynamic_array_override only allowed on dynamic arrays");
        }

        if let Some(override_list) = object.dynamic_array_entries.get_mut(path.as_ref()) {
            if !override_list.remove(&element_id) {
                panic!("Could not find override")
            }
        }
    }

    pub fn do_resolve_dynamic_array(
        &self,
        object_id: ObjectId,
        path: &str,
        nullable_ancestors: &Vec<String>,
        dynamic_array_ancestors: &Vec<String>,
        map_ancestors: &Vec<String>,
        accessed_dynamic_array_keys: &Vec<(String, String)>,
        resolved_entries: &mut Vec<Uuid>,
    ) {
        let obj = self.objects.get(&object_id).unwrap();

        // See if any properties in the path ancestry are replacing parent data
        let mut check_parents = true;

        for checked_property in dynamic_array_ancestors {
            if obj.properties_in_replace_mode.contains(checked_property) {
                check_parents = false;
            }
        }

        for checked_property in map_ancestors {
            if obj.properties_in_replace_mode.contains(checked_property) {
                check_parents = false;
            }
        }

        // Still need to check *this* property in addition to ancestors
        if obj.properties_in_replace_mode.contains(path) {
            check_parents = false;
        }

        // If we do not replace parent data, resolve it now so we can append to it
        if check_parents {
            if let Some(prototype) = obj.prototype {
                self.do_resolve_dynamic_array(
                    prototype,
                    path,
                    nullable_ancestors,
                    dynamic_array_ancestors,
                    map_ancestors,
                    accessed_dynamic_array_keys,
                    resolved_entries,
                );
            }
        }

        if let Some(entries) = obj.dynamic_array_entries.get(path) {
            for entry in entries {
                resolved_entries.push(*entry);
            }
        }
    }

    pub fn resolve_dynamic_array(
        &self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Box<[Uuid]> {
        let mut object_schema = self.object_schema(object_id).unwrap();

        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        let property_schema = Self::property_schema_and_path_ancestors_to_check(
            object_schema,
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
            if self.resolve_is_null(schema_set, object_id, checked_property) != Some(false) {
                return vec![].into_boxed_slice();
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, object_id, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return vec![].into_boxed_slice();
            }
        }

        let mut resolved_entries = vec![];
        self.do_resolve_dynamic_array(
            object_id,
            path.as_ref(),
            &nullable_ancestors,
            &dynamic_array_ancestors,
            &map_ancestors,
            &accessed_dynamic_array_keys,
            &mut resolved_entries,
        );
        resolved_entries.into_boxed_slice()
    }

    pub fn get_override_behavior(
        &self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> OverrideBehavior {
        let object = self.objects.get(&object_id).unwrap();
        let property_schema = object.schema.find_property_schema(&path, schema_set.schemas()).unwrap();

        match property_schema {
            Schema::DynamicArray(_) | Schema::Map(_) => {
                if object.properties_in_replace_mode.contains(path.as_ref()) {
                    OverrideBehavior::Replace
                } else {
                    OverrideBehavior::Append
                }
            }
            _ => OverrideBehavior::Replace,
        }
    }

    pub fn set_override_behavior(
        &mut self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) {
        let object = self.objects.get_mut(&object_id).unwrap();
        let property_schema = object.schema.find_property_schema(&path, schema_set.schemas()).unwrap();

        match property_schema {
            Schema::DynamicArray(_) | Schema::Map(_) => {
                let _ = match behavior {
                    OverrideBehavior::Append => {
                        object.properties_in_replace_mode.remove(path.as_ref())
                    }
                    OverrideBehavior::Replace => object
                        .properties_in_replace_mode
                        .insert(path.as_ref().to_string()),
                };
            }
            _ => panic!("unexpected schema type"),
        }
    }

    pub fn copy_from(
        &mut self,
        other: &DataSet,
        object_id: ObjectId
    ) {
        let object = other.objects.get(&object_id).cloned().unwrap();
        self.objects.insert(object_id, object);
    }
}

pub struct DynamicArrayEntryDelta {
    key: String,
    add: Vec<Uuid>,
    remove: Vec<Uuid>
}



#[derive(Default)]
pub struct ObjectDiff {
    set_prototype: Option<Option<ObjectId>>,
    set_properties: Vec<(String, PropertyValue)>,
    remove_properties: Vec<String>,
    set_null_overrides: Vec<(String, NullOverride)>,
    remove_null_overrides: Vec<String>,
    add_properties_in_replace_mode: Vec<String>,
    remove_properties_in_replace_mode: Vec<String>,
    dynamic_array_entry_deltas: Vec<DynamicArrayEntryDelta>,
}

impl ObjectDiff {
    pub fn has_change(&self) -> bool {
        self.set_prototype.is_some() ||
            !self.set_properties.is_empty() ||
            !self.remove_properties.is_empty() ||
            !self.set_null_overrides.is_empty() ||
            !self.remove_null_overrides.is_empty() ||
            !self.add_properties_in_replace_mode.is_empty() ||
            !self.add_properties_in_replace_mode.is_empty() ||
            !self.remove_properties_in_replace_mode.is_empty() ||
            !self.dynamic_array_entry_deltas.is_empty()
    }
}

pub struct ObjectDiffSet {
    pub apply_diff: ObjectDiff,
    pub revert_diff: ObjectDiff,
}

impl ObjectDiffSet {
    pub fn has_change(&self) -> bool {
        // assume if apply has no changes, neither does revert
        self.apply_diff.has_change()
    }

    pub fn diff_objects(
        before_data_set: &DataSet,
        before_object_id: ObjectId,
        after_data_set: &DataSet,
        after_object_id: ObjectId,
    ) -> Self {
        let before_obj = before_data_set.objects.get(&before_object_id).unwrap();
        let after_obj = after_data_set.objects.get(&after_object_id).unwrap();

        assert_eq!(before_obj.schema.fingerprint(), after_obj.schema.fingerprint());

        let mut apply_diff = ObjectDiff::default();
        let mut revert_diff = ObjectDiff::default();

        //
        // Prototype
        //
        if before_obj.prototype != after_obj.prototype {
            apply_diff.set_prototype = Some(after_obj.prototype);
            revert_diff.set_prototype = Some(before_obj.prototype);
        }

        //
        // Properties
        //
        for (key, before_value) in &before_obj.properties {
            if let Some(after_value) = after_obj.properties.get(key) {
                if !Value::are_matching_property_values(before_value, after_value) {
                    // Value was changed
                    apply_diff.set_properties.push((key.clone(), after_value.as_property_value().unwrap()));
                    revert_diff.set_properties.push((key.clone(), before_value.as_property_value().unwrap()));
                } else {
                    // No change
                }
            } else {
                // Property was removed
                apply_diff.remove_properties.push(key.clone());
                revert_diff.set_properties.push((key.clone(), before_value.as_property_value().unwrap()));
            }
        }

        for (key, after_value) in &after_obj.properties {
            if !before_obj.properties.contains_key(key) {
                // Property was added
                apply_diff.set_properties.push((key.clone(), after_value.as_property_value().unwrap()));
                revert_diff.remove_properties.push(key.clone());
            }
        }


        //
        // Null Overrides
        //
        for (key, &before_value) in &before_obj.property_null_overrides {
            if let Some(after_value) = after_obj.property_null_overrides.get(key).copied() {
                if before_value != after_value {
                    // Value was changed
                    apply_diff.set_null_overrides.push((key.clone(), after_value));
                    revert_diff.set_null_overrides.push((key.clone(), before_value));
                } else {
                    // No change
                }
            } else {
                // Property was removed
                apply_diff.remove_null_overrides.push(key.clone());
                revert_diff.set_null_overrides.push((key.clone(), before_value));
            }
        }

        for (key, &after_value) in &after_obj.property_null_overrides {
            if !before_obj.property_null_overrides.contains_key(key) {
                // Property was added
                apply_diff.set_null_overrides.push((key.clone(), after_value));
                revert_diff.remove_properties.push(key.clone());
            }
        }

        //
        // Properties in replace mode
        //
        for replace_mode_property in &before_obj.properties_in_replace_mode {
            if !after_obj.properties_in_replace_mode.contains(replace_mode_property) {
                // Replace mode disabled
                apply_diff.remove_properties_in_replace_mode.push(replace_mode_property.clone());
                revert_diff.add_properties_in_replace_mode.push(replace_mode_property.clone());
            }
        }

        for replace_mode_property in &after_obj.properties_in_replace_mode {
            if !before_obj.properties_in_replace_mode.contains(replace_mode_property) {
                // Replace mode enabled
                apply_diff.add_properties_in_replace_mode.push(replace_mode_property.clone());
                revert_diff.remove_properties_in_replace_mode.push(replace_mode_property.clone());
            }
        }

        //
        // Dynamic Array Entries
        //
        for (key, old_entries) in &before_obj.dynamic_array_entries {
            if let Some(new_entries) = after_obj.dynamic_array_entries.get(key) {
                // Diff the hashes
                let mut added_entries = Vec::default();
                let mut removed_entries = Vec::default();

                for old_entry in old_entries {
                    if !new_entries.contains(&old_entry) {
                        removed_entries.push(*old_entry);
                    }
                }

                for new_entry in new_entries {
                    if !old_entries.contains(&new_entry) {
                        added_entries.push(*new_entry);
                    }
                }

                if !added_entries.is_empty() || !removed_entries.is_empty() {
                    apply_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: added_entries.clone(),
                        remove: removed_entries.clone(),
                    });
                    revert_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: removed_entries,
                        remove: added_entries,
                    });
                }
            } else {
                if !old_entries.is_empty() {
                    // All of them were removed
                    apply_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: Default::default(),
                        remove: old_entries.iter().copied().collect(),
                    });
                    revert_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: old_entries.iter().copied().collect(),
                        remove: Default::default(),
                    });
                }
            }
        }

        for (key, new_entries) in &after_obj.dynamic_array_entries {
            if !new_entries.is_empty() {
                if !before_obj.dynamic_array_entries.contains_key(key) {
                    // All of them were added
                    apply_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: new_entries.iter().copied().collect(),
                        remove: Default::default(),
                    });
                    revert_diff.dynamic_array_entry_deltas.push(DynamicArrayEntryDelta {
                        key: key.clone(),
                        add: Default::default(),
                        remove: new_entries.iter().copied().collect(),
                    });
                }
            }
        }

        ObjectDiffSet {
            apply_diff,
            revert_diff,
        }
    }
}