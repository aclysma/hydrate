use super::{HashMap, ObjectId, SchemaFingerprint};
use super::schema::*;
use super::value::*;

mod record_type_builder;
use record_type_builder::RecordTypeBuilder;

mod fixed_type_builder;
use fixed_type_builder::FixedTypeBuilder;

mod enum_type_builder;
use enum_type_builder::EnumTypeBuilder;
use crate::BufferId;

/*
pub struct SchemaPropertyMappingNode {
    name: String,
    //index: u64,
    //parent: u32,
    // first_child: u32,
    // children_count: u32,
    children: Box<[SchemaPropertyMappingNode]>,


    //stride: u64,
    schema: Schema
}

pub struct SchemaPropertyMapping {
    //property_schemas: Box<[Schema]>
    root: SchemaPropertyMappingNode
}

impl SchemaPropertyMapping {
    fn add_properties(mappings: &mut Vec<SchemaPropertyMappingNode>, schema: &Schema, prefix: String) {
        match schema {
            Schema::Nullable(x) => {
                let mut children = Vec::with_capacity(2);
                Self::add_properties(&mut children, &Schema::Boolean, format!("{}.is_null", prefix));
                Self::add_properties(&mut children, &*x, format!("{}.value.", prefix));
                mappings.push(SchemaPropertyMappingNode {
                    name: prefix,
                    children: children.into_boxed_slice(),
                    schema: schema.clone(),
                })
            }
            Schema::Boolean | Schema::I32 | Schema::I64 | Schema::U32 | Schema::U64 | Schema::F32 | Schema::F64 | Schema::Bytes | Schema::Buffer | Schema::String => {
                mappings.push(SchemaPropertyMappingNode {
                    name: prefix,
                    children: Vec::default().into_boxed_slice(),
                    schema: schema.clone(),
                });
            }
            // Schema::I32 => {}
            // Schema::I64 => {}
            // Schema::U32 => {}
            // Schema::U64 => {}
            // Schema::F32 => {}
            // Schema::F64 => {}
            // Schema::Bytes => {}
            // Schema::Buffer => {}
            // Schema::String => {}
            Schema::StaticArray(x) => {
                let mut children = Vec::with_capacity(x.length);
                for i in 0..x.length {
                    Self::add_properties(mappings, &*x.item_type, format!("{}.{}", prefix, i));
                }

                mappings.push(SchemaPropertyMappingNode {
                    name: prefix,
                    children: children.into_boxed_slice(),
                    schema: schema.clone(),
                });
            }
            Schema::DynamicArray(x) => {
                let mut children = Vec::with_capacity(2);
                Self::add_properties(&mut children, &Schema::Boolean, format!("{}.include_prototype_values", prefix));
                //Self::add_properties(&mut children, schema.clone(), format!("{}.data", prefix));
                children.push(SchemaPropertyMappingNode {
                    name: format!("{}.data", prefix),
                    children: Vec::default().into_boxed_slice(),
                    schema: Schema::DynamicArray(x.clone()),
                });

                mappings.push(SchemaPropertyMappingNode {
                    name: prefix,
                    children: children.into_boxed_slice(),
                    schema: schema.clone(),
                });

            }
            Schema::Map(_) => {}
            Schema::RecordRef(_) => {
                mappings.push(SchemaPropertyMappingNode {
                    name: prefix,
                    children: Vec::default().into_boxed_slice(),
                    schema: schema.clone(),
                });
            }
            Schema::Record(_) => {}
            //Schema::Enum(_) => {}
            //Schema::Fixed(_) => {}
        }
    }

    pub fn new(schema: Schema) {


    }
}
*/



pub struct DatabaseSchemaInfo {
    schema: Schema,
    fingerprint: SchemaFingerprint,
}

