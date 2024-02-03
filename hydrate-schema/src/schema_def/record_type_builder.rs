use super::schema_def::{SchemaDefDynamicArray, SchemaDefType};
use crate::{SchemaDefRecordFieldMarkup, SchemaDefRecordMarkup};
use uuid::Uuid;

pub struct RecordTypeFieldBuilder {
    pub(super) name: String,
    pub(super) field_uuid: Uuid,
    pub(super) aliases: Vec<String>,
    pub(super) field_type: SchemaDefType,
    pub(super) markup: SchemaDefRecordFieldMarkup,
}

impl RecordTypeFieldBuilder {
    pub fn add_field_alias(
        &mut self,
        alias: impl Into<String>,
    ) {
        self.aliases.push(alias.into());
    }
}

#[derive(Default)]
pub struct RecordTypeBuilder {
    pub(super) aliases: Vec<String>,
    pub(super) fields: Vec<RecordTypeFieldBuilder>,
    pub(super) markup: SchemaDefRecordMarkup,
}

impl RecordTypeBuilder {
    pub fn add_type_alias(
        &mut self,
        alias: impl Into<String>,
    ) {
        self.aliases.push(alias.into())
    }

    pub fn add_nullable(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
        inner_schema: SchemaDefType,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::Nullable(Box::new(inner_schema)),
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_boolean(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::Boolean,
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_i32(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::I32,
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_i64(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::I64,
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_u32(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::U32,
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_u64(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::U64,
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_f32(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::F32,
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_f64(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::F64,
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_bytes(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::Bytes,
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_reference(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
        type_name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::AssetRef(type_name.into()),
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_string(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::String,
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_dynamic_array(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
        schema: SchemaDefType,
    ) {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::DynamicArray(SchemaDefDynamicArray::new(Box::new(schema))),
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
    }

    pub fn add_named_type(
        &mut self,
        name: impl Into<String>,
        field_uuid: Uuid,
        type_name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::NamedType(type_name.into()),
            field_uuid,
            aliases: Default::default(),
            name: name.into(),
            markup: Default::default(),
        });
        self.fields.last_mut().unwrap()
    }
}
