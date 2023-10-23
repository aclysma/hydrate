use crate::{HashMap, Schema, SchemaFingerprint, SchemaLinker, SchemaLinkerResult, SchemaNamedType, Value};
use crate::value::ValueEnum;

#[derive(Default, Clone)]
pub struct SchemaSet {
    schemas_by_name: HashMap<String, SchemaFingerprint>,
    schemas: HashMap<SchemaFingerprint, SchemaNamedType>,
    default_enum_values: HashMap<SchemaFingerprint, Value>,
}

impl SchemaSet {
    pub fn schemas(&self) -> &HashMap<SchemaFingerprint, SchemaNamedType> {
        &self.schemas
    }

    pub fn default_value_for_enum(&self, fingerprint: SchemaFingerprint) -> Option<&Value> {
        self.default_enum_values.get(&fingerprint)
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

    pub fn add_linked_types(
        &mut self,
        linker: SchemaLinker,
    ) -> SchemaLinkerResult<()> {
        let linked = linker.link_schemas()?;

        //TODO: check no name collisions and merge with DB

        for (k, v) in linked.schemas {
            if let Some(enum_schema) = v.as_enum() {
                let old = self.default_enum_values.insert(k, Value::Enum(ValueEnum::new(enum_schema.default_value().name().to_string())));
                //TODO: Assert values are the same
                //assert!(old.is_none());
            }
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

    pub fn restore_named_types(
        &mut self,
        named_types: Vec<SchemaNamedType>,
    ) {
        for named_type in named_types {
            self.schemas.insert(named_type.fingerprint(), named_type);
        }
    }
}