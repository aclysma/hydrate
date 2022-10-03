use std::io::BufRead;
use std::str::FromStr;
use uuid::Uuid;
use super::{HashSet, HashMap, ObjectId, SchemaFingerprint};
use super::schema::*;
use super::value::*;

mod record_type_builder;
use record_type_builder::RecordTypeBuilder;

mod fixed_type_builder;
use fixed_type_builder::FixedTypeBuilder;

mod enum_type_builder;
use enum_type_builder::EnumTypeBuilder;
use crate::BufferId;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NullOverride {
    SetNull,
    SetNonNull
}

pub struct DatabaseSchemaInfo {
    schema: Schema,
    fingerprint: SchemaFingerprint,
}

pub struct DatabaseObjectInfo {
    schema: Schema, // Will always be a SchemaRecord
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
    schemas: HashMap<SchemaFingerprint, DatabaseSchemaInfo>,
    objects: HashMap<ObjectId, DatabaseObjectInfo>,
}


impl Database {
    pub fn schema(&self, fingerprint: SchemaFingerprint) -> Option<&Schema> {
        self.schemas.get(&fingerprint).map(|x| &x.schema)
    }

    fn register_schema(&mut self, name: impl Into<String>, schema: Schema) {
        let fingerprint = schema.fingerprint();

        if let Some(existing_schema) = self.schemas.get(&fingerprint) {
            return;
        }

        self.schemas.entry(fingerprint).or_insert_with(|| {
            DatabaseSchemaInfo {
                fingerprint,
                schema,
            }
        });

        let old = self.schemas_by_name.insert(name.into(), fingerprint);

        // We do not allow a single schema name to reference different schemas
        assert_eq!(old.unwrap_or(fingerprint), fingerprint);
    }

    pub fn register_record_type<F: Fn(&mut RecordTypeBuilder)>(&mut self, name: impl Into<String>, f: F) -> SchemaRecord {
        let mut builder = RecordTypeBuilder::default();
        (f)(&mut builder);

        let fields: Vec<_> = builder.fields.into_iter().map(|x| {
            SchemaRecordField::new(x.name, x.aliases.into_boxed_slice(), x.field_type)
        }).collect();
        let schema_record = SchemaRecord::new(name.into(), builder.aliases.into_boxed_slice(), fields.into_boxed_slice());

        let schema = Schema::Record(schema_record.clone());
        println!("Registering struct {} {}", schema_record.name(), schema_record.fingerprint_uuid());
        self.register_schema(schema_record.name(), schema);

        schema_record
    }

    pub fn register_enum_type<F: Fn(&mut EnumTypeBuilder)>(&mut self, name: impl Into<String>, f: F) -> SchemaEnum {
        let mut builder = EnumTypeBuilder::default();
        (f)(&mut builder);

        let mut symbols: Vec<_> = builder.symbols.into_iter().map(|x| {
            SchemaEnumSymbol::new(x.name, x.aliases.into_boxed_slice(), x.value)
        }).collect();
        symbols.sort_by_key(|x| x.value());
        let schema_enum = SchemaEnum::new(name.into(), builder.aliases.into_boxed_slice(), symbols.into_boxed_slice());

        let schema = Schema::Enum(schema_enum.clone());
        println!("Registering enum {} {:?}", schema_enum.name(), schema.fingerprint());
        self.register_schema(schema_enum.name(), schema);

        schema_enum
    }

    pub fn register_fixed_type<F: Fn(&mut FixedTypeBuilder)>(&mut self, name: impl Into<String>, length: usize, f: F) -> SchemaFixed {
        let mut builder = FixedTypeBuilder::default();
        (f)(&mut builder);

        let schema_fixed = SchemaFixed::new(name.into(), builder.aliases.into_boxed_slice(), length);

        let schema = Schema::Fixed(schema_fixed.clone());
        println!("Registering fixed {} {}", schema_fixed.name(), schema.fingerprint().as_uuid());
        self.register_schema(schema_fixed.name(), schema);

        schema_fixed
    }