pub struct DatabaseObjectInfo {
    schema: Schema, // Will always be a SchemaRecord
    //value: Value, // Will always be a ValueRecord
    prototype: Option<ObjectId>,
    properties: HashMap<String, Value>
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
            prototype: None,
            properties: Default::default()
        };

        self.insert_object(obj)
    }

    pub fn new_object_from_prototype(&mut self, prototype: ObjectId) -> ObjectId {
        let prototype_info = self.objects.get(&prototype).unwrap();
        let obj = DatabaseObjectInfo {
            schema: prototype_info.schema.clone(),
            prototype: Some(prototype),
            properties: Default::default(),
        };

        self.insert_object(obj)
    }

    pub fn object_schema(&self, object: ObjectId) -> &Schema {
        let o = self.objects.get(&object).unwrap();
        &o.schema
    }


    pub fn get_value<T: AsRef<str>>(&self, object: ObjectId,  path: T) -> Option<&Value> {
        let mut object_id = Some(object);

        let mut schema = self.object_schema(object);
        for path_segment in path.as_ref().split(".") {
            let s = schema.find_property_schema(path_segment);
            if let Some(s) = s {
                schema = s;
            }
        }

        // match schema {
        //     Schema::Nullable(_) => {}
        //     Schema::Boolean => {}
        //     Schema::I32 => {}
        //     Schema::I64 => {}
        //     Schema::U32 => {}
        //     Schema::U64 => {}
        //     Schema::F32 => {}
        //     Schema::F64 => {}
        //     Schema::Bytes => {}
        //     Schema::Buffer => {}
        //     Schema::String => {}
        //     Schema::StaticArray(_) => {}
        //     Schema::DynamicArray(_) => {}
        //     Schema::Map(_) => {}
        //     Schema::RecordRef(_) => {}
        //     Schema::Record(_) => {}
        //     Schema::Enum(_) => {}
        //     Schema::Fixed(_) => {}
        // }


        while let Some(obj_id) = object_id {
            let obj = self.objects.get(&obj_id).unwrap();
            if let Some(value) = obj.properties.get(path.as_ref()) {
                return Some(value);
            }

            object_id = obj.prototype;
        }

        None


        //let schema = self.object_schema(object).clone();
    }

    // fn get_or_create_value(&mut self, object: ObjectId) {
    //
    // }


    /*
    pub fn object_property_resolver(&self, object: ObjectId) -> ObjectPropertyResolver {
        ObjectPropertyResolver {
            object_id: object,
            schema: self.object_schema(object).clone()
        }
    }
    */

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

