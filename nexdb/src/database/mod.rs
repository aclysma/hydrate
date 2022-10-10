use super::schema::*;
use super::{HashMap, HashSet, ObjectId, SchemaFingerprint};
use std::io::BufRead;
use std::str::FromStr;
use uuid::Uuid;

use crate::{BufferId, HashMapKeys, SchemaLinker, SchemaLinkerResult};

pub mod value;
pub use value::Value;

#[cfg(test)]
mod tests;

// pub struct ArchivedObject {
//     object_id: Uuid,
//     schema: Uuid,
//     schema_name: String,
//     prototype: Uuid,
//     properties: HashMap<String, Value>
// }
//
// impl ArchivedObject {
//     pub fn archive(object_id: ObjectId, object: &DatabaseObjectInfo) -> ArchivedObject {
//         // Store simple properties
//         let mut properties: HashMap<String, Value> = Default::default();
//         for (key, value) in &object.properties {
//             properties.insert(key.clone(), value.clone());
//         }
//
//         // Store nullable status as a property
//
//         // Store replace mode as a property
//
//         // Store dynamic array entries as a property
//
//         ArchivedObject {
//             object_id: Uuid::from_u128(object_id.0),
//             schema: object.schema.fingerprint().as_uuid(),
//             schema_name: object.schema.name().to_string(),
//             prototype: object.prototype.map(|x| Uuid::from_u128(x.0)).unwrap(),
//             properties
//         }
//     }
// }


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NullOverride {
    SetNull,
    SetNonNull,
}

pub struct DatabaseObjectInfo {
    pub(crate) schema: SchemaNamedType, // Will always be a SchemaRecord
    pub(crate) prototype: Option<ObjectId>,
    pub(crate) properties: HashMap<String, Value>,
    pub(crate) property_null_overrides: HashMap<String, NullOverride>,
    pub(crate) properties_in_replace_mode: HashSet<String>,
    pub(crate) dynamic_array_entries: HashMap<String, Vec<Uuid>>,
}

//TODO: Delete unused property data when path ancestor is null or in replace mode

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OverrideBehavior {
    Append,
    Replace,
}

#[derive(Default)]
pub struct Database {
    schemas_by_name: HashMap<String, SchemaFingerprint>,
    schemas: HashMap<SchemaFingerprint, SchemaNamedType>,
    objects: HashMap<ObjectId, DatabaseObjectInfo>,
}

