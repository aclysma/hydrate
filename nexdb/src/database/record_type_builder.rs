use crate::{Schema, SchemaRecord};

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

    pub fn add_f32(&mut self, name: impl Into<String>) -> &mut RecordTypeFieldBuilder {
        self.fields.push(RecordTypeFieldBuilder {
            field_type: Schema::F32,
            aliases: Default::default(),
            name: name.into(),
        });
        self.fields.last_mut().unwrap()
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