/*
const EMPTY_BYTE_SLICE: &'static [u8] = &[];


#[derive(Debug)]
pub enum PropertyResolverError {
    FieldDoesNotExist,
    IncorrectType
}

pub type PropertyResolverResult<T> = Result<T, PropertyResolverError>;

pub struct ObjectPropertyResolver {
    object_id: ObjectId,
    schema: Schema,
}

impl ObjectPropertyResolver {
    fn find_property_path_value<'a, T: AsRef<str>>(&'a self, db: &'a Database, path: &[T]) -> Option<&Value> {
        let mut object_id = Some(self.object_id);

        while let Some(o) = object_id {
            let obj = db.objects.get(&o).unwrap();

            let value = obj.value.find_property_path_value(path);
            if value.is_some() {
                return value;
            }

            object_id = obj.prototype;
        }

        None
    }

    fn get_path<PathT: AsRef<str>, ValueT, ExtractFn: Fn(&Value) -> PropertyResolverResult<ValueT>>(&self, db: &mut Database, path: &[PathT], default: ValueT, extract_fn: ExtractFn) -> PropertyResolverResult<ValueT> {
        let value = self.find_property_path_value(db, path);
        if let Some(value) = value {
            (extract_fn)(value)
        } else {
            Ok(default)
        }
    }

    pub fn set_path<PathT: AsRef<str>, CompatibleFn: Fn(&Schema) -> bool>(&self, db: &mut Database, path: &[PathT], value: Value, compatible_fn: CompatibleFn) -> PropertyResolverResult<()> {
        let schema = self.schema.find_property_path_schema(path);
        if schema.is_none() {
            Err(PropertyResolverError::FieldDoesNotExist)?;
        }

        let schema = schema.unwrap();
        if !((compatible_fn)(schema)) {
            Err(PropertyResolverError::IncorrectType)?;
        }

        db.objects.get_mut(&self.object_id).unwrap().value.set_property_path_value(path, value);

        Ok(())
    }


    pub fn get_path_boolean<T: AsRef<str>>(&self, db: &mut Database, path: &[T]) -> PropertyResolverResult<bool> {
        self.get_path(db, path, false, |value| {
            if let Value::Boolean(value) = value {
                Ok(*value)
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        })
    }

    pub fn set_path_boolean<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: bool) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::Boolean(value), |schema| schema.is_boolean())
    }

    pub fn get_path_i32<T: AsRef<str>>(&self, db: &mut Database, path: &[T]) -> PropertyResolverResult<i32> {
        self.get_path(db, path, 0, |value| {
            if let Value::I32(value) = value {
                Ok(*value)
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        })
    }

    pub fn set_path_i32<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: i32) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::I32(value), |schema| schema.is_i32())
    }

    pub fn get_path_u32<T: AsRef<str>>(&self, db: &mut Database, path: &[T]) -> PropertyResolverResult<u32> {
        self.get_path(db, path, 0, |value| {
            if let Value::U32(value) = value {
                Ok(*value)
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        })
    }

    pub fn set_path_u32<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: u32) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::U32(value), |schema| schema.is_u32())
    }

    pub fn get_path_i64<T: AsRef<str>>(&self, db: &mut Database, path: &[T]) -> PropertyResolverResult<i64> {
        self.get_path(db, path, 0, |value| {
            if let Value::I64(value) = value {
                Ok(*value)
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        })
    }

    pub fn set_path_i64<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: i64) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::I64(value), |schema| schema.is_i64())
    }

    pub fn get_path_u64<T: AsRef<str>>(&self, db: &mut Database, path: &[T]) -> PropertyResolverResult<u64> {
        self.get_path(db, path, 0, |value| {
            if let Value::U64(value) = value {
                Ok(*value)
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        })
    }

    pub fn set_path_u64<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: u64) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::U64(value), |schema| schema.is_u64())
    }

    pub fn get_path_f32<T: AsRef<str>>(&self, db: &mut Database, path: &[T]) -> PropertyResolverResult<f32> {
        self.get_path(db, path, 0.0, |value| {
            if let Value::F32(value) = value {
                Ok(*value)
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        })
    }

    pub fn set_path_f32<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: f32) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::F32(value), |schema| schema.is_f32())
    }

    pub fn get_path_f64<T: AsRef<str>>(&self, db: &mut Database, path: &[T]) -> PropertyResolverResult<f64> {
        self.get_path(db, path, 0.0, |value| {
            if let Value::F64(value) = value {
                Ok(*value)
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        })
    }

    pub fn set_path_f64<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: f64) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::F64(value), |schema| schema.is_f64())
    }

    pub fn get_path_bytes<'a, T: AsRef<str>>(&'a self, db: &'a mut Database, path: &[T]) -> PropertyResolverResult<&'a [u8]> {
        let value = self.find_property_path_value(db, path);
        if let Some(value) = value {
            if let Value::Bytes(value) = value {
                Ok(value)
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        } else {
            Ok(EMPTY_BYTE_SLICE)
        }
    }

    pub fn set_path_bytes<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: Vec<u8>) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::Bytes(value), |schema| schema.is_bytes())
    }

    pub fn get_path_buffer<T: AsRef<str>>(&self, db: &mut Database, path: &[T]) -> PropertyResolverResult<BufferId> {
        self.get_path(db, path, BufferId::null(), |value| {
            if let Value::Buffer(value) = value {
                Ok(*value)
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        })
    }

    pub fn set_path_buffer<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: BufferId) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::Buffer(value), |schema| schema.is_buffer())
    }

    pub fn get_path_string<'a, T: AsRef<str>>(&'a self, db: &'a mut Database, path: &[T]) -> PropertyResolverResult<&'a str> {
        let value = self.find_property_path_value(db, path);
        if let Some(value) = value {
            if let Value::String(value) = value {
                Ok(value.as_str())
            } else {
                Err(PropertyResolverError::IncorrectType)
            }
        } else {
            Ok("")
        }
    }

    pub fn set_path_string<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: String) -> PropertyResolverResult<()> {
        self.set_path(db, path, Value::String(value), |schema| schema.is_string())
    }

    // static array
    // dynamic array
    // map
    // record_ref
    // enum

    pub fn set_path_fixed<T: AsRef<str>>(&self, db: &mut Database, path: &[T], value: Box<[u8]>) -> PropertyResolverResult<()> {
        let expected_length = value.len();
        self.set_path(db, path, Value::Fixed(value), |schema| {
            if let Schema::Fixed(schema_fixed) = schema {
                if schema_fixed.length() == expected_length {
                    return true;
                }
            }

            false
        })
    }




/*
    fn find_property_path_value<'a, T: AsRef<str>>(&'a self, db: &'a Database, path: &[T]) -> Option<&Schema> {
        let o = Some(self.object_id);

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
    */

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
*/