impl Database {
    pub fn all_objects<'a>(&'a self) -> HashMapKeys<'a, ObjectId, DatabaseObjectInfo> {
        self.objects.keys()
    }

    pub fn schemas(&self) -> &HashMap<SchemaFingerprint, SchemaNamedType> {
        &self.schemas
    }

    pub(crate) fn objects(&self) -> &HashMap<ObjectId, DatabaseObjectInfo> {
        &self.objects
    }

    pub fn add_linked_types(
        &mut self,
        mut linker: SchemaLinker,
    ) -> SchemaLinkerResult<()> {
        let linked = linker.finish()?;

        //TODO: check no name collisions and merge with DB

        for (k, v) in linked.schemas {
            let old = self.schemas.insert(k, v);
            //assert!(old.is_none());
            //TODO: Assert schemas are the same
        }

        for (k, v) in linked.schemas_by_name {
            let old = self.schemas_by_name.insert(k, v);
            assert!(old.is_none());
        }

        Ok(())
    }

    pub fn find_named_type(
        &self,
        name: impl AsRef<str>,
    ) -> Option<&SchemaNamedType> {
        self.schemas_by_name
            .get(name.as_ref())
            .map(|fingerprint| self.find_named_type_by_fingerprint(*fingerprint))
            .flatten()
    }

    pub fn find_named_type_by_fingerprint(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> Option<&SchemaNamedType> {
        self.schemas.get(&fingerprint)
    }

    pub fn default_value_for_schema(
        &self,
        schema: &Schema,
    ) -> Value {
        Value::default_for_schema(schema, &self.schemas)
    }

    pub(crate) fn insert_object(
        &mut self,
        obj_info: DatabaseObjectInfo,
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
        let obj = DatabaseObjectInfo {
            schema: SchemaNamedType::Record(schema.clone()),
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
        let obj = DatabaseObjectInfo {
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
        object_id: ObjectId,
        prototype: Option<ObjectId>,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        properties_in_replace_mode: HashSet<String>,
        dynamic_array_entries: HashMap<String, Vec<Uuid>>,
    ) {
        let schema = self.schemas.get(&schema).unwrap();
        let obj = DatabaseObjectInfo {
            schema: schema.clone(),
            prototype,
            properties,
            property_null_overrides,
            properties_in_replace_mode,
            dynamic_array_entries,
        };

        self.objects.insert(object_id, obj);
    }

    pub(crate) fn restore_named_type(
        &mut self,
        named_type: SchemaNamedType
    ) {
        self.schemas.insert(named_type.fingerprint(), named_type);
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
        object: ObjectId
    ) -> Option<ObjectId> {
        let o = self.objects.get(&object).unwrap();
        o.prototype
    }

    pub fn object_schema(
        &self,
        object: ObjectId,
    ) -> &SchemaNamedType {
        let o = self.objects.get(&object).unwrap();
        &o.schema
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
        named_type: &'a SchemaNamedType,
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
            let s = schema.find_property_schema(path_segment, named_types)?;
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
                .find_property_schema(split_path.last().unwrap(), named_types)?
                .clone();
        }

        Some(schema)
    }

    pub fn get_null_override(
        &self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<NullOverride> {
        let mut object_schema = self.object_schema(object);
        let property_schema = object_schema.find_property_schema(&path, &self.schemas).unwrap();

        if property_schema.is_nullable() {
            let obj = self.objects.get(&object).unwrap();
            obj.property_null_overrides.get(path.as_ref()).copied()
        } else {
            None
        }
    }

    pub fn set_null_override(
        &mut self,
        object: ObjectId,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) {
        let mut object_schema = self.object_schema(object);
        let property_schema = object_schema.find_property_schema(&path, &self.schemas).unwrap();

        if property_schema.is_nullable() {
            let obj = self.objects.get_mut(&object).unwrap();
            obj.property_null_overrides
                .insert(path.as_ref().to_string(), null_override);
        }
    }

    pub fn remove_null_override(
        &mut self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) {
        let mut object_schema = self.object_schema(object);
        let property_schema = object_schema.find_property_schema(&path, &self.schemas).unwrap();

        if property_schema.is_nullable() {
            let obj = self.objects.get_mut(&object).unwrap();
            obj.property_null_overrides.remove(path.as_ref());
        }
    }

    // None return means the property can't be resolved, maybe because something higher in
    // property hierarchy is null or non-existing
    pub fn resolve_is_null(
        &self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<bool> {
        let mut object_id = Some(object);
        let mut object_schema = self.object_schema(object);

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
            &self.schemas,
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
            if self.resolve_is_null(object, checked_property) != Some(false) {
                return None;
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(object, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return None;
            }
        }

        while let Some(obj_id) = object_id {
            let obj = self.objects.get(&obj_id).unwrap();

            if let Some(value) = obj.property_null_overrides.get(path.as_ref()) {
                return Some(*value == NullOverride::SetNull);
            }

            object_id = obj.prototype;
        }

        //TODO: Return schema default value
        Some(true)
    }

    pub fn has_property_override(
        &self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> bool {
        self.get_property_override(object, path).is_some()
    }

    // Just gets if this object has a property without checking prototype chain for fallback or returning a default
    // Returning none means it is not overridden
    pub fn get_property_override(
        &self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<&Value> {
        let obj = self.objects.get(&object).unwrap();
        obj.properties.get(path.as_ref())
    }

    // Just sets a property on this object, making it overridden, or replacing the existing override
    pub fn set_property_override(
        &mut self,
        object: ObjectId,
        path: impl AsRef<str>,
        value: Value,
    ) -> bool {
        let mut object_schema = self.object_schema(object);
        let mut property_schema =
            object_schema.find_property_schema(&path, &self.schemas).unwrap();

        //TODO: Should we check for null in path ancestors?
        //TODO: Only allow setting on values that exist, in particular, dynamic array overrides
        if !value.matches_schema(&property_schema, &self.schemas) {
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
            &self.schemas,
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )
        .unwrap();

        for checked_property in &nullable_ancestors {
            if self.resolve_is_null(object, checked_property) != Some(false) {
                return false;
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(object, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return false;
            }
        }

        let obj = self.objects.get_mut(&object).unwrap();
        obj.properties.insert(path.as_ref().to_string(), value);
        true
    }

    pub fn remove_property_override(
        &mut self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        let mut obj = self.objects.get_mut(&object).unwrap();
        obj.properties.remove(path.as_ref())
    }

    pub fn apply_property_override_to_prototype(
        &mut self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) {
        let obj = self.objects.get(&object).unwrap();
        let prototype = obj.prototype;

        if let Some(prototype) = prototype {
            let v = self.remove_property_override(object, path.as_ref());
            if let Some(v) = v {
                self.set_property_override(prototype, path, v);
            }
        }
    }

    pub fn resolve_property(
        &self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<Value> {
        let mut object_id = Some(object);
        let mut object_schema = self.object_schema(object);

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
            &self.schemas,
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )
        .unwrap();

        for checked_property in &nullable_ancestors {
            if self.resolve_is_null(object, checked_property) != Some(false) {
                return None;
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(object, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return None;
            }
        }

        while let Some(obj_id) = object_id {
            let obj = self.objects.get(&obj_id).unwrap();

            if let Some(value) = obj.properties.get(path.as_ref()) {
                return Some(value.clone());
            }

            object_id = obj.prototype;
        }

        //TODO: Return schema default value
        Some(Value::default_for_schema(&property_schema, &self.schemas).clone())
    }

    pub fn get_dynamic_array_overrides(
        &self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> &[Uuid] {
        let mut object_schema = self.object_schema(object);
        let property_schema = object_schema.find_property_schema(&path, &self.schemas).unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("get_dynamic_array_overrides only allowed on dynamic arrays");
        }

        let obj = self.objects.get(&object).unwrap();
        if let Some(overrides) = obj.dynamic_array_entries.get(path.as_ref()) {
            &*overrides
        } else {
            &[]
        }
    }

    pub fn add_dynamic_array_override(
        &mut self,
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> Uuid {
        let mut object_schema = self.object_schema(object).clone();
        let property_schema = object_schema.find_property_schema(&path, &self.schemas).unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("add_dynamic_array_override only allowed on dynamic arrays");
        }

        let mut obj = self.objects.get_mut(&object).unwrap();
        let entry = obj
            .dynamic_array_entries
            .entry(path.as_ref().to_string())
            .or_insert(Default::default());
        let new_uuid = Uuid::new_v4();
        entry.push(new_uuid);
        new_uuid
    }

    pub fn remove_dynamic_array_override(
        &mut self,
        object: ObjectId,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) {
        let mut object_schema = self.object_schema(object).clone();
        let property_schema = object_schema.find_property_schema(&path, &self.schemas).unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("remove_dynamic_array_override only allowed on dynamic arrays");
        }

        let mut obj = self.objects.get_mut(&object).unwrap();
        if let Some(override_list) = obj.dynamic_array_entries.get_mut(path.as_ref()) {
            let index = override_list.iter().position(|x| *x == element_id);
            if let Some(index) = index {
                override_list.remove(index);
            } else {
                panic!("override not found");
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
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> Box<[Uuid]> {
        let mut object_schema = self.object_schema(object);

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
            &self.schemas,
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        );
        if property_schema.is_none() {
            panic!("dynamic array not found");
        }

        for checked_property in &nullable_ancestors {
            if self.resolve_is_null(object, checked_property) != Some(false) {
                return vec![].into_boxed_slice();
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(object, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return vec![].into_boxed_slice();
            }
        }

        let mut resolved_entries = vec![];
        self.do_resolve_dynamic_array(
            object,
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
        object: ObjectId,
        path: impl AsRef<str>,
    ) -> OverrideBehavior {
        let object = self.objects.get(&object).unwrap();
        let property_schema = object.schema.find_property_schema(&path, &self.schemas).unwrap();

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
        object: ObjectId,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) {
        let object = self.objects.get_mut(&object).unwrap();
        let property_schema = object.schema.find_property_schema(&path, &self.schemas).unwrap();

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
}
