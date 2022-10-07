use std::io::BufRead;
use std::str::FromStr;
use uuid::Uuid;
use super::{HashMap, HashSet, ObjectId, SchemaFingerprint};
use super::schema::*;
use super::value::*;

use crate::{BufferId, SchemaLinker, SchemaLinkerResult};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NullOverride {
    SetNull,
    SetNonNull
}

pub struct DatabaseObjectInfo {
    schema: SchemaNamedType, // Will always be a SchemaRecord
    prototype: Option<ObjectId>,
    properties: HashMap<String, Value>,
    property_null_overrides: HashMap<String, NullOverride>,
    properties_in_replace_mode: HashSet<String>,
    dynamic_array_entries: HashMap<String, Vec<Uuid>>,
}

//TODO: Delete unused property data when path ancestor is null or in replace mode

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OverrideBehavior {
    Append,
    Replace
}

#[derive(Default)]
pub struct Database {
    schemas_by_name: HashMap<String, SchemaFingerprint>,
    schemas: HashMap<SchemaFingerprint, SchemaNamedType>,
    objects: HashMap<ObjectId, DatabaseObjectInfo>,
}

impl Database {
    pub fn add_linked_types(&mut self, mut linker: SchemaLinker) -> SchemaLinkerResult<()> {
        let linked = linker.finish()?;

        //TODO: check no name collisions and merge with DB

        for (k, v) in linked.schemas {
            let old = self.schemas.insert(k, v);
            assert!(old.is_none());
        }

        for (k, v) in linked.schemas_by_name {
            let old = self.schemas_by_name.insert(k, v);
            assert!(old.is_none());
        }

        Ok(())
    }

    pub fn find_named_type(&self, name: impl AsRef<str>) -> Option<&SchemaNamedType> {
        self.schemas_by_name.get(name.as_ref()).map(|fingerprint| self.find_named_type_by_fingerprint(*fingerprint)).flatten()
    }

    pub fn find_named_type_by_fingerprint(&self, fingerprint: SchemaFingerprint) -> Option<&SchemaNamedType> {
        self.schemas.get(&fingerprint)
    }

    pub fn default_value_for_schema(&self, schema: &Schema) -> Value {
        Value::default_for_schema(schema, &self.schemas)
    }


    fn insert_object(&mut self, obj_info: DatabaseObjectInfo) -> ObjectId {
        let id = ObjectId(uuid::Uuid::new_v4().as_u128());
        let old = self.objects.insert(id, obj_info);
        assert!(old.is_none());

        id
    }

