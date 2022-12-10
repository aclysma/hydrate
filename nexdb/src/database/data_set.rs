use crate::SchemaSet;
use crate::{
    HashMap, HashMapKeys, HashSet, HashSetIter, ObjectId, Schema, SchemaFingerprint,
    SchemaNamedType, SchemaRecord, Value,
};
use std::str::{FromStr, Split};
use uuid::Uuid;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ObjectSourceId(Uuid);

impl ObjectSourceId {
    pub fn new() -> Self {
        ObjectSourceId(Uuid::new_v4())
    }

    pub(crate) fn new_with_uuid(uuid: Uuid) -> Self {
        ObjectSourceId(uuid)
    }

    pub fn uuid(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectPath(String);

impl ObjectPath {
    pub fn new(s: &str) -> Self {
        ObjectPath(s.to_string())
    }

    pub fn root() -> Self {
        ObjectPath("db:/".to_string())
    }

    pub fn join(
        &self,
        rhs: &str,
    ) -> ObjectPath {
        if self.0.ends_with("/") {
            ObjectPath(format!("{}{}", self.0, rhs))
        } else {
            ObjectPath(format!("{}/{}", self.0, rhs))
        }
    }

    pub fn strip_prefix(
        &self,
        prefix: &ObjectPath,
    ) -> Option<ObjectPath> {
        self.0
            .strip_prefix(&prefix.0)
            .map(|x| ObjectPath(x.to_string()))
    }

    pub fn split_components(
        &self
    ) -> Vec<&str> {
        //let split = self.strip_prefix(&Self::root()).unwrap();
        self.0.split("/").collect()
    }

    pub fn as_string(&self) -> &str {
        &self.0
    }

    pub fn starts_with(&self, other: &ObjectPath) -> bool {
        self.0.starts_with(&other.0)
    }
}

impl From<&str> for ObjectPath {
    fn from(s: &str) -> Self {
        ObjectPath(s.to_string())
    }
}

impl From<String> for ObjectPath {
    fn from(s: String) -> Self {
        ObjectPath(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectLocation {
    source: ObjectSourceId,
    path: ObjectPath,
}

impl ObjectLocation {
    pub fn new(
        source: ObjectSourceId,
        path: ObjectPath,
    ) -> Self {
        ObjectLocation { source, path }
    }

    pub fn source(&self) -> ObjectSourceId {
        self.source
    }

    pub fn path(&self) -> &ObjectPath {
        &self.path
    }
}

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

pub struct DataObjectDelta {}

#[derive(Clone, Debug)]
pub struct DataObjectInfo {
    pub(crate) schema: SchemaRecord,
    pub(crate) object_location: ObjectLocation,
    pub(crate) prototype: Option<ObjectId>,
    pub(crate) properties: HashMap<String, Value>,
    pub(crate) property_null_overrides: HashMap<String, NullOverride>,
    pub(crate) properties_in_replace_mode: HashSet<String>,
    pub(crate) dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
}

impl DataObjectInfo {
    pub fn object_location(&self) -> &ObjectLocation {
        &self.object_location
    }

    pub fn schema(&self) -> &SchemaRecord {
        &self.schema
    }
}

#[derive(Default)]
pub struct DataSet {
    pub(crate) objects: HashMap<ObjectId, DataObjectInfo>,
}

impl DataSet {
    pub fn all_objects<'a>(&'a self) -> HashMapKeys<'a, ObjectId, DataObjectInfo> {
        self.objects.keys()
    }

    pub(crate) fn objects(&self) -> &HashMap<ObjectId, DataObjectInfo> {
        &self.objects
    }

    pub(crate) fn objects_mut(&mut self) -> &mut HashMap<ObjectId, DataObjectInfo> {
        &mut self.objects
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

    pub(crate) fn restore_object(
        &mut self,
        object_id: ObjectId,
        object_location: ObjectLocation,
        schema_set: &SchemaSet,
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
            object_location,
            prototype,
            properties,
            property_null_overrides,
            properties_in_replace_mode,
            dynamic_array_entries,
        };

        self.objects.insert(object_id, obj);
    }

    pub fn new_object(
        &mut self,
        object_location: ObjectLocation,
        schema: &SchemaRecord,
    ) -> ObjectId {
        let obj = DataObjectInfo {
            schema: schema.clone(),
            object_location,
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
        object_location: ObjectLocation,
        prototype: ObjectId,
    ) -> ObjectId {
        let prototype_info = self.objects.get(&prototype).unwrap();
        let obj = DataObjectInfo {
            schema: prototype_info.schema.clone(),
            object_location,
            prototype: Some(prototype),
            properties: Default::default(),
            property_null_overrides: Default::default(),
            properties_in_replace_mode: Default::default(),
            dynamic_array_entries: Default::default(),
        };

        self.insert_object(obj)
    }

    pub fn delete_object(
        &mut self,
        object_id: ObjectId,
    ) {
        //TODO: Kill subobjects too
        //TODO: Write tombstone?
        self.objects.remove(&object_id);
    }

    pub fn set_object_location(
        &mut self,
        object_id: ObjectId,
        new_location: ObjectLocation
    ) {
        self.objects.get_mut(&object_id).unwrap().object_location = new_location;
    }

    pub fn copy_from(
        &mut self,
        other: &DataSet,
        object_id: ObjectId,
    ) {
        let object = other.objects.get(&object_id).cloned().unwrap();
        self.objects.insert(object_id, object);
    }

    pub fn object_prototype(
        &self,
        object_id: ObjectId,
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
                    let shortened_path = Self::truncate_property_path(path.as_ref(), i);
                    nullable_ancestors.push(shortened_path);
                }
                Schema::DynamicArray(_) => {
                    let shortened_path = Self::truncate_property_path(path.as_ref(), i);
                    dynamic_array_ancestors.push(shortened_path.clone());

                    parent_is_dynamic_array = true;
                }
                Schema::Map(_) => {
                    let shortened_path = Self::truncate_property_path(path.as_ref(), i);
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
                .find_field_schema(last_path_segment, named_types)?
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
        let property_schema = object
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

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
        let object = self.objects.get_mut(&object_id).unwrap();
        let property_schema = object
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

        if property_schema.is_nullable() {
            object
                .property_null_overrides
                .insert(path.as_ref().to_string(), null_override);
        }
    }

    pub fn remove_null_override(
        &mut self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) {
        let object = self.objects.get_mut(&object_id).unwrap();
        let property_schema = object
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

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
        let object_schema = self.object_schema(object_id).unwrap();

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
        let object_schema = self.object_schema(object_id).unwrap();
        let property_schema = object_schema
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

        let _property_schema = Self::property_schema_and_path_ancestors_to_check(
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
        let object = self.objects.get_mut(&object_id).unwrap();
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
        let object_schema = self.object_schema(object_id).unwrap();

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

    pub fn get_dynamic_array_overrides(
        &self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<HashSetIter<Uuid>> {
        let object = self.objects.get(&object_id).unwrap();
        let property_schema = object
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

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
        let object = self.objects.get_mut(&object_id).unwrap();
        let property_schema = object
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

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
        let property_schema = object
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

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
        let object_schema = self.object_schema(object_id).unwrap();

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
        let property_schema = object
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

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
        let property_schema = object
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .unwrap();

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
