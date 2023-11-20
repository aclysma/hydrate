use crate::{
    AssetId, HashMap, HashSet, OrderedSet, Schema, SchemaFingerprint, SchemaRecord, SingleObject,
    Value,
};
pub use crate::{DataSetError, DataSetResult};
use crate::{NullOverride, SchemaSet};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::ToString;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetName(String);

impl AssetName {
    pub fn new<T: Into<String>>(name: T) -> Self {
        AssetName(name.into())
    }

    pub fn empty() -> Self {
        AssetName(String::default())
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct AssetLocation {
    path_node_id: AssetId,
}

impl AssetLocation {
    pub fn new(path_node_id: AssetId) -> Self {
        AssetLocation { path_node_id }
    }

    pub fn null() -> AssetLocation {
        AssetLocation {
            path_node_id: AssetId::null(),
        }
    }

    pub fn path_node_id(&self) -> AssetId {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImporterId(pub Uuid);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BuilderId(pub usize);

// This newtype ensures we do not allow both a None and a Some("") importable name
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct ImportableName(String);

impl ImportableName {
    // In some cases in serialization it is convenient to let empty string imply a None importable
    // name. This constructor makes this convenient.
    pub fn new(name: String) -> Self {
        ImportableName(name)
    }

    // Does not accept Some("") as this is ambiguous between a None name and an empty string name.
    // "" is not an allowed importable name.
    pub fn new_optional(name: Option<String>) -> Self {
        if let Some(name) = name {
            assert!(!name.is_empty());
            ImportableName(name)
        } else {
            ImportableName(String::default())
        }
    }

    pub fn name(&self) -> Option<&str> {
        if self.0.is_empty() {
            None
        } else {
            Some(&self.0)
        }
    }

    pub fn is_default(&self) -> bool {
        self.0.is_empty()
    }
}

/// Describes the conditions that we imported the file
#[derive(Clone, Debug)]
pub struct ImportInfo {
    // Set on initial import
    importer_id: ImporterId,

    // Set on initial import, or re-import. This affects the import step.
    // Anything that just affects the build step should be an asset property instead.
    // Removed for now as I don't have a practical use for it right now. Generally I think we could
    // lean towards importing everything and using asset properties to make the build step produce
    // less data if we don't want everything.
    //import_options: SingleObject,

    // Set on initial import, or re-import. Used to monitor to detect stale imported data and
    // automaticlaly re-import, and as a heuristic when importing other files that reference this
    // file to link to this asset rather than importing another copy.
    source_file_path: PathBuf,

    // If the asset comes from a file with more than one importable thing, we require a string key
    // to specify which importable this asset represents.
    importable_name: ImportableName,

    // All the file references that need to be resolved in order to build the asset (this represents
    // file references encountered in the input data, and only changes when data is re-imported)
    file_references: Vec<PathBuf>,
}

impl ImportInfo {
    pub fn new(
        importer_id: ImporterId,
        source_file_path: PathBuf,
        importable_name: ImportableName,
        file_references: Vec<PathBuf>,
    ) -> Self {
        ImportInfo {
            importer_id,
            source_file_path,
            importable_name,
            file_references,
        }
    }

    pub fn importer_id(&self) -> ImporterId {
        self.importer_id
    }

    pub fn source_file_path(&self) -> &Path {
        &self.source_file_path
    }

    pub fn importable_name(&self) -> Option<&str> {
        self.importable_name.name()
    }

    pub fn file_references(&self) -> &[PathBuf] {
        &self.file_references
    }
}

/// Affects how we build the file. However most of the time use asset properties instead. The only
/// things in here should be system-level configuration that is relevant to any asset type
#[derive(Clone, Debug, Default)]
pub struct BuildInfo {
    // Imported files often reference other files. During import, referenced files will also be
    // imported. We maintain the correlation between paths and imported asset ID here for use when
    // processing the imported data.
    pub file_reference_overrides: HashMap<PathBuf, AssetId>,
}

/// The full state of a single asset in a dataset
#[derive(Clone, Debug)]
pub struct DataSetAssetInfo {
    schema: SchemaRecord,

    pub(super) asset_name: AssetName,
    pub(super) asset_location: AssetLocation,

    // Stores the configuration/choices that were made when the asset was last imported
    pub(super) import_info: Option<ImportInfo>,
    pub(super) build_info: BuildInfo,

    pub(super) prototype: Option<AssetId>,
    pub(super) properties: HashMap<String, Value>,
    pub(super) property_null_overrides: HashMap<String, NullOverride>,
    pub(super) properties_in_replace_mode: HashSet<String>,
    pub(super) dynamic_array_entries: HashMap<String, OrderedSet<Uuid>>,
}

impl DataSetAssetInfo {
    pub fn schema(&self) -> &SchemaRecord {
        &self.schema
    }

    pub fn asset_name(&self) -> &AssetName {
        &self.asset_name
    }

    pub fn asset_location(&self) -> &AssetLocation {
        &self.asset_location
    }

    pub fn import_info(&self) -> &Option<ImportInfo> {
        &self.import_info
    }

    pub fn build_info(&self) -> &BuildInfo {
        &self.build_info
    }

    pub fn prototype(&self) -> Option<AssetId> {
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

    pub fn dynamic_array_entries(&self) -> &HashMap<String, OrderedSet<Uuid>> {
        &self.dynamic_array_entries
    }
}

/// A collection of assets. Methods support serializing/deserializing, resolving property values,
/// etc. This includes being aware of schema and prototypes.
#[derive(Default, Clone)]
pub struct DataSet {
    assets: HashMap<AssetId, DataSetAssetInfo>,
}

impl DataSet {
    pub fn assets(&self) -> &HashMap<AssetId, DataSetAssetInfo> {
        &self.assets
    }

    // Exposed to allow diffs to apply changes
    pub(super) fn assets_mut(&mut self) -> &mut HashMap<AssetId, DataSetAssetInfo> {
        &mut self.assets
    }

    pub fn take_assets(self) -> HashMap<AssetId, DataSetAssetInfo> {
        self.assets
    }

    // Inserts the asset but only if the ID is not already in use
    fn insert_asset(
        &mut self,
        id: AssetId,
        obj_info: DataSetAssetInfo,
    ) -> DataSetResult<()> {
        if self.assets.contains_key(&id) {
            Err(DataSetError::DuplicateAssetId)
        } else {
            let old = self.assets.insert(id, obj_info);
            assert!(old.is_none());
            Ok(())
        }
    }

    /// Creates the asset, overwriting it if it already exists
    pub fn restore_asset(
        &mut self,
        asset_id: AssetId,
        asset_name: AssetName,
        asset_location: AssetLocation,
        import_info: Option<ImportInfo>,
        build_info: BuildInfo,
        schema_set: &SchemaSet,
        prototype: Option<AssetId>,
        schema: SchemaFingerprint,
        properties: HashMap<String, Value>,
        property_null_overrides: HashMap<String, NullOverride>,
        properties_in_replace_mode: HashSet<String>,
        dynamic_array_entries: HashMap<String, OrderedSet<Uuid>>,
    ) -> DataSetResult<()> {
        let schema = schema_set
            .schemas()
            .get(&schema)
            .ok_or(DataSetError::SchemaNotFound)?;
        let schema_record = schema.as_record().cloned()?;
        let obj = DataSetAssetInfo {
            schema: schema_record,
            asset_name,
            asset_location,
            import_info,
            build_info,
            prototype,
            properties,
            property_null_overrides,
            properties_in_replace_mode,
            dynamic_array_entries,
        };

        self.assets.insert(asset_id, obj);
        Ok(())
    }

    /// Creates an asset with a particular ID with no properties set. Fails if the asset ID is already
    /// in use.
    pub fn new_asset_with_id(
        &mut self,
        asset_id: AssetId,
        asset_name: AssetName,
        asset_location: AssetLocation,
        schema: &SchemaRecord,
    ) -> DataSetResult<()> {
        let obj = DataSetAssetInfo {
            schema: schema.clone(),
            asset_name: asset_name,
            asset_location: asset_location,
            import_info: None,
            build_info: Default::default(),
            prototype: None,
            properties: Default::default(),
            property_null_overrides: Default::default(),
            properties_in_replace_mode: Default::default(),
            dynamic_array_entries: Default::default(),
        };

        self.insert_asset(asset_id, obj)
    }

    /// Creates a new asset with no properties set. Uses a unique UUID and should not fail
    pub fn new_asset(
        &mut self,
        asset_name: AssetName,
        asset_location: AssetLocation,
        schema: &SchemaRecord,
    ) -> AssetId {
        let id = AssetId::from_uuid(Uuid::new_v4());

        // The unwrap here is safe because a duplicate UUID is statistically very unlikely
        self.new_asset_with_id(id, asset_name, asset_location, schema)
            .expect("Randomly created UUID collided with existing UUID");

        id
    }

    /// Creates a new asset and sets it to use the given prototype asset ID as the new object's prototype
    /// May fail if the prototype asset is not found
    pub fn new_asset_from_prototype(
        &mut self,
        asset_name: AssetName,
        asset_location: AssetLocation,
        prototype_asset_id: AssetId,
    ) -> DataSetResult<AssetId> {
        let prototype_schema = self
            .assets
            .get(&prototype_asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

        let id = self.new_asset(
            asset_name,
            asset_location,
            &prototype_schema.schema().clone(),
        );
        self.assets
            .get_mut(&id)
            .expect("Newly created asset was not found")
            .prototype = Some(prototype_asset_id);
        Ok(id)
    }

    /// Populate an empty asset with data from a SingleObject. The asset should already exist, and
    /// the schema must match.
    pub fn copy_from_single_object(
        &mut self,
        asset_id: AssetId,
        single_object: &SingleObject,
    ) -> DataSetResult<()> {
        let asset = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

        if asset.schema.fingerprint() != single_object.schema().fingerprint() {
            return Err(DataSetError::SingleObjectDoesNotMatchSchema);
        };

        // Reset the state
        asset.prototype = None;
        asset.properties.clear();
        asset.property_null_overrides.clear();
        asset.properties_in_replace_mode.clear();
        asset.dynamic_array_entries.clear();

        for (property, value) in single_object.properties() {
            asset.properties.insert(property.clone(), value.clone());
        }

        for (property, null_override) in single_object.property_null_overrides() {
            asset
                .property_null_overrides
                .insert(property.clone(), *null_override);
        }

        for (property, dynamic_array_entries) in single_object.dynamic_array_entries() {
            let property_entry = asset
                .dynamic_array_entries
                .entry(property.clone())
                .or_default();
            for element in &*dynamic_array_entries {
                let is_newly_inserted = property_entry.try_insert_at_end(*element);
                // elements are UUIDs and they should have been unique
                assert!(is_newly_inserted);
            }
        }

        Ok(())
    }

    /// Returns error if asset did not exist
    pub fn delete_asset(
        &mut self,
        asset_id: AssetId,
    ) -> DataSetResult<()> {
        if self.assets.remove(&asset_id).is_none() {
            Err(DataSetError::AssetNotFound)
        } else {
            Ok(())
        }
    }

    /// Returns error if asset does not exist
    pub fn set_asset_location(
        &mut self,
        asset_id: AssetId,
        new_location: AssetLocation,
    ) -> DataSetResult<()> {
        let asset = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

        asset.asset_location = new_location;
        Ok(())
    }

    /// Returns error if asset does not exist
    pub fn set_import_info(
        &mut self,
        asset_id: AssetId,
        import_info: ImportInfo,
    ) -> DataSetResult<()> {
        let asset = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

        asset.import_info = Some(import_info);
        Ok(())
    }

    /// Returns error if other asset does not exist. This will create or overwrite the asset in this
    /// dataset and does not require that the schema be the same if the asset already existed. No
    /// validation is performed to ensure that references to other assets or the prototype exist.
    pub fn copy_from(
        &mut self,
        other: &DataSet,
        asset_id: AssetId,
    ) -> DataSetResult<()> {
        let asset = other
            .assets
            .get(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

        self.assets.insert(asset_id, asset.clone());
        Ok(())
    }

    /// Returns the asset name, or none if the asset was not found
    pub fn asset_name(
        &self,
        asset_id: AssetId,
    ) -> Option<&AssetName> {
        self.assets.get(&asset_id).map(|x| &x.asset_name)
    }

    /// Sets the asset's name, fails if the asset does not exist
    pub fn set_asset_name(
        &mut self,
        asset_id: AssetId,
        asset_name: AssetName,
    ) -> DataSetResult<()> {
        let asset = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

        asset.asset_name = asset_name;
        Ok(())
    }

    /// Returns the asset's parent or none if the asset does not exist
    pub fn asset_location(
        &self,
        asset_id: AssetId,
    ) -> Option<&AssetLocation> {
        self.assets.get(&asset_id).map(|x| &x.asset_location)
    }

    /// Returns the asset locations from the parent all the way up to the root parent. If a cycle is
    /// detected or any elements in the chain are not found, an error is returned
    pub fn asset_location_chain(
        &self,
        asset_id: AssetId,
    ) -> DataSetResult<Vec<AssetLocation>> {
        let mut asset_location_chain = Vec::default();

        // If this asset's location is none, return an empty list
        let Some(mut obj_iter) = self.asset_location(asset_id).cloned() else {
            return Ok(asset_location_chain);
        };

        // Iterate up the chain
        while !obj_iter.path_node_id.is_null() {
            if asset_location_chain.contains(&obj_iter) {
                // Detected a cycle, return an empty list
                return Err(DataSetError::LocationCycleDetected);
            }

            asset_location_chain.push(obj_iter.clone());
            obj_iter = if let Some(location) = self.asset_location(obj_iter.path_node_id).cloned() {
                // May be null, in which case we will terminate and return this list so far not including the null
                location
            } else {
                // The parent was specified but not found, default to empty list if the chain is in a bad state
                return Err(DataSetError::LocationParentNotFound);
            };
        }

        Ok(asset_location_chain)
    }

    /// Gets the import info, returns None if the asset does not exist or there is no import info
    /// associated with the asset
    pub fn import_info(
        &self,
        asset_id: AssetId,
    ) -> Option<&ImportInfo> {
        self.assets
            .get(&asset_id)
            .map(|x| x.import_info.as_ref())
            .flatten()
    }

    fn do_resolve_all_file_references(
        &self,
        asset_id: AssetId,
        all_references: &mut HashMap<PathBuf, AssetId>,
    ) -> DataSetResult<()> {
        let asset = self.assets.get(&asset_id);
        if let Some(asset) = asset {
            if let Some(prototype) = asset.prototype {
                self.do_resolve_all_file_references(prototype, all_references)?;
            }

            for (k, v) in &asset.build_info.file_reference_overrides {
                all_references.insert(k.clone(), *v);
            }
        } else {
            return Err(DataSetError::AssetNotFound);
        }

        Ok(())
    }

    pub fn resolve_all_file_references(
        &self,
        asset_id: AssetId,
    ) -> DataSetResult<HashMap<PathBuf, AssetId>> {
        let mut all_references = HashMap::default();
        self.do_resolve_all_file_references(asset_id, &mut all_references)?;
        Ok(all_references)
    }

    pub fn get_all_file_reference_overrides(
        &mut self,
        asset_id: AssetId,
    ) -> Option<&HashMap<PathBuf, AssetId>> {
        self.assets
            .get(&asset_id)
            .map(|x| &x.build_info.file_reference_overrides)
    }

    pub fn set_file_reference_override(
        &mut self,
        asset_id: AssetId,
        path: PathBuf,
        referenced_asset_id: AssetId,
    ) -> DataSetResult<()> {
        let asset = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

        asset
            .build_info
            .file_reference_overrides
            .insert(path, referenced_asset_id);
        Ok(())
    }

    pub fn asset_prototype(
        &self,
        asset_id: AssetId,
    ) -> Option<AssetId> {
        self.assets.get(&asset_id).map(|x| x.prototype).flatten()
    }

    pub fn asset_schema(
        &self,
        asset_id: AssetId,
    ) -> Option<&SchemaRecord> {
        self.assets.get(&asset_id).map(|x| &x.schema)
    }

    /// This is intended to just hash the properties of the object. Things like name, location,
    /// import info, build info are not considered. (This may change)
    pub fn hash_properties(
        &self,
        asset_id: AssetId,
    ) -> Option<u64> {
        let asset = self.assets.get(&asset_id)?;
        let schema = &asset.schema;

        let mut hasher = siphasher::sip::SipHasher::default();

        schema.fingerprint().hash(&mut hasher);

        // We ignore the following properties for now
        //asset_name
        //asset_location
        //import_info
        //build_info

        if let Some(prototype) = asset.prototype {
            // We may fail to find the prototype, there is a good chance this means our data is in
            // a bad state, but it is not considered fatal. Generally in these circumstances we
            // carry on as if the prototype was set to None.
            self.hash_properties(prototype).hash(&mut hasher);
        }

        // properties
        let mut properties_hash = 0;
        for (key, value) in &asset.properties {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            key.hash(&mut inner_hasher);
            value.hash(&mut inner_hasher);
            properties_hash = properties_hash ^ inner_hasher.finish();
        }
        properties_hash.hash(&mut hasher);

        // property_null_overrides
        let mut property_null_overrides_hash = 0;
        for (key, value) in &asset.property_null_overrides {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            key.hash(&mut inner_hasher);
            value.hash(&mut inner_hasher);
            property_null_overrides_hash = property_null_overrides_hash ^ inner_hasher.finish();
        }
        property_null_overrides_hash.hash(&mut hasher);

        // properties_in_replace_mode
        let mut properties_in_replace_mode_hash = 0;
        for value in &asset.properties_in_replace_mode {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            value.hash(&mut inner_hasher);
            properties_in_replace_mode_hash =
                properties_in_replace_mode_hash ^ inner_hasher.finish();
        }
        properties_in_replace_mode_hash.hash(&mut hasher);

        // dynamic_array_entries
        let mut dynamic_array_entries_hash = 0;
        for (key, value) in &asset.dynamic_array_entries {
            let mut inner_hasher = siphasher::sip::SipHasher::default();
            key.hash(&mut inner_hasher);

            let mut uuid_set_hash = 0;
            for id in value.iter() {
                let mut inner_hasher2 = siphasher::sip::SipHasher::default();
                id.hash(&mut inner_hasher2);
                uuid_set_hash = uuid_set_hash ^ inner_hasher2.finish();
            }
            uuid_set_hash.hash(&mut inner_hasher);

            dynamic_array_entries_hash = dynamic_array_entries_hash ^ inner_hasher.finish();
        }
        dynamic_array_entries_hash.hash(&mut hasher);

        let asset_hash = hasher.finish();
        Some(asset_hash)
    }

    /// Gets if the property has a null override associated with it *on this object* ignoring the
    /// prototype. An error will be returned if the asset doesn't exist, the schema doesn't exist,
    /// or if this field is not nullable
    pub fn get_null_override(
        &self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        let asset = self
            .assets
            .get(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let property_schema = asset
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if property_schema.is_nullable() {
            // Not existing in the map implies that it is unset
            Ok(asset
                .property_null_overrides
                .get(path.as_ref())
                .copied()
                .unwrap_or(NullOverride::Unset))
        } else {
            Err(DataSetError::InvalidSchema)
        }
    }

    /// Sets or removes the null override state on this object.
    pub fn set_null_override(
        &mut self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
        null_override: NullOverride,
    ) -> DataSetResult<()> {
        let asset = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let property_schema = asset
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if property_schema.is_nullable() {
            if null_override != NullOverride::Unset {
                asset
                    .property_null_overrides
                    .insert(path.as_ref().to_string(), null_override);
            } else {
                // Not existing in the map implies that it is unset
                asset.property_null_overrides.remove(path.as_ref());
            }
            Ok(())
        } else {
            Err(DataSetError::InvalidSchema)
        }
    }

    // None return means something higher in property hierarchy is null or non-existing
    pub fn resolve_null_override(
        &self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<NullOverride> {
        let asset_schema = self
            .asset_schema(asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

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
            asset_schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )?;

        // This field is not nullable, return an error
        if !property_schema.is_nullable() {
            return Err(DataSetError::InvalidSchema);
        }

        // See if this field was contained in any nullables. If any of those were null, return None.
        for checked_property in &nullable_ancestors {
            if self.resolve_null_override(schema_set, asset_id, checked_property)?
                != NullOverride::SetNonNull
            {
                return Err(DataSetError::PathParentIsNull);
            }
        }

        // See if this field was contained in a container. If any of those containers didn't contain
        // this property path, return None
        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, asset_id, path)?;
            if !dynamic_array_entries
                .contains(&Uuid::from_str(key).map_err(|_| DataSetError::UuidParseError)?)
            {
                return Err(DataSetError::PathDynamicArrayEntryDoesNotExist);
            }
        }

        // Recursively look for a null override for this property being set. We can make a call
        let mut prototype_id = Some(asset_id);
        while let Some(prototype_id_iter) = prototype_id {
            let obj = self
                .assets
                .get(&prototype_id_iter)
                .ok_or(DataSetError::AssetNotFound)?;

            if let Some(value) = obj.property_null_overrides.get(path.as_ref()) {
                match value {
                    // We do not put NullOverride::Unset in the property_null_overrides map
                    NullOverride::Unset => unreachable!(),
                    NullOverride::SetNull => return Ok(NullOverride::SetNull),
                    NullOverride::SetNonNull => return Ok(NullOverride::SetNonNull),
                }
            }

            prototype_id = obj.prototype;
        }

        // By default
        Ok(NullOverride::Unset)
    }

    pub fn has_property_override(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<bool> {
        Ok(self.get_property_override(asset_id, path)?.is_some())
    }

    // Just gets if this asset has a property without checking prototype chain for fallback or returning a default
    // Returning none means it is not overridden
    pub fn get_property_override(
        &self,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<Option<&Value>> {
        let asset = self
            .assets
            .get(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        Ok(asset.properties.get(path.as_ref()))
    }

    // Just sets a property on this asset, making it overridden, or replacing the existing override
    pub fn set_property_override(
        &mut self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
        value: Option<Value>,
    ) -> DataSetResult<Option<Value>> {
        let asset_schema = self
            .asset_schema(asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let property_schema = asset_schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if let Some(value) = &value {
            if !value.matches_schema(&property_schema, schema_set.schemas()) {
                log::debug!(
                    "Value {:?} doesn't match schema {:?}",
                    value,
                    property_schema
                );
                return Err(DataSetError::ValueDoesNotMatchSchema);
            }
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
            asset_schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )?;

        for checked_property in &nullable_ancestors {
            if self.resolve_null_override(schema_set, asset_id, checked_property)?
                != NullOverride::SetNonNull
            {
                return Err(DataSetError::PathParentIsNull);
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, asset_id, path)?;
            if !dynamic_array_entries
                .contains(&Uuid::from_str(key).map_err(|_| DataSetError::UuidParseError)?)
            {
                return Err(DataSetError::PathDynamicArrayEntryDoesNotExist);
            }
        }

        let obj = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let old_value = if let Some(value) = value {
            obj.properties.insert(path.as_ref().to_string(), value)
        } else {
            obj.properties.remove(path.as_ref())
        };
        Ok(old_value)
    }

    pub fn apply_property_override_to_prototype(
        &mut self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<()> {
        let asset = self
            .assets
            .get(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let prototype_id = asset.prototype;

        if let Some(prototype_id) = prototype_id {
            let v = self.set_property_override(schema_set, asset_id, path.as_ref(), None)?;
            if let Some(v) = v {
                // The property existed on the child, set it on the prototype
                self.set_property_override(schema_set, prototype_id, path, Some(v))?;
            } else {
                // The property didn't exist on the child, do nothing
            }
        } else {
            // The asset has no prototype, do nothing
        }

        Ok(())
    }

    pub fn resolve_property<'a>(
        &'a self,
        schema_set: &'a SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<&'a Value> {
        let asset_schema = self
            .asset_schema(asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        let property_schema = super::property_schema_and_path_ancestors_to_check(
            asset_schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )?;

        for checked_property in &nullable_ancestors {
            if self.resolve_null_override(schema_set, asset_id, checked_property)?
                != NullOverride::SetNonNull
            {
                return Err(DataSetError::PathParentIsNull);
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, asset_id, path)?;
            if !dynamic_array_entries
                .contains(&Uuid::from_str(key).map_err(|_| DataSetError::UuidParseError)?)
            {
                return Err(DataSetError::PathDynamicArrayEntryDoesNotExist);
            }
        }

        let mut prototype_id = Some(asset_id);
        while let Some(prototype_id_iter) = prototype_id {
            let obj = self.assets.get(&prototype_id_iter);
            if let Some(obj) = obj {
                if let Some(value) = obj.properties.get(path.as_ref()) {
                    return Ok(value);
                }

                prototype_id = obj.prototype;
            } else {
                // The prototype being referenced was not found, break out of the loop and pretend
                // like the prototype is unset
                prototype_id = None;
            }
        }

        Ok(Value::default_for_schema(&property_schema, schema_set))
    }

    pub fn get_dynamic_array_overrides(
        &self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<std::slice::Iter<Uuid>> {
        let asset = self
            .assets
            .get(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let property_schema = asset
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_dynamic_array() {
            return Err(DataSetError::InvalidSchema);
        }

        if let Some(overrides) = asset.dynamic_array_entries.get(path.as_ref()) {
            Ok(overrides.iter())
        } else {
            Ok(std::slice::Iter::default())
        }
    }

    pub fn add_dynamic_array_override(
        &mut self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<Uuid> {
        let asset = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let property_schema = asset
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_dynamic_array() {
            return Err(DataSetError::InvalidSchema);
        }

        let entry = asset
            .dynamic_array_entries
            .entry(path.as_ref().to_string())
            .or_insert(Default::default());
        let new_uuid = Uuid::new_v4();
        let newly_inserted = entry.try_insert_at_end(new_uuid);
        if !newly_inserted {
            panic!("Created a new random UUID but it matched an existing UUID");
        }
        Ok(new_uuid)
    }

    pub fn remove_dynamic_array_override(
        &mut self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
        element_id: Uuid,
    ) -> DataSetResult<bool> {
        let asset = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let property_schema = asset
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        if !property_schema.is_dynamic_array() {
            return Err(DataSetError::InvalidSchema);
        }

        if let Some(override_list) = asset.dynamic_array_entries.get_mut(path.as_ref()) {
            // Return if the override existed or not
            let was_removed = override_list.remove(&element_id);
            Ok(was_removed)
        } else {
            // The override didn't exist
            Ok(false)
        }
    }

    pub fn do_resolve_dynamic_array(
        &self,
        asset_id: AssetId,
        path: &str,
        nullable_ancestors: &Vec<String>,
        dynamic_array_ancestors: &Vec<String>,
        map_ancestors: &Vec<String>,
        accessed_dynamic_array_keys: &Vec<(String, String)>,
        resolved_entries: &mut Vec<Uuid>,
    ) -> DataSetResult<()> {
        let obj = self
            .assets
            .get(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

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
                // If the prototype is not found, we behave as if the prototype was not set
                if self.assets.contains_key(&prototype) {
                    self.do_resolve_dynamic_array(
                        prototype,
                        path,
                        nullable_ancestors,
                        dynamic_array_ancestors,
                        map_ancestors,
                        accessed_dynamic_array_keys,
                        resolved_entries,
                    )?;
                }
            }
        }

        if let Some(entries) = obj.dynamic_array_entries.get(path) {
            for entry in entries {
                resolved_entries.push(*entry);
            }
        }

        Ok(())
    }

    pub fn resolve_dynamic_array(
        &self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<Box<[Uuid]>> {
        let asset_schema = self
            .asset_schema(asset_id)
            .ok_or(DataSetError::AssetNotFound)?;

        // Contains the path segments that we need to check for being null
        let mut nullable_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut dynamic_array_ancestors = vec![];
        // Contains the path segments that we need to check for being in append mode
        let mut map_ancestors = vec![];
        // Contains the dynamic arrays we access and what keys are used to access them
        let mut accessed_dynamic_array_keys = vec![];

        let _property_schema = super::property_schema_and_path_ancestors_to_check(
            asset_schema,
            &path,
            schema_set.schemas(),
            &mut nullable_ancestors,
            &mut dynamic_array_ancestors,
            &mut map_ancestors,
            &mut accessed_dynamic_array_keys,
        )?;

        for checked_property in &nullable_ancestors {
            if self.resolve_null_override(schema_set, asset_id, checked_property)?
                != NullOverride::SetNonNull
            {
                return Err(DataSetError::PathParentIsNull);
            }
        }

        for (path, key) in &accessed_dynamic_array_keys {
            let dynamic_array_entries = self.resolve_dynamic_array(schema_set, asset_id, path)?;
            if !dynamic_array_entries
                .contains(&Uuid::from_str(key).map_err(|_| DataSetError::UuidParseError)?)
            {
                return Err(DataSetError::PathDynamicArrayEntryDoesNotExist);
            }
        }

        let mut resolved_entries = vec![];
        self.do_resolve_dynamic_array(
            asset_id,
            path.as_ref(),
            &nullable_ancestors,
            &dynamic_array_ancestors,
            &map_ancestors,
            &accessed_dynamic_array_keys,
            &mut resolved_entries,
        )?;
        Ok(resolved_entries.into_boxed_slice())
    }

    pub fn get_override_behavior(
        &self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
    ) -> DataSetResult<OverrideBehavior> {
        let asset = self
            .assets
            .get(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let property_schema = asset
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        Ok(match property_schema {
            Schema::DynamicArray(_) | Schema::Map(_) => {
                if asset.properties_in_replace_mode.contains(path.as_ref()) {
                    OverrideBehavior::Replace
                } else {
                    OverrideBehavior::Append
                }
            }
            _ => OverrideBehavior::Replace,
        })
    }

    pub fn set_override_behavior(
        &mut self,
        schema_set: &SchemaSet,
        asset_id: AssetId,
        path: impl AsRef<str>,
        behavior: OverrideBehavior,
    ) -> DataSetResult<()> {
        let asset = self
            .assets
            .get_mut(&asset_id)
            .ok_or(DataSetError::AssetNotFound)?;
        let property_schema = asset
            .schema
            .find_property_schema(&path, schema_set.schemas())
            .ok_or(DataSetError::SchemaNotFound)?;

        match property_schema {
            Schema::DynamicArray(_) | Schema::Map(_) => {
                let _ = match behavior {
                    OverrideBehavior::Append => {
                        asset.properties_in_replace_mode.remove(path.as_ref())
                    }
                    OverrideBehavior::Replace => asset
                        .properties_in_replace_mode
                        .insert(path.as_ref().to_string()),
                };
                Ok(())
            }
            _ => Err(DataSetError::InvalidSchema),
        }
    }
}
