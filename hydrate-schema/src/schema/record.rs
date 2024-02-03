use super::Schema;
use crate::{
    HashMap, HashSet, SchemaDefRecordFieldMarkup, SchemaDefRecordMarkup, SchemaFingerprint,
    SchemaNamedType,
};
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct SchemaRecordField {
    name: String,
    aliases: Box<[String]>,
    field_schema: Schema,
    markup: SchemaDefRecordFieldMarkup,
    field_uuid: Uuid,
}

impl SchemaRecordField {
    pub fn new(
        name: String,
        field_uuid: Uuid,
        aliases: Box<[String]>,
        field_schema: Schema,
        markup: SchemaDefRecordFieldMarkup,
    ) -> Self {
        SchemaRecordField {
            name,
            field_uuid,
            aliases,
            field_schema,
            markup,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    pub fn field_schema(&self) -> &Schema {
        &self.field_schema
    }

    pub fn markup(&self) -> &SchemaDefRecordFieldMarkup {
        &self.markup
    }

    pub fn field_uuid(&self) -> Uuid {
        self.field_uuid
    }
}

#[derive(Debug)]
pub struct SchemaRecordInner {
    name: String,
    type_uuid: Uuid,
    fingerprint: SchemaFingerprint,
    aliases: Box<[String]>,
    fields: Box<[SchemaRecordField]>,
    markup: SchemaDefRecordMarkup,
}

#[derive(Clone, Debug)]
pub struct SchemaRecord {
    inner: Arc<SchemaRecordInner>,
}

impl Deref for SchemaRecord {
    type Target = SchemaRecordInner;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl SchemaRecord {
    pub fn new(
        name: String,
        type_uuid: Uuid,
        fingerprint: SchemaFingerprint,
        aliases: Box<[String]>,
        mut fields: Vec<SchemaRecordField>,
        markup: SchemaDefRecordMarkup,
    ) -> Self {
        // Check names are unique
        for i in 0..fields.len() {
            for j in 0..i {
                assert_ne!(fields[i].name, fields[j].name);
            }
        }

        fields.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

        let inner = SchemaRecordInner {
            name,
            type_uuid,
            fingerprint,
            aliases,
            fields: fields.into_boxed_slice(),
            markup,
        };

        SchemaRecord {
            inner: Arc::new(inner),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_uuid(&self) -> Uuid {
        self.type_uuid
    }

    pub fn fingerprint(&self) -> SchemaFingerprint {
        self.fingerprint
    }

    pub fn aliases(&self) -> &[String] {
        &*self.aliases
    }

    pub fn fields(&self) -> &[SchemaRecordField] {
        &*self.fields
    }

    pub fn field_schema(
        &self,
        field_name: impl AsRef<str>,
    ) -> Option<&Schema> {
        for field in &*self.fields {
            if field.name == field_name.as_ref() {
                return Some(&field.field_schema);
            }
        }

        None
    }

    // pub fn find_schemas_used_in_property_path(
    //     &self,
    //     path: impl AsRef<str>,
    //     named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    //     used_schemas: &mut HashSet<SchemaFingerprint>,
    // ) {
    //     SchemaNamedType::Record(self.clone()).find_schemas_used_in_property_path(path, named_types, used_schemas);
    // }

    pub fn find_property_schema(
        &self,
        path: impl AsRef<str>,
        named_types: &HashMap<SchemaFingerprint, SchemaNamedType>,
    ) -> Option<Schema> {
        SchemaNamedType::Record(self.clone()).find_property_schema(path, named_types)
    }

    pub fn find_field_from_name(
        &self,
        field_name: &str,
    ) -> Option<&SchemaRecordField> {
        self.fields.iter().find(|x| x.name == field_name)
    }

    pub fn find_field_from_field_uuid(
        &self,
        field_uuid: Uuid,
    ) -> Option<&SchemaRecordField> {
        self.fields.iter().find(|x| x.field_uuid == field_uuid)
    }

    pub fn markup(&self) -> &SchemaDefRecordMarkup {
        &self.markup
    }
}
