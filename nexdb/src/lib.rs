
#![allow(dead_code)]

extern crate core;

pub type HashMap<K, V> = std::collections::HashMap<K, V, ahash::RandomState>;
pub type HashSet<T> = std::collections::HashMap<T, ahash::RandomState>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct SchemaFingerprint(u128);
impl SchemaFingerprint {
    pub fn as_uuid(&self) -> Uuid {
        Uuid::from_u128(self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ObjectId(u128);
impl ObjectId {
    pub fn null() -> Self {
        ObjectId(0)
    }
}

mod schema;

use uuid::Uuid;
pub use schema::*;

mod value;
pub use value::*;


pub struct RecordTypeFieldBuilder {
    pub name: String,
    pub aliases: Vec<String>,
    pub field_type: Schema,
}

impl RecordTypeFieldBuilder {
    pub fn add_field_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }
}

#[derive(Default)]
pub struct RecordTypeBuilder {
    aliases: Vec<String>,
    fields: Vec<RecordTypeFieldBuilder>,
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

pub struct EnumTypeSymbolBuilder {
    pub name: String,
    pub aliases: Vec<String>,
    pub value: i32,
}

impl EnumTypeSymbolBuilder {
    pub fn add_symbol_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into());
    }
}

#[derive(Default)]
pub struct EnumTypeBuilder {
    aliases: Vec<String>,
    symbols: Vec<EnumTypeSymbolBuilder>,
}

impl EnumTypeBuilder {
    pub fn add_type_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into())
    }

    pub fn add_symbol(&mut self, name: impl Into<String>, value: i32) -> &mut EnumTypeSymbolBuilder {
        self.symbols.push(EnumTypeSymbolBuilder {
            name: name.into(),
            aliases: Default::default(),
            value,
        });
        self.symbols.last_mut().unwrap()
    }
}


#[derive(Default)]
pub struct FixedTypeBuilder {
    aliases: Vec<String>,
}

impl FixedTypeBuilder {
    pub fn add_type_alias(&mut self, alias: impl Into<String>) {
        self.aliases.push(alias.into())
    }
}




pub struct DatabaseSchemaInfo {
    schema: Schema,
    fingerprint: SchemaFingerprint,
}

pub struct DatabaseObjectInfo {
    schema: Schema, // Will always be a SchemaRecord
    value: Value, // Will always be a ValueRecord
    prototype: Option<ObjectId>,
}

#[derive(Default)]
pub struct Database {
    schemas_by_name: HashMap<String, SchemaFingerprint>,
    schemas: HashMap<SchemaFingerprint, DatabaseSchemaInfo>,
    objects: HashMap<ObjectId, DatabaseObjectInfo>,
}


impl Database {
    pub fn schema(&self, fingerprint: SchemaFingerprint) -> Option<&Schema> {
        self.schemas.get(&fingerprint).map(|x| &x.schema)
    }

    fn register_schema(&mut self, name: impl Into<String>, schema: Schema) {
        let fingerprint = schema.fingerprint();
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
        println!("Registering enum {} {:?}", schema_enum.name(), schema.fingerprint());
        self.register_schema(schema_enum.name(), schema);

        schema_enum
    }

    pub fn register_fixed_type<F: Fn(&mut FixedTypeBuilder)>(&mut self, name: impl Into<String>, length: usize, f: F) -> SchemaFixed {
        let mut builder = FixedTypeBuilder::default();
        (f)(&mut builder);

        let schema_fixed = SchemaFixed::new(name.into(), builder.aliases.into_boxed_slice(), length);

        let schema = Schema::Fixed(schema_fixed.clone());
        println!("Registering fixed {} {}", schema_fixed.name(), schema.fingerprint().as_uuid());
        self.register_schema(schema_fixed.name(), schema);

        schema_fixed
    }

    pub fn find_schema_by_name(&self, name: impl AsRef<str>) -> Option<Schema> {
        self.schemas_by_name.get(name.as_ref()).map(|fingerprint| self.find_schema_by_fingerprint(*fingerprint)).flatten()
    }

    pub fn find_schema_by_fingerprint(&self, fingerprint: SchemaFingerprint) -> Option<Schema> {
        self.schemas.get(&fingerprint).map(|x| x.schema.clone())
    }

