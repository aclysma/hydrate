use super::schema_def::{SchemaDefDynamicArray, SchemaDefType};

pub struct RecordTypeFieldBuilder {
    pub(super) name: String,
    pub(super) aliases: Vec<String>,
    pub(super) field_type: SchemaDefType,
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
        inner_schema: SchemaDefType,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::Nullable(Box::new(inner_schema)),
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_boolean(
        &mut self,
        name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::Boolean,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_i32(
        &mut self,
        name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::I32,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_i64(
        &mut self,
        name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::I64,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_u32(
        &mut self,
        name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::U32,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_u64(
        &mut self,
        name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::U64,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_f32(
        &mut self,
        name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::F32,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_f64(
        &mut self,
        name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::F64,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_string(
        &mut self,
        name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::String,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_dynamic_array(
        &mut self,
        name: impl Into<String>,
        schema: SchemaDefType,
    ) {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::DynamicArray(SchemaDefDynamicArray::new(Box::new(schema))),
            aliases: Default::default(),
            name: name.into(),
        });
    }

    pub fn add_struct(
        &mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
    ) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: SchemaDefType::NamedType(type_name.into()),
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }
}