    pub fn find_schema_by_name(&self, name: &impl AsRef<str>) -> Option<Schema> {
        self.schemas_by_name.get(name.as_ref()).map(|fingerprint| self.find_schema_by_fingerprint(*fingerprint)).flatten()
    }

    pub fn find_schema_by_fingerprint(&self, fingerprint: SchemaFingerprint) -> Option<Schema> {
        self.schemas.get(&fingerprint).map(|x| x.schema.clone())
    }







    fn insert_object(&mut self, obj_info: DatabaseObjectInfo) -> ObjectId {
        let id = ObjectId(uuid::Uuid::new_v4().as_u128());
        let old = self.objects.insert(id, obj_info);
        assert!(old.is_none());

        id
    }

    pub fn new_object(&mut self, schema: &SchemaRecord) -> ObjectId {
        let obj = DatabaseObjectInfo {
            schema: Schema::Record(schema.clone()),
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







    pub fn object_schema(&self, object: ObjectId) -> &Schema {
        let o = self.objects.get(&object).unwrap();
        &o.schema
    }

    fn property_schema<'a>(mut schema: &'a Schema, path: &impl AsRef<str>) -> Option<&'a Schema> {
        //TODO: Escape map keys (and probably avoid path strings anyways)
        let split_path = path.as_ref().split(".");

        // Iterate the path segments to find
        for path_segment in split_path {
            let s = schema.find_property_schema(path_segment);
            if let Some(s) = s {
                schema = s;
            } else {
                return None;
            }
        }

        Some(schema)
    }








    fn truncate_property_path(path: &impl AsRef<str>, max_segment_count: usize) -> String {
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
        mut schema: &'a Schema,
        path: &impl AsRef<str>,
        nullable_ancestors: &mut Vec<String>,
        dynamic_array_ancestors: &mut Vec<String>,
        map_ancestors: &mut Vec<String>,
        accessed_dynamic_array_keys: &mut Vec<(String, String)>
    ) -> Option<&'a Schema> {
        //TODO: Escape map keys (and probably avoid path strings anyways)
        let split_path = path.as_ref().split(".");

        //println!("property_schema_and_parents_to_check_for_replace_mode {}", path.as_ref());
        // Iterate the path segments to find

        let mut parent_is_dynamic_array = false;

        for (i, path_segment) in split_path.enumerate() { //.as_ref().split(".").enumerate() {
            let s = schema.find_property_schema(path_segment);
            //println!("  next schema {:?}", s);

            // current path needs to be verified as existing
            if parent_is_dynamic_array {
                accessed_dynamic_array_keys.push((Self::truncate_property_path(path, i - 1), path_segment.to_string()));
            }

            parent_is_dynamic_array = false;

            if let Some(s) = s {
                // If it's nullable, we need to check for value being null before looking up the prototype chain
                // If it's a map or dynamic array, we need to check for append mode before looking up the prototype chain
                match s {
                    Schema::Nullable(_) => {
                        let mut shortened_path = Self::truncate_property_path(path, i);
                        nullable_ancestors.push(shortened_path);
                    }
                    Schema::DynamicArray(_) => {
                        let mut shortened_path = Self::truncate_property_path(path, i);
                        dynamic_array_ancestors.push(shortened_path.clone());

                        parent_is_dynamic_array = true;
                    },
                    Schema::Map(_) => {
                        let mut shortened_path = Self::truncate_property_path(path, i);
                        map_ancestors.push(shortened_path);
                    }
                    _ => {}
                }

                schema = s;
            } else {
                return None;
            }
        }

