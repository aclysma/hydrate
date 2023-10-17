use crate::{
    HashMap, HashMapKeys, HashSet, HashSetIter, ObjectId, Schema, SchemaFingerprint,
    SchemaRecord, Value,
};
use crate::{NullOverride, SchemaSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use uuid::Uuid;

#[derive(Debug)]
pub enum DataSetError {
    ValueDoesNotMatchSchema,
    PathParentIsNull,
    PathDynamicArrayEntryDoesNotExist,
    UnexpectedEnumSymbol,
    DuplicateObjectId
}

pub type DataSetResult<T> = Result<T, DataSetError>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ObjectSourceId(Uuid);

impl ObjectSourceId {
    pub fn new() -> Self {
        ObjectSourceId(Uuid::new_v4())
    }

    pub fn null() -> Self {
        ObjectSourceId(Uuid::nil())
    }

    pub fn uuid(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectName(String);

impl ObjectName {
    pub fn new<T: Into<String>>(name: T) -> Self {
        ObjectName(name.into())
    }

    pub fn empty() -> Self {
        ObjectName(String::default())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_string(&self) -> Option<&String> {
        if self.0.is_empty() {
            None
        } else {
            Some(&self.0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ObjectLocation {
    path_node_id: ObjectId,
}

impl ObjectLocation {
    pub fn new(
        path_node_id: ObjectId,
    ) -> Self {
        ObjectLocation {
            path_node_id,
        }
    }

    pub fn null() -> ObjectLocation {
        ObjectLocation {
            path_node_id: ObjectId::null(),
        }
    }

    pub fn path_node_id(&self) -> ObjectId {
        self.path_node_id
    }

    pub fn is_null(&self) -> bool {
        self.path_node_id.is_null()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OverrideBehavior {
    Append,
    Replace,
}

pub struct DataObjectDelta {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImporterId(pub Uuid);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BuilderId(pub usize);

#[derive(Clone, Debug)]
pub struct ImportInfo {
    // Set on initial import
    importer_id: ImporterId,

    // Set on initial import, or re-import. This affects the import step.
    // Anything that just affects the build step should be an asset property instead.
    //import_options: SingleObject,

    // Set on initial import, or re-import. Used to monitor to detect stale imported data and
    // automaticlaly re-import, and as a heuristic when importing other files that reference this
    // file to link to this object rather than importing another copy.
    source_file_path: PathBuf,

    // If the asset comes from a file with more than one importable thing, we require a string key
    // to specify which importable this asset represents. It will be an empty string for simple
    // cases where a file has one importable thing.
    importable_name: String,

    // All the file references that need to be resolved in order to build the asset (this represents
    // file references encountered in the input data, and only changes when data is re-imported)
    file_references: Vec<PathBuf>,
}

impl ImportInfo {
    pub fn new(
        importer_id: ImporterId, /*, import_options: SingleObject*/
        source_file_path: PathBuf,
        importable_name: String,
        file_references: Vec<PathBuf>,
    ) -> Self {
        ImportInfo {
            importer_id,
            //import_options,
            source_file_path,
            importable_name,
            file_references, //referenced_source_file_overrides: Default::default()
        }
    }

    pub fn importer_id(&self) -> ImporterId {
        self.importer_id
    }

    pub fn source_file_path(&self) -> &Path {
        &self.source_file_path
    }

    pub fn importable_name(&self) -> &str {
        &self.importable_name
    }

    pub fn file_references(&self) -> &[PathBuf] {
        &self.file_references
    }
}

#[derive(Clone, Debug, Default)]
pub struct BuildInfo {
    pub file_reference_overrides: HashMap<PathBuf, ObjectId>,
}

#[derive(Clone, Debug)]
pub struct DataObjectInfo {
    schema: SchemaRecord,
    //name: Option<String>,
    //path: ObjectPath,
    //
    pub(super) object_name: ObjectName,
    pub(super) object_location: ObjectLocation,

    // Stores the configuration/choices that were made when the asset was last imported
    pub(super) import_info: Option<ImportInfo>,
    pub(super) build_info: BuildInfo,

    pub(super) prototype: Option<ObjectId>,
    pub(super) properties: HashMap<String, Value>,
    pub(super) property_null_overrides: HashMap<String, NullOverride>,
    pub(super) properties_in_replace_mode: HashSet<String>,
    pub(super) dynamic_array_entries: HashMap<String, HashSet<Uuid>>,
}

impl DataObjectInfo {
    pub fn schema(&self) -> &SchemaRecord {
        &self.schema
    }

    pub fn object_name(&self) -> &ObjectName {
        &self.object_name
    }

    pub fn object_location(&self) -> &ObjectLocation {
        &self.object_location
    }

    pub fn import_info(&self) -> &Option<ImportInfo> {
        &self.import_info
    }

    // pub fn path(&self) -> &ObjectPath {
    //     &self.path
    // }

    pub fn build_info(&self) -> &BuildInfo {
        &self.build_info
    }

    pub fn prototype(&self) -> Option<ObjectId> {
        self.prototype
    }

    pub fn properties(&self) -> &HashMap<String, Value> {
        &self.properties
    }

    pub fn property_null_overrides(&self) -> &HashMap<String, NullOverride> {
        &self.property_null_overrides
    }

    pub fn properties_in_replace_mode(&self) -> &HashSet<String> {
        &self.properties_in_replace_mode
    }

    pub fn dynamic_array_entries(&self) -> &HashMap<String, HashSet<Uuid>> {
        &self.dynamic_array_entries
    }
}

#[derive(Default)]
pub struct DataSet {
    objects: HashMap<ObjectId, DataObjectInfo>,
}

impl DataSet {
    pub fn all_objects<'a>(&'a self) -> HashMapKeys<'a, ObjectId, DataObjectInfo> {
        self.objects.keys()
    }

    pub fn objects(&self) -> &HashMap<ObjectId, DataObjectInfo> {
        &self.objects
    }

    pub fn take_objects(self) -> HashMap<ObjectId, DataObjectInfo> {
        self.objects
    }

    pub(super) fn objects_mut(&mut self) -> &mut HashMap<ObjectId, DataObjectInfo> {
        &mut self.objects
    }

    // fn insert_object(
    //     &mut self,
    //     obj_info: DataObjectInfo,
    // ) -> ObjectId {
    //     let id = ObjectId(uuid::Uuid::new_v4().as_u128());
    //     self.insert_object_with_id(id, obj_info).unwrap();
    //
    //     id
    // }

    fn insert_object(
        &mut self,
        id: ObjectId,
        obj_info: DataObjectInfo,
    ) -> DataSetResult<()> {
        if self.objects.contains_key(&id) {
            return Err(DataSetError::DuplicateObjectId);
        }

        let old = self.objects.insert(id, obj_info);
        assert!(old.is_none());

        Ok(())
    }

    pub fn restore_object(
        &mut self,
        object_id: ObjectId,
        object_name: ObjectName,
        object_location: ObjectLocation,
        import_info: Option<ImportInfo>,
        build_info: BuildInfo,
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
            object_name,
            object_location,
            import_info,
            build_info,
            prototype,
            properties,
            property_null_overrides,
            properties_in_replace_mode,
            dynamic_array_entries,
        };

        self.objects.insert(object_id, obj);
    }

    pub fn new_object_with_id(
        &mut self,
        object_id: ObjectId,
        object_name: ObjectName,
        object_location: ObjectLocation,
        schema: &SchemaRecord,
    ) -> DataSetResult<()> {
        let obj = DataObjectInfo {
            schema: schema.clone(),
            object_name,
            object_location,
            import_info: None,
            build_info: Default::default(),
            prototype: None,
            properties: Default::default(),
            property_null_overrides: Default::default(),
            properties_in_replace_mode: Default::default(),
            dynamic_array_entries: Default::default(),
        };

        self.insert_object(object_id, obj)
    }

    pub fn new_object(
        &mut self,
        object_name: ObjectName,
        object_location: ObjectLocation,
        schema: &SchemaRecord,
    ) -> ObjectId {
        let id = ObjectId(uuid::Uuid::new_v4().as_u128());
        self.new_object_with_id(id, object_name, object_location, schema).unwrap();
        id
    }

    pub fn new_object_from_prototype(
        &mut self,
        object_name: ObjectName,
        object_location: ObjectLocation,
        prototype: ObjectId,
    ) -> ObjectId {
        let id = ObjectId(uuid::Uuid::new_v4().as_u128());
        let prototype_info = self.objects.get(&prototype).unwrap();
        let obj = DataObjectInfo {
            schema: prototype_info.schema.clone(),
            object_name,
            object_location,
            import_info: None,
            build_info: Default::default(),
            prototype: Some(prototype),
            properties: Default::default(),
            property_null_overrides: Default::default(),
            properties_in_replace_mode: Default::default(),
            dynamic_array_entries: Default::default(),
        };

        self.insert_object(id, obj).unwrap();
        id
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
        new_location: ObjectLocation,
    ) {
        self.objects.get_mut(&object_id).unwrap().object_location = new_location;
    }

    pub fn set_import_info(
        &mut self,
        object_id: ObjectId,
        import_info: ImportInfo,
    ) {
        self.objects.get_mut(&object_id).unwrap().import_info = Some(import_info);
    }

    pub fn copy_from(
        &mut self,
        other: &DataSet,
        object_id: ObjectId,
    ) {
        let object = other.objects.get(&object_id).cloned().unwrap();
        self.objects.insert(object_id, object);
    }

    pub fn object_name(
        &self,
        object_id: ObjectId,
    ) -> &ObjectName {
        let object = self.objects.get(&object_id).unwrap();
        &object.object_name
    }

    pub fn set_object_name(
        &mut self,
        object_id: ObjectId,
        object_name: ObjectName
    ) {
        self.objects.get_mut(&object_id).unwrap().object_name = object_name;
    }

    // Returns the object's parent
    pub fn object_location(
        &self,
        object_id: ObjectId,
    ) -> Option<&ObjectLocation> {
        self.objects.get(&object_id).map(|x| &x.object_location)
    }

    // Returns the object locations from the parent all the way up to the root parent. If a cycle is
    // detected or any elements in the chain are not found, an empty list is returned.
    pub fn object_location_chain(
        &self,
        object_id: ObjectId,
    ) -> Vec<ObjectLocation> {
        let mut object_location_chain = Vec::default();

        // If this object's location is none, return an empty list
        let Some(mut obj_iter) = self.object_location(object_id).cloned() else {
            return object_location_chain;
        };

        // Iterate up the chain
        while !obj_iter.path_node_id.is_null() {
            if object_location_chain.contains(&obj_iter) {
                // Detected a cycle, return an empty list
                return Vec::default();
            }

            object_location_chain.push(obj_iter.clone());
            obj_iter = if let Some(location) = self.object_location(obj_iter.path_node_id).cloned() {
                // May be null, in which case we will terminate and return this list so far not including the null
                location
            } else {
                // The parent was specified but not found, default to empty list if the chain is in a bad state
                return Vec::default();
            };
        }

        object_location_chain
    }

    pub fn import_info(
        &self,
        object_id: ObjectId,
    ) -> Option<&ImportInfo> {
        self.objects
            .get(&object_id)
            .map(|x| x.import_info.as_ref())
            .flatten()
    }

    // pub fn import_info(
    //     &self,
    //     object_id: ObjectId
    // ) -> Option<&ImportInfo> {
    //     self.objects.get(&object_id).map(|x| x.import_info.as_ref()).flatten()
    // }

    fn do_resolve_all_file_references(
        &self,
        object_id: ObjectId,
        all_references: &mut HashMap<PathBuf, ObjectId>,
    ) -> bool {
        let object = self.objects.get(&object_id);
        if let Some(object) = object {
            if let Some(prototype) = object.prototype {
                if !self.do_resolve_all_file_references(prototype, all_references) {
                    return false;
                }
            }

            for (k, v) in &object.build_info.file_reference_overrides {
                all_references.insert(k.clone(), *v);
            }
        } else {
            return false;
        }

        true
    }

    pub fn resolve_all_file_references(
        &self,
        object_id: ObjectId,
    ) -> Option<HashMap<PathBuf, ObjectId>> {
        let mut all_references = HashMap::default();
        if self.do_resolve_all_file_references(object_id, &mut all_references) {
            Some(all_references)
        } else {
            None
        }
    }

    pub fn get_all_file_reference_overrides(
        &mut self,
        object_id: ObjectId,
    ) -> Option<&HashMap<PathBuf, ObjectId>> {
        self.objects
            .get(&object_id)
            .map(|x| &x.build_info.file_reference_overrides)
    }

    pub fn set_file_reference_override(
        &mut self,
        object_id: ObjectId,
        path: PathBuf,
        referenced_object_id: ObjectId,
    ) {
        self.objects.get_mut(&object_id).map(|x| {
            x.build_info
                .file_reference_overrides
                .insert(path, referenced_object_id)
        });
    }

    // pub fn build_info(
    //     &self,
    //     object_id: ObjectId
    // ) -> Option<&BuildInfo> {
    //     self.objects.get(&object_id).map(|x| x.build_info.as_ref()).flatten()
    // }
    //
    // pub fn build_info_mut(
    //     &mut self,
    //     object_id: ObjectId
    // ) -> Option<&mut BuildInfo> {
    //     self.objects.get_mut(&object_id).map(|x| x.build_info.as_mut()).flatten()
    // }

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

    pub fn hash_properties(
        &self,
        object_id: ObjectId,
    ) -> Option<u64> {
        let object = self.objects.get(&object_id)?;
        let schema = &object.schema;

        let mut hasher = siphasher::sip::SipHasher::default();

        schema.fingerprint().hash(&mut hasher);
        //object_name
        //object_location
        //import_info
        //build_info
        if let Some(prototype) = object.prototype {
            self.hash_properties(prototype).hash(&mut hasher);
        }

        // properties
        let mut properties_hash = 0;
        for (key, value) in &object.properties {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            key.hash(&mut inner_hasher);
            value.hash(&mut inner_hasher);
            properties_hash = properties_hash ^ inner_hasher.finish();
        }
        properties_hash.hash(&mut hasher);

        // property_null_overrides
        let mut property_null_overrides_hash = 0;
        for (key, value) in &object.property_null_overrides {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            key.hash(&mut inner_hasher);
            value.hash(&mut inner_hasher);
            property_null_overrides_hash = property_null_overrides_hash ^ inner_hasher.finish();
        }
        property_null_overrides_hash.hash(&mut hasher);

        // properties_in_replace_mode
        let mut properties_in_replace_mode_hash = 0;
        for value in &object.properties_in_replace_mode {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            value.hash(&mut inner_hasher);
            properties_in_replace_mode_hash =
                properties_in_replace_mode_hash ^ inner_hasher.finish();
        }
        properties_in_replace_mode_hash.hash(&mut hasher);

        // dynamic_array_entries
        let mut dynamic_array_entries_hash = 0;
        for (key, value) in &object.dynamic_array_entries {
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
        dynamic_array_entries_hash.hash(&mut hasher);

        let object_hash = hasher.finish();
        Some(object_hash)
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

        let property_schema = super::property_schema_and_path_ancestors_to_check(
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
    ) -> DataSetResult<()> {
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
                return Err(DataSetError::PathParentIsNull);
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, object_id, path);
            if !dynamic_array_entries.contains(&Uuid::from_str(key).unwrap()) {
                return Err(DataSetError::PathDynamicArrayEntryDoesNotExist);
            }
        }

        let obj = self.objects.get_mut(&object_id).unwrap();
        obj.properties.insert(path.as_ref().to_string(), value);
        Ok(())
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
    ) -> DataSetResult<()> {
        let object = self.objects.get(&object_id).unwrap();
        let prototype_id = object.prototype;

        if let Some(prototype_id) = prototype_id {
            let v = self.remove_property_override(object_id, path.as_ref());
            if let Some(v) = v {
                // The property existed on the child, set it on the prototype
                self.set_property_override(schema_set, prototype_id, path, v)
            } else {
                // The property didn't exist on the child, do nothing
                Ok(())
            }
        } else {
            // There is no prototype or it couldn't be found, do nothing
            Ok(())
        }
    }

    pub fn resolve_property(
        &self,
        schema_set: &SchemaSet,
        object_id: ObjectId,
        path: impl AsRef<str>,
    ) -> Option<&Value> {
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

        let property_schema = super::property_schema_and_path_ancestors_to_check(
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
                return Some(value);
            }

            prototype_id = obj.prototype;
        }

        //TODO: Return schema default value
        Some(Value::default_for_schema(&property_schema, schema_set))
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

        let property_schema = super::property_schema_and_path_ancestors_to_check(
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
