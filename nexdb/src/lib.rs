
#![allow(dead_code)]

pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
pub type HashSet<T> = std::collections::HashMap<T, ahash::RandomState>;

pub struct ObjectId(u128);

mod schema;
pub use schema::*;

mod value;
pub use value::*;

pub struct DatabaseSchemaInfo {
    schema: Schema,
    fingerprint: u128,
}
//
// pub struct DatabaseObjectInfo {
//     schema: Schema,
//
// }

#[derive(Default)]
pub struct Database {
    schemas_by_name: HashMap<String, u128>,
    schemas: HashMap<u128, DatabaseSchemaInfo>,
    //objects: HashMap<u128, Value>
}

pub struct RecordTypeFieldBuilder {
    pub name: String,
    pub aliases: Vec<String>,
    pub field_type: Schema,
}

impl RecordTypeFieldBuilder {
    pub fn add_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }
}

#[derive(Default)]
pub struct RecordTypeBuilder {
    aliases: Vec<String>,
    fields: Vec<RecordTypeFieldBuilder>,
}

impl RecordTypeBuilder {
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

pub struct EnumTypeSymbolBuilder {
    pub name: String,
    pub aliases: Vec<String>,
    pub value: i32,
}

impl EnumTypeSymbolBuilder {
    pub fn add_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }
}

#[derive(Default)]
pub struct EnumTypeBuilder {
    aliases: Vec<String>,
    symbols: Vec<EnumTypeSymbolBuilder>,
}

impl EnumTypeBuilder {
    pub fn add_symbol(&mut self, name: impl Into<String>, value: i32) -> &mut EnumTypeSymbolBuilder {
        self.symbols.push(EnumTypeSymbolBuilder {
            name: name.into(),
            aliases: Default::default(),
            value,
        });
        self.symbols.last_mut().unwrap()
    }
}

impl Database {
    fn register_schema(&mut self, name: impl Into<String>, schema: Schema) {
        let fingerprint = schema.fingerprint128();
        self.schemas.entry(fingerprint).or_insert_with(|| {
            DatabaseSchemaInfo {
                fingerprint,
                schema,
            }
        });
        let old = self.schemas_by_name.insert(name.into(), fingerprint);

        // We do not allow a single schema name to reference different schemas
        assert_eq!(old.unwrap_or(fingerprint), fingerprint);
    }

    pub fn register_record_type<F: Fn(&mut RecordTypeBuilder)>(&mut self, name: impl Into<String>, f: F) -> SchemaRecord {
        let mut builder = RecordTypeBuilder::default();
        (f)(&mut builder);

        let fields: Vec<_> = builder.fields.into_iter().map(|x| {
            SchemaRecordField::new(x.name, x.aliases.into_boxed_slice(), x.field_type)
        }).collect();
        let schema_record = SchemaRecord::new(name.into(), builder.aliases.into_boxed_slice(), fields.into_boxed_slice());

        let schema = Schema::Record(schema_record.clone());
        println!("Registering struct {} {}", schema_record.name(), schema_record.fingerprint_uuid());
        self.register_schema(schema_record.name(), schema);

        schema_record
    }

    pub fn register_enum_type<F: Fn(&mut EnumTypeBuilder)>(&mut self, name: impl Into<String>, f: F) -> SchemaEnum {
        let mut builder = EnumTypeBuilder::default();
        (f)(&mut builder);

        let mut symbols: Vec<_> = builder.symbols.into_iter().map(|x| {
            SchemaEnumSymbol::new(x.name, x.aliases.into_boxed_slice(), x.value)
        }).collect();
        symbols.sort_by_key(|x| x.value());
        let schema_enum = SchemaEnum::new(name.into(), builder.aliases.into_boxed_slice(), symbols.into_boxed_slice());

        let schema = Schema::Enum(schema_enum.clone());
        println!("Registering enum {} {}", schema_enum.name(), schema.fingerprint_uuid());
        self.register_schema(schema_enum.name(), schema);

        schema_enum
    }

    pub fn find_schema_by_name(&self, name: impl AsRef<str>) -> Option<Schema> {
        self.schemas_by_name.get(name.as_ref()).map(|fingerprint| self.find_schema_by_fingerprint(*fingerprint)).flatten()
    }

    pub fn find_schema_by_fingerprint(&self, fingerprint: u128) -> Option<Schema> {
        self.schemas.get(&fingerprint).map(|x| x.schema.clone())
    }
}