        Some(schema)
    }




    pub fn get_null_override(&self, object: ObjectId, path: &impl AsRef<str>) -> Option<NullOverride> {
        let mut object_schema = self.object_schema(object);
        let property_schema = Self::property_schema(object_schema, &path).unwrap();

        if property_schema.is_nullable() {
            let obj = self.objects.get(&object).unwrap();
            obj.property_null_overrides.get(path.as_ref()).copied()
        } else {
            None
        }
    }

    pub fn clear_null_override(&mut self, object: ObjectId, path: &impl AsRef<str>) {
        let mut object_schema = self.object_schema(object);
        let property_schema = Self::property_schema(object_schema, &path).unwrap();

        if property_schema.is_nullable() {
            let obj = self.objects.get_mut(&object).unwrap();
            obj.property_null_overrides.remove(path.as_ref());
        }
    }

    pub fn set_null_override(&mut self, object: ObjectId, path: &impl AsRef<str>, null_override: NullOverride) {
        let mut object_schema = self.object_schema(object);
        let property_schema = Self::property_schema(object_schema, &path).unwrap();

        if property_schema.is_nullable() {
            let obj = self.objects.get_mut(&object).unwrap();
            obj.property_null_overrides.insert(path.as_ref().to_string(), null_override);
        }
    }

    // None return means the property can't be resolved, maybe because something higher in
    // property hierarchy is null or non-existing
    pub fn resolve_is_null(&self, object: ObjectId, path: &impl AsRef<str>) -> Option<bool> {
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
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys
        ).unwrap();

        if !property_schema.is_nullable() {
            panic!("schema not nullable");
        }

        while let Some(obj_id) = object_id {
            let obj = self.objects.get(&obj_id).unwrap();
            for checked_property in &nullable_ancestors {
                if let Some(null_override) = obj.property_null_overrides.get(checked_property) {
                    if *null_override == NullOverride::SetNull {
                        return None;
                    }
                }
            }

            for (path, key) in &accessed_dynamic_array_keys {
                let dynamic_array_entries = self.resolve_dynamic_array(obj_id, path);
                if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                    return None;
                }
            }

            if let Some(value) = obj.property_null_overrides.get(path.as_ref()) {
                return Some(*value == NullOverride::SetNull);
            }

            for checked_property in &dynamic_array_ancestors {
                if obj.properties_in_replace_mode.contains(checked_property) {
                    return None;
                }
            }

            for checked_property in &map_ancestors {
                if obj.properties_in_replace_mode.contains(checked_property) {
                    return None;
                }
            }

            // for checked_property in &schema_parents_to_check_for_key_exists {
            //     if obj.dynamic_array_entries.contains(checked_property) {
            //         return None;
            //     }
            // }

            object_id = obj.prototype;
        }

        //TODO: Return schema default value
        Some(true)
    }

    pub fn has_property_override(&self, object: ObjectId, path: &impl AsRef<str>) -> bool {
        self.get_property_override(object, path).is_some()
    }

    // Just gets if this object has a property without checking prototype chain for fallback or returning a default
    // Returning none means it is not overridden
    pub fn get_property_override(&self, object: ObjectId, path: &impl AsRef<str>) -> Option<&Value> {
        let obj = self.objects.get(&object).unwrap();
        obj.properties.get(path.as_ref())
    }

    // Just sets a property on this object, making it overridden, or replacing the existing override
    pub fn set_property_override(&mut self, object: ObjectId, path: &impl AsRef<str>, value: Value) {
        let mut object_schema = self.object_schema(object);
        let mut property_schema = Self::property_schema(object_schema, &path).unwrap();

        match property_schema {
            Schema::Nullable(inner_schema) => property_schema = inner_schema,
            _ => {}
        }

        //TODO: Should we check for null in path ancestors?
        //TODO: Only allow setting on values that exist, in particular, dynamic array overrides
        if !value.matches_schema(property_schema) {
            panic!("Value {:?} doesn't match schema {:?}", value, property_schema);
        }

        let obj = self.objects.get_mut(&object).unwrap();
        obj.properties.insert(path.as_ref().to_string(), value);
    }

    pub fn remove_property_override(&mut self, object: ObjectId, path: &impl AsRef<str>) -> Option<Value> {
        let mut obj = self.objects.get_mut(&object).unwrap();
        obj.properties.remove(path.as_ref())
    }

    pub fn apply_property_override_to_prototype(&mut self, object: ObjectId, path: &impl AsRef<str>) {
        let obj = self.objects.get(&object).unwrap();
        let prototype = obj.prototype;

        if let Some(prototype) = prototype {
            let v = self.remove_property_override(object, path);
            if let Some(v) = v {
                self.set_property_override(prototype, path, v);
            }
        }
    }

    pub fn resolve_property(&self, object: ObjectId, path: &impl AsRef<str>) -> Option<&Value> {
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
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys
        );

        while let Some(obj_id) = object_id {
            let obj = self.objects.get(&obj_id).unwrap();
            for checked_property in &nullable_ancestors {
                if let Some(null_override) = obj.property_null_overrides.get(checked_property) {
                    if *null_override == NullOverride::SetNull {
                        return None;
                    }
                }
            }

            for (path, key) in &accessed_dynamic_array_keys {
                let dynamic_array_entries = self.resolve_dynamic_array(obj_id, path);
                if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                    return None;
                }
            }

            if let Some(value) = obj.properties.get(path.as_ref()) {
                return Some(value);
            }

            for checked_property in &dynamic_array_ancestors {
                if obj.properties_in_replace_mode.contains(checked_property) {
                    return None;
                }
            }

            for checked_property in &map_ancestors {
                if obj.properties_in_replace_mode.contains(checked_property) {
                    return None;
                }
            }

            // for checked_property in &schema_parents_to_check_for_key_exists {
            //     if obj.dynamic_array_entries.contains(checked_property) {
            //         return None;
            //     }
            // }

            object_id = obj.prototype;
        }

        //TODO: Return schema default value
        None
    }










    pub fn get_dynamic_array_overrides(&self, object: ObjectId, path: &impl AsRef<str>) -> &[Uuid] {
        let mut object_schema = self.object_schema(object);
        let property_schema = Self::property_schema(object_schema, &path).unwrap();

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

    pub fn add_dynamic_array_override(&mut self, object: ObjectId, path: &impl AsRef<str>) -> Uuid {
        let mut object_schema = self.object_schema(object).clone();
        let property_schema = Self::property_schema(&object_schema, &path).unwrap();

        if !property_schema.is_dynamic_array() {
            panic!("add_dynamic_array_override only allowed on dynamic arrays");
        }

        let mut obj = self.objects.get_mut(&object).unwrap();
        let entry = obj.dynamic_array_entries.entry(path.as_ref().to_string()).or_insert(Default::default());
        let new_uuid = Uuid::new_v4();
        entry.push(new_uuid);
        new_uuid
    }

    pub fn remove_dynamic_array_override(&mut self, object: ObjectId, path: &impl AsRef<str>, element_id: Uuid) {
        let mut object_schema = self.object_schema(object).clone();
        let property_schema = Self::property_schema(&object_schema, &path).unwrap();

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

        // if path ancestor is nulled, we do not use any entries from this object or parent
        for checked_property in nullable_ancestors {
            if let Some(null_override) = obj.property_null_overrides.get(checked_property) {
                if *null_override == NullOverride::SetNull {
                    return;
                }
            }
        }

        for (path, key) in accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(object_id, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return;
            }
        }

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

    pub fn resolve_dynamic_array(&self, object: ObjectId, path: &impl AsRef<str>) -> Box<[Uuid]> {
        let mut object_schema = self.object_schema(object);

        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];


        let property_schema = Self::property_schema_and_path_ancestors_to_check(object_schema, &path, &mut nullable_ancestors, &mut dynamic_array_ancestors, &mut map_ancestors, &mut accessed_dynamic_array_keys);
        if property_schema.is_none() {
            panic!("dynamic array not found");
        }

        let mut resolved_entries = vec![];
        self.do_resolve_dynamic_array(object, path.as_ref(), &nullable_ancestors, &dynamic_array_ancestors, &map_ancestors, &accessed_dynamic_array_keys, &mut resolved_entries);
        resolved_entries.into_boxed_slice()
    }











    pub fn get_override_behavior(&self, object: ObjectId, path: &impl AsRef<str>) -> OverrideBehavior {
        let object = self.objects.get(&object).unwrap();
        let property_schema = Self::property_schema(&object.schema, &path).unwrap();

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

    pub fn set_override_behavior(&mut self, object: ObjectId, path: &impl AsRef<str>, behavior: OverrideBehavior) {
        let object = self.objects.get_mut(&object).unwrap();
        let property_schema = Self::property_schema(&object.schema, &path).unwrap();

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