    pub fn new_object(&mut self, schema: &SchemaRecord) -> ObjectId {
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

    pub fn new_object_from_prototype(&mut self, prototype: ObjectId) -> ObjectId {
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







    pub fn object_schema(&self, object: ObjectId) -> &SchemaNamedType {
        let o = self.objects.get(&object).unwrap();
        &o.schema
    }

    fn property_schema(schema: &SchemaNamedType, path: impl AsRef<str>, named_types: &HashMap<SchemaFingerprint, SchemaNamedType>) -> Option<Schema> {
        let mut schema = Schema::NamedType(schema.fingerprint());

        //TODO: Escape map keys (and probably avoid path strings anyways)
        let split_path = path.as_ref().split(".");

        // Iterate the path segments to find
        for path_segment in split_path {
            let s = schema.find_property_schema(path_segment, named_types);
            if let Some(s) = s {
                schema = s.clone();
            } else {
                return None;
            }
        }

        Some(schema)
    }








    fn truncate_property_path(path: impl AsRef<str>, max_segment_count: usize) -> String {
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
        accessed_dynamic_array_keys: &mut Vec<(String, String)>
    ) -> Option<Schema> {
        let mut schema = Schema::NamedType(named_type.fingerprint());

        //TODO: Escape map keys (and probably avoid path strings anyways)
        let split_path: Vec<_> = path.as_ref().split(".").collect();

        //println!("property_schema_and_parents_to_check_for_replace_mode {}", path.as_ref());
        // Iterate the path segments to find

        let mut parent_is_dynamic_array = false;

        for (i, path_segment) in split_path[0..split_path.len() - 1].iter().enumerate() { //.as_ref().split(".").enumerate() {
            let s = schema.find_property_schema(path_segment, named_types)?;
            //println!("  next schema {:?}", s);

            // current path needs to be verified as existing
            if parent_is_dynamic_array {
                accessed_dynamic_array_keys.push((Self::truncate_property_path(path.as_ref(), i - 1), path_segment.to_string()));
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
                    },
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
            schema = schema.find_property_schema(split_path.last().unwrap(), named_types)?.clone();
        }

        Some(schema)
    }




    pub fn get_null_override(&self, object: ObjectId, path: impl AsRef<str>) -> Option<NullOverride> {
        let mut object_schema = self.object_schema(object);
        let property_schema = Self::property_schema(object_schema, &path, &self.schemas).unwrap();

        if property_schema.is_nullable() {
            let obj = self.objects.get(&object).unwrap();
            obj.property_null_overrides.get(path.as_ref()).copied()
        } else {
            None
        }
    }

    pub fn set_null_override(&mut self, object: ObjectId, path: impl AsRef<str>, null_override: NullOverride) {
        let mut object_schema = self.object_schema(object);
        let property_schema = Self::property_schema(object_schema, &path, &self.schemas).unwrap();

        if property_schema.is_nullable() {
            let obj = self.objects.get_mut(&object).unwrap();
            obj.property_null_overrides.insert(path.as_ref().to_string(), null_override);
        }
    }

    pub fn remove_null_override(&mut self, object: ObjectId, path: impl AsRef<str>) {
        let mut object_schema = self.object_schema(object);
        let property_schema = Self::property_schema(object_schema, &path, &self.schemas).unwrap();

        if property_schema.is_nullable() {
            let obj = self.objects.get_mut(&object).unwrap();
            obj.property_null_overrides.remove(path.as_ref());
        }
    }

    // None return means the property can't be resolved, maybe because something higher in
    // property hierarchy is null or non-existing
    pub fn resolve_is_null(&self, object: ObjectId, path: impl AsRef<str>) -> Option<bool> {
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
            &mut accessed_dynamic_array_keys
        ).unwrap();
        println!("SCHEMA OF PATH {} IS {:?}", path.as_ref(), property_schema);

        if !property_schema.is_nullable() {
            panic!("not nullable");
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










    pub fn has_property_override(&self, object: ObjectId, path: impl AsRef<str>) -> bool {
        self.get_property_override(object, path).is_some()
    }

    // Just gets if this object has a property without checking prototype chain for fallback or returning a default
    // Returning none means it is not overridden
    pub fn get_property_override(&self, object: ObjectId, path: impl AsRef<str>) -> Option<&Value> {
        let obj = self.objects.get(&object).unwrap();
        obj.properties.get(path.as_ref())
    }

    // Just sets a property on this object, making it overridden, or replacing the existing override
    pub fn set_property_override(&mut self, object: ObjectId, path: impl AsRef<str>, value: Value) -> bool {
        let mut object_schema = self.object_schema(object);
        let mut property_schema = Self::property_schema(object_schema, &path, &self.schemas).unwrap();

        //TODO: Should we check for null in path ancestors?
        //TODO: Only allow setting on values that exist, in particular, dynamic array overrides
        if !value.matches_schema(&property_schema, &self.schemas) {
            panic!("Value {:?} doesn't match schema {:?}", value, property_schema);
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
            &mut accessed_dynamic_array_keys
        ).unwrap();

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

    pub fn remove_property_override(&mut self, object: ObjectId, path: impl AsRef<str>) -> Option<Value> {
        let mut obj = self.objects.get_mut(&object).unwrap();
        obj.properties.remove(path.as_ref())
    }

    pub fn apply_property_override_to_prototype(&mut self, object: ObjectId, path: impl AsRef<str>) {
        let obj = self.objects.get(&object).unwrap();
        let prototype = obj.prototype;

        if let Some(prototype) = prototype {
            let v = self.remove_property_override(object, path.as_ref());
            if let Some(v) = v {
                self.set_property_override(prototype, path, v);
            }
        }
    }

    pub fn resolve_property(&self, object: ObjectId, path: impl AsRef<str>) -> Option<Value> {
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
            &mut accessed_dynamic_array_keys
        ).unwrap();

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










    pub fn get_dynamic_array_overrides(&self, object: ObjectId, path: impl AsRef<str>) -> &[Uuid] {
        let mut object_schema = self.object_schema(object);
        let property_schema = Self::property_schema(object_schema, &path, &self.schemas).unwrap();

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

    pub fn add_dynamic_array_override(&mut self, object: ObjectId, path: impl AsRef<str>) -> Uuid {
        let mut object_schema = self.object_schema(object).clone();
        let property_schema = Self::property_schema(&object_schema, &path, &self.schemas).unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("add_dynamic_array_override only allowed on dynamic arrays");
        }

        let mut obj = self.objects.get_mut(&object).unwrap();
        let entry = obj.dynamic_array_entries.entry(path.as_ref().to_string()).or_insert(Default::default());
        let new_uuid = Uuid::new_v4();
        entry.push(new_uuid);
        new_uuid
    }

    pub fn remove_dynamic_array_override(&mut self, object: ObjectId, path: impl AsRef<str>, element_id: Uuid) {
        let mut object_schema = self.object_schema(object).clone();
        let property_schema = Self::property_schema(&object_schema, &path, &self.schemas).unwrap();

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
        resolved_entries: &mut Vec<Uuid>
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
                    resolved_entries
                );
            }
        }

        if let Some(entries) = obj.dynamic_array_entries.get(path) {
            for entry in entries {
                resolved_entries.push(*entry);
            }
        }
    }

    pub fn resolve_dynamic_array(&self, object: ObjectId, path: impl AsRef<str>) -> Box<[Uuid]> {
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
            &mut accessed_dynamic_array_keys
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
        self.do_resolve_dynamic_array(object, path.as_ref(), &nullable_ancestors, &dynamic_array_ancestors, &map_ancestors, &accessed_dynamic_array_keys, &mut resolved_entries);
        resolved_entries.into_boxed_slice()
    }











    pub fn get_override_behavior(&self, object: ObjectId, path: impl AsRef<str>) -> OverrideBehavior {
        let object = self.objects.get(&object).unwrap();
        let property_schema = Self::property_schema(&object.schema, &path, &self.schemas).unwrap();

        match property_schema {
            Schema::DynamicArray(_) | Schema::Map(_) => {
                if object.properties_in_replace_mode.contains(path.as_ref()) {
                    OverrideBehavior::Replace
                } else {
                    OverrideBehavior::Append
                }
            },
            _ => OverrideBehavior::Replace
        }
    }

    pub fn set_override_behavior(&mut self, object: ObjectId, path: impl AsRef<str>, behavior: OverrideBehavior) {
        let object = self.objects.get_mut(&object).unwrap();
        let property_schema = Self::property_schema(&object.schema, &path, &self.schemas).unwrap();

        match property_schema {
            Schema::DynamicArray(_) | Schema::Map(_) => {
                let _ = match behavior {
                    OverrideBehavior::Append => object.properties_in_replace_mode.remove(path.as_ref()),
                    OverrideBehavior::Replace => object.properties_in_replace_mode.insert(path.as_ref().to_string()),
                };
            },
            _ => panic!("unexpected schema type")
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Database, NullOverride, OverrideBehavior, Schema, SchemaDefDynamicArray, SchemaDefType, SchemaDynamicArray, SchemaLinker, SchemaLinkerResult, SchemaRecord, SchemaRecordField, Value};

    fn create_vec3_schema(linker: &mut SchemaLinker) -> SchemaLinkerResult<()> {
        linker.register_record_type("Vec3", |builder| {
            builder.add_f32("x");
            builder.add_f32("y");
            builder.add_f32("z");
        })
    }

    // We want the same fingerprint out of a record as a Schema::Record(record)
    #[test]
    fn set_struct_values() {
        let mut linker = SchemaLinker::default();
        create_vec3_schema(&mut linker).unwrap();

        let mut db = Database::default();
        db.add_linked_types(linker).unwrap();
        let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap().clone();

        let obj = db.new_object(&vec3_type);
        assert_eq!(db.resolve_property(obj, "x").map(|x| x.as_f32()), Some(Some(0.0)));
        db.set_property_override(obj, "x", Value::F32(10.0));
        assert_eq!(db.resolve_property(obj, "x").map(|x| x.as_f32()), Some(Some(10.0)));
        db.set_property_override(obj, "y", Value::F32(20.0));
        assert_eq!(db.resolve_property(obj, "y").map(|x| x.as_f32()), Some(Some(20.0)));
        db.set_property_override(obj, "z", Value::F32(30.0));
        assert_eq!(db.resolve_property(obj, "z").map(|x| x.as_f32()), Some(Some(30.0)));
    }

    #[test]
    fn set_struct_values_in_struct() {
        let mut linker = SchemaLinker::default();
        create_vec3_schema(&mut linker).unwrap();

        linker.register_record_type("OuterStruct", |builder| {
            builder.add_struct("a", "Vec3");
            builder.add_struct("b", "Vec3");
        }).unwrap();

        let mut db = Database::default();
        db.add_linked_types(linker).unwrap();
        //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
        let outer_struct_type = db.find_named_type("OuterStruct").unwrap().as_record().unwrap().clone();

        let obj = db.new_object(&outer_struct_type);
        assert_eq!(db.resolve_property(obj, "a.x").map(|x| x.as_f32()), Some(Some(0.0)));
        db.set_property_override(obj, "a.x", Value::F32(10.0));
        assert_eq!(db.resolve_property(obj, "a.x").map(|x| x.as_f32()), Some(Some(10.0)));
        assert_eq!(db.resolve_property(obj, "b.x").map(|x| x.as_f32()), Some(Some(0.0)));
        db.set_property_override(obj, "b.x", Value::F32(20.0));
        assert_eq!(db.resolve_property(obj, "a.x").map(|x| x.as_f32()), Some(Some(10.0)));
        assert_eq!(db.resolve_property(obj, "b.x").map(|x| x.as_f32()), Some(Some(20.0)));
    }

    #[test]
    fn set_simple_property_override() {
        let mut linker = SchemaLinker::default();
        create_vec3_schema(&mut linker).unwrap();

        let mut db = Database::default();
        db.add_linked_types(linker).unwrap();
        let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap().clone();

        let obj1 = db.new_object(&vec3_type);
        let obj2 = db.new_object_from_prototype(obj1);
        assert_eq!(db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()), Some(0.0));
        assert_eq!(db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()), Some(0.0));
        assert_eq!(db.has_property_override(obj1, "x"), false);
        assert_eq!(db.has_property_override(obj2, "x"), false);
        assert_eq!(db.get_property_override(obj1, "x").is_none(), true);
        assert_eq!(db.get_property_override(obj2, "x").is_none(), true);

        db.set_property_override(obj1, "x", Value::F32(10.0));
        assert_eq!(db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()), Some(10.0));
        assert_eq!(db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()), Some(10.0));
        assert_eq!(db.has_property_override(obj1, "x"), true);
        assert_eq!(db.has_property_override(obj2, "x"), false);
        assert_eq!(db.get_property_override(obj1, "x").unwrap().as_f32().unwrap(), 10.0);
        assert_eq!(db.get_property_override(obj2, "x").is_none(), true);

        db.set_property_override(obj2, "x", Value::F32(20.0));
        assert_eq!(db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()), Some(10.0));
        assert_eq!(db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()), Some(20.0));
        assert_eq!(db.has_property_override(obj1, "x"), true);
        assert_eq!(db.has_property_override(obj2, "x"), true);
        assert_eq!(db.get_property_override(obj1, "x").unwrap().as_f32().unwrap(), 10.0);
        assert_eq!(db.get_property_override(obj2, "x").unwrap().as_f32().unwrap(), 20.0);

        db.remove_property_override(obj1, "x");
        assert_eq!(db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()), Some(0.0));
        assert_eq!(db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()), Some(20.0));
        assert_eq!(db.has_property_override(obj1, "x"), false);
        assert_eq!(db.has_property_override(obj2, "x"), true);
        assert_eq!(db.get_property_override(obj1, "x").is_none(), true);
        assert_eq!(db.get_property_override(obj2, "x").unwrap().as_f32().unwrap(), 20.0);

        db.remove_property_override(obj2, "x");
        assert_eq!(db.resolve_property(obj1, "x").map(|x| x.as_f32().unwrap()), Some(0.0));
        assert_eq!(db.resolve_property(obj2, "x").map(|x| x.as_f32().unwrap()), Some(0.0));
        assert_eq!(db.has_property_override(obj1, "x"), false);
        assert_eq!(db.has_property_override(obj2, "x"), false);
        assert_eq!(db.get_property_override(obj1, "x").is_none(), true);
        assert_eq!(db.get_property_override(obj2, "x").is_none(), true);
    }

    #[test]
    fn property_in_nullable() {
        // let vec3_schema_record = create_vec3_schema();
        //
        // let outer_struct = SchemaRecord::new("OuterStruct".to_string(), vec![].into_boxed_slice(), vec![
        //     SchemaRecordField::new(
        //         "nullable".to_string(),
        //         vec![].into_boxed_slice(),
        //         Schema::Nullable(Box::new(Schema::Record(vec3_schema_record)))
        //     )
        // ].into_boxed_slice());
        //
        // let mut db = Database::default();


        let mut linker = SchemaLinker::default();
        create_vec3_schema(&mut linker).unwrap();

        linker.register_record_type("OuterStruct", |builder| {
            builder.add_nullable("nullable", SchemaDefType::NamedType("Vec3".to_string()));
        }).unwrap();

        let mut db = Database::default();
        db.add_linked_types(linker).unwrap();
        //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap().clone();
        let outer_struct_type = db.find_named_type("OuterStruct").unwrap().as_record().unwrap().clone();

        let obj = db.new_object(&outer_struct_type);

        assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), true);
        assert_eq!(db.resolve_property(obj, "nullable.value.x").map(|x| x.as_f32().unwrap()), None);
        // This should fail because we are trying to set a null value
        assert!(!db.set_property_override(obj, "nullable.value.x", Value::F32(10.0)));
        assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), true);
        assert_eq!(db.resolve_property(obj, "nullable.value.x").map(|x| x.as_f32().unwrap()), None);
        db.set_null_override(obj, "nullable", NullOverride::SetNonNull);
        assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), false);
        assert_eq!(db.resolve_is_null(obj, "nullable"), Some(false));
        // This is still set to 0 because the above set should have failed
        assert_eq!(db.resolve_property(obj, "nullable.value.x").map(|x| x.as_f32().unwrap()), Some(0.0));
        db.set_property_override(obj, "nullable.value.x", Value::F32(10.0));
        assert_eq!(db.resolve_property(obj, "nullable.value.x").map(|x| x.as_f32().unwrap()), Some(10.0));

    }

    #[test]
    fn nullable_property_in_nullable() {
        // let vec3_schema_record = create_vec3_schema();
        //
        // let outer_struct = SchemaRecord::new("OuterStruct".to_string(), vec![].into_boxed_slice(), vec![
        //     SchemaRecordField::new(
        //         "nullable".to_string(),
        //         vec![].into_boxed_slice(),
        //         Schema::Nullable(Box::new(Schema::Nullable(Box::new(Schema::Record(vec3_schema_record)))))
        //     )
        // ].into_boxed_slice());
        //
        // let mut db = Database::default();


        let mut linker = SchemaLinker::default();
        create_vec3_schema(&mut linker).unwrap();

        linker.register_record_type("OuterStruct", |builder| {
            builder.add_nullable("nullable", SchemaDefType::Nullable(Box::new(SchemaDefType::NamedType("Vec3".to_string()))));
        }).unwrap();

        let mut db = Database::default();
        db.add_linked_types(linker).unwrap();
        //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
        let outer_struct_type = db.find_named_type("OuterStruct").unwrap().as_record().unwrap().clone();



        let obj = db.new_object(&outer_struct_type);

        assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), true);
        // This returns none because parent property is null, so this property should act like it doesn't exist
        assert_eq!(db.resolve_is_null(obj, "nullable.value"), None);
        assert_eq!(db.resolve_property(obj, "nullable.value.value.x").map(|x| x.as_f32().unwrap()), None);
        // This attempt to set should fail because an ancestor path is null
        assert!(!db.set_property_override(obj, "nullable.value.value.x", Value::F32(10.0)));
        assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), true);
        assert_eq!(db.resolve_is_null(obj, "nullable.value"), None);
        assert_eq!(db.resolve_property(obj, "nullable.value.value.x").map(|x| x.as_f32().unwrap()), None);
        db.set_null_override(obj, "nullable", NullOverride::SetNonNull);
        assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), false);
        assert_eq!(db.resolve_is_null(obj, "nullable.value").unwrap(), true);
        assert_eq!(db.resolve_property(obj, "nullable.value.value.x").map(|x| x.as_f32().unwrap()), None);
        db.set_null_override(obj, "nullable.value", NullOverride::SetNonNull);
        assert_eq!(db.resolve_is_null(obj, "nullable").unwrap(), false);
        assert_eq!(db.resolve_is_null(obj, "nullable.value").unwrap(), false);
        // This is default value because the attempt to set it to 10 above should have failed
        assert_eq!(db.resolve_property(obj, "nullable.value.value.x").map(|x| x.as_f32().unwrap()), Some(0.0));
        assert!(db.set_property_override(obj, "nullable.value.value.x", Value::F32(10.0)));
        assert_eq!(db.resolve_property(obj, "nullable.value.value.x").map(|x| x.as_f32().unwrap()), Some(10.0));
    }

    //TODO: Test override nullable

    #[test]
    fn struct_in_dynamic_array() {
        // let vec3_schema_record = create_vec3_schema();
        //
        // let outer_struct = SchemaRecord::new("OuterStruct".to_string(), vec![].into_boxed_slice(), vec![
        //     SchemaRecordField::new(
        //         "array".to_string(),
        //         vec![].into_boxed_slice(),
        //         Schema::DynamicArray(SchemaDynamicArray::new(Box::new(Schema::Record(vec3_schema_record))))
        //     )
        // ].into_boxed_slice());
        //
        // let mut db = Database::default();


        let mut linker = SchemaLinker::default();
        create_vec3_schema(&mut linker).unwrap();

        linker.register_record_type("OuterStruct", |builder| {
            builder.add_dynamic_array("array", SchemaDefType::NamedType("Vec3".to_string()));
        }).unwrap();

        let mut db = Database::default();
        db.add_linked_types(linker).unwrap();
        //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
        let outer_struct_type = db.find_named_type("OuterStruct").unwrap().as_record().unwrap().clone();




        let obj = db.new_object(&outer_struct_type);

        assert!(db.resolve_dynamic_array(obj, "array").is_empty());
        let uuid1 = db.add_dynamic_array_override(obj, "array");
        let prop1 = format!("array.{}.x", uuid1);
        assert_eq!(db.resolve_dynamic_array(obj, "array"), vec![uuid1].into_boxed_slice());
        let uuid2 = db.add_dynamic_array_override(obj, "array");
        let prop2 = format!("array.{}.x", uuid2);
        assert_eq!(db.resolve_dynamic_array(obj, "array"), vec![uuid1, uuid2].into_boxed_slice());

        assert_eq!(db.resolve_property(obj, &prop1).unwrap().as_f32().unwrap(), 0.0);
        assert_eq!(db.resolve_property(obj, &prop2).unwrap().as_f32().unwrap(), 0.0);
        db.set_property_override(obj, &prop1, Value::F32(10.0));
        assert_eq!(db.resolve_property(obj, &prop1).unwrap().as_f32().unwrap(), 10.0);
        assert_eq!(db.resolve_property(obj, &prop2).unwrap().as_f32().unwrap(), 0.0);
        db.set_property_override(obj, &prop2, Value::F32(20.0));
        assert_eq!(db.resolve_property(obj, &prop1).unwrap().as_f32().unwrap(), 10.0);
        assert_eq!(db.resolve_property(obj, &prop2).unwrap().as_f32().unwrap(), 20.0);

        db.remove_dynamic_array_override(obj, "array", uuid1);
        assert_eq!(db.resolve_dynamic_array(obj, "array"), vec![uuid2].into_boxed_slice());
        assert!(db.resolve_property(obj, &prop1).is_none());
        assert_eq!(db.resolve_property(obj, &prop2).unwrap().as_f32().unwrap(), 20.0);
    }

    #[test]
    fn dynamic_array_override_behavior() {
        // let vec3_schema_record = create_vec3_schema();
        //
        // let outer_struct = SchemaRecord::new("OuterStruct".to_string(), vec![].into_boxed_slice(), vec![
        //     SchemaRecordField::new(
        //         "array".to_string(),
        //         vec![].into_boxed_slice(),
        //         Schema::DynamicArray(SchemaDynamicArray::new(Box::new(Schema::Record(vec3_schema_record))))
        //     )
        // ].into_boxed_slice());
        //
        // let mut db = Database::default();


        let mut linker = SchemaLinker::default();
        create_vec3_schema(&mut linker).unwrap();

        linker.register_record_type("OuterStruct", |builder| {
            builder.add_dynamic_array("array", SchemaDefType::NamedType("Vec3".to_string()));
        }).unwrap();

        let mut db = Database::default();
        db.add_linked_types(linker).unwrap();
        //let vec3_type = db.find_named_type("Vec3").unwrap().as_record().unwrap();
        let outer_struct_type = db.find_named_type("OuterStruct").unwrap().as_record().unwrap().clone();

        let obj1 = db.new_object(&outer_struct_type);
        let obj2 = db.new_object_from_prototype(obj1);

        let item1 = db.add_dynamic_array_override(obj1, "array");
        let item2 = db.add_dynamic_array_override(obj2, "array");

        assert_eq!(db.resolve_dynamic_array(obj1, "array"), vec![item1].into_boxed_slice());
        assert_eq!(db.resolve_dynamic_array(obj2, "array"), vec![item1, item2].into_boxed_slice());

        // This should fail, this override is on obj2, not obj1
        assert!(!db.set_property_override(obj1, format!("array.{}.x", item2), Value::F32(20.0)));

        db.set_property_override(obj1, format!("array.{}.x", item1), Value::F32(10.0));
        db.set_property_override(obj2, format!("array.{}.x", item2), Value::F32(20.0));

        db.set_override_behavior(obj2, "array", OverrideBehavior::Replace);
        assert_eq!(db.resolve_dynamic_array(obj2, "array"), vec![item2].into_boxed_slice());

        assert!(db.resolve_property(obj2, format!("array.{}.x", item1)).is_none());
        assert_eq!(db.resolve_property(obj2, format!("array.{}.x", item2)).unwrap().as_f32().unwrap(), 20.0);

        // This should fail, this override is on obj1 which we no longer inherit
        assert!(!db.set_property_override(obj2, format!("array.{}.x", item1), Value::F32(30.0)));

        db.set_override_behavior(obj2, "array", OverrideBehavior::Append);
        assert_eq!(db.resolve_dynamic_array(obj2, "array"), vec![item1, item2].into_boxed_slice());
    }
}