    fn insert_object(&mut self, obj_info: DatabaseObjectInfo) -> ObjectId {
        let id = ObjectId(uuid::Uuid::new_v4().as_u128());
        let old = self.objects.insert(id, obj_info);
        assert!(old.is_none());

        id
    }

    pub fn new_object(&mut self, schema: &SchemaRecord) -> ObjectId {
        let obj = DatabaseObjectInfo {
            schema: Schema::Record(schema.clone()),
            value: Value::Record(ValueRecord::default()),
            prototype: None,
        };

        self.insert_object(obj)
    }

    pub fn new_object_from_prototype(&mut self, prototype: ObjectId) -> ObjectId {
        let prototype_info = self.objects.get(&prototype).unwrap();
        let obj = DatabaseObjectInfo {
            schema: prototype_info.schema.clone(),
            value: Value::Record(ValueRecord::default()),
            prototype: Some(prototype)
        };

        self.insert_object(obj)
    }

    pub fn object_schema(&self, object: ObjectId) -> &Schema {
        let o = self.objects.get(&object).unwrap();
        &o.schema
    }

    fn get_or_create_value(&mut self, object: ObjectId) {

    }


    pub fn object_property_resolver(&self, object: ObjectId) -> ObjectPropertyResolver {
        ObjectPropertyResolver {
            object_id: object,
            schema: self.object_schema(object).fingerprint()
        }
    }

    // pub fn find_property_schema<T: AsRef<str>>(&self, schema: &Schema, path: &[T]) -> Option<&Schema> {
    //     let mut s = Some(schema);
    //
    //     //let mut schema_record = schema.as
    //     for p in path {
    //         match s {
    //             Schema::Nullable(x) => s = Some(&*x),
    //             Schema::Record(x) => s = self.schema(x.fingerprint()),
    //             _ => s = None
    //         }
    //
    //         if s.is_none() {
    //             return None;
    //         }
    //     }
    //
    //     s
    // }
}

pub struct ObjectPropertyResolver {
    object_id: ObjectId,
    schema: SchemaFingerprint,
}

impl ObjectPropertyResolver {
    // fn has_property<T: AsRef<str>>(&self, db: &Database, path: &[T]) -> bool {
    //     let schema = db.schema(self.schema).unwrap();
    //     let mut s = schema;
    //
    //     //let mut schema_record = schema.as
    //     for p in path {
    //         match s {
    //             Schema::Nullable(x) => s = &*x,
    //             Schema::Record(x) => s = db.schema(x.fingerprint()).unwrap(),
    //             _ => return false
    //         }
    //     }
    //
    //     match s {
    //         Schema::Nullable(_) => false,
    //         Schema::Record(_) => false,
    //         _ => true,
    //     }
    // }


    fn find_property_path_value<'a, T: AsRef<str>>(&'a self, db: &'a Database, path: &[T]) -> Option<&Schema> {
        let mut o = Some(self.object_id);

        while let Some(o) = o {
            let obj = db.objects.get(&o).unwrap();

            obj.value.find_property_path_value(path);
        }

        None
    }

    fn find_property_path_schema<'a, T: AsRef<str>>(&'a self, db: &'a Database, path: &[T]) -> Option<&Schema> {
        let schema = db.schema(self.schema).unwrap();
        schema.find_property_path_schema(path)







        // let mut s = Some(schema);
        //
        // for p in path {
        //     let mut record = None;
        //     match s {
        //         Schema::Nullable(x) => {
        //             if let Schema::Record(x) = x {
        //                 record = Some(x);
        //             }
        //         },
        //         Schema::Record(x) => {
        //             record = Some(x);
        //         },
        //         _ => {}
        //     }
        //
        //     s = None;
        //     if let Some(record) = record {
        //         for field in record.fields() {
        //             if field.name() == p {
        //                 s = Some(field.field_schema())
        //             }
        //         }
        //     }
        //
        //     if s.is_none() {
        //         return None;
        //     }
        // }
        //
        // s
    }

    // fn get_f32<T: AsRef<str>>(&self, db: &Database, path: &[T]) -> f32 {
    //     //assert!(self.has_property(db, path));
    //
    //     let mut object = db.objects.get(&self.object_id).unwrap();
    //
    //     while object.is_some() {
    //         // check if object has the property
    //
    //         if let Some(object_id) = object.prototype {
    //             object = db.objects.get(&object_id);
    //         }
    //
    //     }
    // }
}
