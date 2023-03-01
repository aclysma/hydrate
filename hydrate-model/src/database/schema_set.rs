use crate::{
    HashMap, Schema, SchemaFingerprint, SchemaLinker, SchemaLinkerResult, SchemaNamedType, Value,
};

#[derive(Default, Clone)]
pub struct SchemaSet {
    pub(crate) schemas_by_name: HashMap<String, SchemaFingerprint>,
    pub(crate) schemas: HashMap<SchemaFingerprint, SchemaNamedType>,
}

impl SchemaSet {
    pub fn schemas(&self) -> &HashMap<SchemaFingerprint, SchemaNamedType> {
        &self.schemas
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
    ) -> &Value {
        Value::default_for_schema(schema, &self.schemas)
    }

    pub fn add_linked_types(
        &mut self,
        linker: SchemaLinker,
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

    pub(crate) fn restore_named_types(
        &mut self,
        named_types: Vec<SchemaNamedType>,
    ) {
        for named_type in named_types {
            self.schemas.insert(named_type.fingerprint(), named_type);
        }
    }
}
