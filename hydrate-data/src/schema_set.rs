use crate::value::ValueEnum;
use crate::{
    DataSetError, DataSetResult, HashMap, SchemaFingerprint, SchemaLinker, SchemaLinkerResult,
    SchemaNamedType, Value,
};
use std::sync::Arc;
use uuid::Uuid;

/// Accumulates linked types and can be used to create a schema. This allows validation of types
/// and some work that can be pre-cached, such as generating default values for enums. (Values
/// are not a concept that exists in the hydrate-schema crate)
#[derive(Default)]
pub struct SchemaSetBuilder {
    schemas_by_type_uuid: HashMap<Uuid, SchemaFingerprint>,
    schemas_by_name: HashMap<String, SchemaFingerprint>,
    schemas: HashMap<SchemaFingerprint, SchemaNamedType>,
    default_enum_values: HashMap<SchemaFingerprint, Value>,
}

impl SchemaSetBuilder {
    pub fn build(self) -> SchemaSet {
        let inner = SchemaSetInner {
            schemas_by_type_uuid: self.schemas_by_type_uuid,
            schemas_by_name: self.schemas_by_name,
            schemas: self.schemas,
            default_enum_values: self.default_enum_values,
        };

        SchemaSet {
            inner: Arc::new(inner),
        }
    }

    pub fn add_linked_types(
        &mut self,
        linker: SchemaLinker,
    ) -> SchemaLinkerResult<()> {
        let linked = linker.link_schemas()?;

        //TODO: check no name collisions and merge with DB

        for (k, v) in linked.schemas {
            if let Some(enum_schema) = v.try_as_enum() {
                let default_value = Value::Enum(ValueEnum::new(
                    enum_schema.default_value().name().to_string(),
                ));
                let old = self.default_enum_values.insert(k, default_value.clone());
                if let Some(old) = old {
                    assert_eq!(old.as_enum().unwrap(), default_value.as_enum().unwrap());
                }
            }
            let v_fingerprint = v.fingerprint();
            let old = self.schemas.insert(k, v);
            if let Some(old) = old {
                assert_eq!(old.fingerprint(), v_fingerprint);
            }
        }

        for (k, v) in linked.schemas_by_name {
            let old = self.schemas_by_name.insert(k, v);
            assert!(old.is_none());
        }

        for (k, v) in linked.schemas_by_type_uuid {
            let old = self.schemas_by_type_uuid.insert(k, v);
            assert!(old.is_none());
        }

        Ok(())
    }

    pub fn restore_named_types(
        &mut self,
        named_types: Vec<SchemaNamedType>,
    ) {
        for named_type in named_types {
            self.schemas.insert(named_type.fingerprint(), named_type);
        }
    }
}

pub struct SchemaSetInner {
    schemas_by_type_uuid: HashMap<Uuid, SchemaFingerprint>,
    schemas_by_name: HashMap<String, SchemaFingerprint>,
    schemas: HashMap<SchemaFingerprint, SchemaNamedType>,
    default_enum_values: HashMap<SchemaFingerprint, Value>,
}

#[derive(Clone)]
pub struct SchemaSet {
    inner: Arc<SchemaSetInner>,
}

impl SchemaSet {
    pub fn schemas(&self) -> &HashMap<SchemaFingerprint, SchemaNamedType> {
        &self.inner.schemas
    }

    pub fn schemas_by_type_uuid(&self) -> &HashMap<Uuid, SchemaFingerprint> {
        &self.inner.schemas_by_type_uuid
    }

    pub fn default_value_for_enum(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> Option<&Value> {
        self.inner.default_enum_values.get(&fingerprint)
    }

    pub fn find_named_type_by_type_uuid(
        &self,
        type_uuid: Uuid,
    ) -> DataSetResult<&SchemaNamedType> {
        Ok(self
            .try_find_named_type_by_type_uuid(type_uuid)
            .ok_or(DataSetError::SchemaNotFound)?)
    }

    pub fn try_find_named_type_by_type_uuid(
        &self,
        type_uuid: Uuid,
    ) -> Option<&SchemaNamedType> {
        self.inner
            .schemas_by_type_uuid
            .get(&type_uuid)
            .map(|fingerprint| self.find_named_type_by_fingerprint(*fingerprint))
            .flatten()
    }

    pub fn find_named_type(
        &self,
        name: impl AsRef<str>,
    ) -> DataSetResult<&SchemaNamedType> {
        Ok(self
            .try_find_named_type(name)
            .ok_or(DataSetError::SchemaNotFound)?)
    }

    pub fn try_find_named_type(
        &self,
        name: impl AsRef<str>,
    ) -> Option<&SchemaNamedType> {
        self.inner
            .schemas_by_name
            .get(name.as_ref())
            .map(|fingerprint| self.find_named_type_by_fingerprint(*fingerprint))
            .flatten()
    }

    pub fn find_named_type_by_fingerprint(
        &self,
        fingerprint: SchemaFingerprint,
    ) -> Option<&SchemaNamedType> {
        self.inner.schemas.get(&fingerprint)
    }
}
