use crate::{Schema, SchemaDynamicArray, SchemaRecord};

pub struct RecordTypeFieldBuilder {
    pub(super) name: String,
    pub(super) aliases: Vec<String>,
    pub(super) field_type: Schema,
}

impl RecordTypeFieldBuilder {
    pub fn add_field_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }
}

#[derive(Default)]
pub struct RecordTypeBuilder {
    pub(super) aliases: Vec<String>,
    pub(super) fields: Vec<RecordTypeFieldBuilder>,
}

impl RecordTypeBuilder {
    pub fn add_type_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into())
    }

    pub fn add_nullable(&mut self, name: impl Into<String>, inner_schema: &Schema) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::Nullable(Box::new(inner_schema.clone())),
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_boolean(&mut self, name: impl Into<String>) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::Boolean,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_i32(&mut self, name: impl Into<String>) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::I32,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_i64(&mut self, name: impl Into<String>) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::I64,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_u32(&mut self, name: impl Into<String>) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::U32,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_u64(&mut self, name: impl Into<String>) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::U64,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_f32(&mut self, name: impl Into<String>) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::F32,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_f64(&mut self, name: impl Into<String>) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::F64,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_string(&mut self, name: impl Into<String>) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::String,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }

    pub fn add_dynamic_array(&mut self, name: impl Into<String>, schema: &Schema) {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::DynamicArray(SchemaDynamicArray::new(Box::new(schema.clone()))),
            aliases: Default::default(),
            name: name.into(),
        });
    }

    pub fn add_struct(&mut self, name: impl Into<String>, schema: &SchemaRecord) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::Record(schema.clone()),
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
    }
}