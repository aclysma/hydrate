use crate::{
    HashMap, Schema, SchemaDynamicArray, SchemaEnum, SchemaEnumSymbol, SchemaFingerprint,
    SchemaFixed, SchemaMap, SchemaNamedType, SchemaRecord, SchemaRecordField, SchemaStaticArray,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct CachedSchemaStaticArray {
    item_type: Box<CachedSchema>,
    length: usize,
}

impl CachedSchemaStaticArray {
    fn new_from_schema(schema: &SchemaStaticArray) -> Self {
        CachedSchemaStaticArray {
            item_type: Box::new(CachedSchema::new_from_schema(schema.item_type())),
            length: schema.length,
        }
    }

    fn to_schema(self) -> SchemaStaticArray {
        SchemaStaticArray::new(Box::new(self.item_type.to_schema()), self.length)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedSchemaDynamicArray {
    item_type: Box<CachedSchema>,
}

impl CachedSchemaDynamicArray {
    fn new_from_schema(schema: &SchemaDynamicArray) -> Self {
        CachedSchemaDynamicArray {
            item_type: Box::new(CachedSchema::new_from_schema(schema.item_type())),
        }
    }

    fn to_schema(self) -> SchemaDynamicArray {
        SchemaDynamicArray::new(Box::new(self.item_type.to_schema()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedSchemaMap {
    key_type: Box<CachedSchema>,
    value_type: Box<CachedSchema>,
}

impl CachedSchemaMap {
    fn new_from_schema(schema: &SchemaMap) -> Self {
        CachedSchemaMap {
            key_type: Box::new(CachedSchema::new_from_schema(schema.key_type())),
            value_type: Box::new(CachedSchema::new_from_schema(schema.value_type())),
        }
    }

    fn to_schema(self) -> SchemaMap {
        SchemaMap::new(
            Box::new(self.key_type.to_schema()),
            Box::new(self.value_type.to_schema()),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedSchemaRecordField {
    name: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    aliases: Vec<String>,
    field_schema: Box<CachedSchema>,
}

impl CachedSchemaRecordField {
    fn new_from_schema(schema: &SchemaRecordField) -> Self {
        CachedSchemaRecordField {
            name: schema.name().to_string(),
            aliases: schema.aliases().iter().cloned().collect(),
            field_schema: Box::new(CachedSchema::new_from_schema(schema.field_schema())),
        }
    }

    fn to_schema(self) -> SchemaRecordField {
        SchemaRecordField::new(
            self.name,
            self.aliases.into_boxed_slice(),
            self.field_schema.to_schema(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedSchemaRecord {
    name: String,
    fingerprint: Uuid,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    aliases: Vec<String>,
    fields: Vec<CachedSchemaRecordField>,
}

impl CachedSchemaRecord {
    fn new_from_schema(schema: &SchemaRecord) -> Self {
        let mut fields = Vec::with_capacity(schema.fields().len());
        for field in schema.fields() {
            fields.push(CachedSchemaRecordField::new_from_schema(field));
        }

        CachedSchemaRecord {
            name: schema.name().to_string(),
            fingerprint: schema.fingerprint().as_uuid(),
            aliases: schema.aliases().iter().cloned().collect(),
            fields,
        }
    }

    fn to_schema(self) -> SchemaRecord {
        let mut fields = Vec::with_capacity(self.fields.len());
        for field in self.fields {
            fields.push(field.to_schema());
        }

        SchemaRecord::new(
            self.name,
            SchemaFingerprint(self.fingerprint.as_u128()),
            self.aliases.into_boxed_slice(),
            fields,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedSchemaEnumSymbol {
    name: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    aliases: Vec<String>,
    //value: i32,
}

impl CachedSchemaEnumSymbol {
    fn new_from_schema(schema: &SchemaEnumSymbol) -> Self {
        CachedSchemaEnumSymbol {
            name: schema.name().to_string(),
            aliases: schema.aliases().iter().cloned().collect(),
            //value: schema.value(),
        }
    }

    fn to_schema(self) -> SchemaEnumSymbol {
        SchemaEnumSymbol::new(
            self.name,
            self.aliases.into_boxed_slice(), /*, self.value*/
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedSchemaEnum {
    name: String,
    fingerprint: Uuid,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    aliases: Vec<String>,
    symbols: Vec<CachedSchemaEnumSymbol>,
}

impl CachedSchemaEnum {
    fn new_from_schema(schema: &SchemaEnum) -> Self {
        let mut symbols = Vec::with_capacity(schema.symbols().len());
        for field in schema.symbols() {
            symbols.push(CachedSchemaEnumSymbol::new_from_schema(field));
        }

        CachedSchemaEnum {
            name: schema.name().to_string(),
            fingerprint: schema.fingerprint().as_uuid(),
            aliases: schema.aliases().iter().cloned().collect(),
            symbols,
        }
    }

    fn to_schema(self) -> SchemaEnum {
        let mut symbols = Vec::with_capacity(self.symbols.len());
        for symbol in self.symbols {
            symbols.push(symbol.to_schema());
        }

        SchemaEnum::new(
            self.name,
            SchemaFingerprint(self.fingerprint.as_u128()),
            self.aliases.into_boxed_slice(),
            symbols.into_boxed_slice(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedSchemaFixed {
    name: String,
    fingerprint: Uuid,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    aliases: Vec<String>,
    length: usize,
}

impl CachedSchemaFixed {
    fn new_from_schema(schema: &SchemaFixed) -> Self {
        CachedSchemaFixed {
            name: schema.name().to_string(),
            fingerprint: schema.fingerprint().as_uuid(),
            aliases: schema.aliases().iter().cloned().collect(),
            length: schema.length(),
        }
    }

    fn to_schema(self) -> SchemaFixed {
        SchemaFixed::new(
            self.name,
            SchemaFingerprint(self.fingerprint.as_u128()),
            self.aliases.into_boxed_slice(),
            self.length,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum CachedSchemaNamedType {
    Record(CachedSchemaRecord),
    Enum(CachedSchemaEnum),
    Fixed(CachedSchemaFixed),
}

impl CachedSchemaNamedType {
    fn fingerprint(&self) -> Uuid {
        match self {
            CachedSchemaNamedType::Record(x) => x.fingerprint,
            CachedSchemaNamedType::Enum(x) => x.fingerprint,
            CachedSchemaNamedType::Fixed(x) => x.fingerprint,
        }
    }

    fn new_from_schema(schema: &SchemaNamedType) -> CachedSchemaNamedType {
        match schema {
            SchemaNamedType::Record(x) => {
                CachedSchemaNamedType::Record(CachedSchemaRecord::new_from_schema(x))
            }
            SchemaNamedType::Enum(x) => {
                CachedSchemaNamedType::Enum(CachedSchemaEnum::new_from_schema(x))
            }
            SchemaNamedType::Fixed(x) => {
                CachedSchemaNamedType::Fixed(CachedSchemaFixed::new_from_schema(x))
            }
        }
    }

    fn to_schema(self) -> SchemaNamedType {
        match self {
            CachedSchemaNamedType::Record(x) => SchemaNamedType::Record(x.to_schema()),
            CachedSchemaNamedType::Enum(x) => SchemaNamedType::Enum(x.to_schema()),
            CachedSchemaNamedType::Fixed(x) => SchemaNamedType::Fixed(x.to_schema()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum CachedSchema {
    /// Marks the field as possible to be null
    Nullable(Box<CachedSchema>),
    Boolean,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    /// Variable amount of bytes stored within the object, intended to be relatively small
    Bytes,
    /// A variable amount of bytes stored on a reference-counted heap and shared. Can be large (MBs)
    Buffer,
    /// Variable-length UTF-8 String
    String,
    /// Fixed-size array of values
    StaticArray(CachedSchemaStaticArray),
    DynamicArray(CachedSchemaDynamicArray),
    Map(CachedSchemaMap),
    //RecordRef(CachedSchemaRefConstraint),
    ObjectRef(Uuid),
    /// Named type, it could be an enum, record, etc.
    NamedType(Uuid),
}

impl CachedSchema {
    fn new_from_schema(schema: &Schema) -> CachedSchema {
        match schema {
            Schema::Nullable(inner_schema) => {
                CachedSchema::Nullable(Box::new(CachedSchema::new_from_schema(&*inner_schema)))
            }
            Schema::Boolean => CachedSchema::Boolean,
            Schema::I32 => CachedSchema::I32,
            Schema::I64 => CachedSchema::I64,
            Schema::U32 => CachedSchema::U32,
            Schema::U64 => CachedSchema::U64,
            Schema::F32 => CachedSchema::F32,
            Schema::F64 => CachedSchema::F64,
            Schema::Bytes => CachedSchema::Bytes,
            Schema::Buffer => CachedSchema::Buffer,
            Schema::String => CachedSchema::String,
            Schema::StaticArray(x) => {
                CachedSchema::StaticArray(CachedSchemaStaticArray::new_from_schema(x))
            }
            Schema::DynamicArray(x) => {
                CachedSchema::DynamicArray(CachedSchemaDynamicArray::new_from_schema(x))
            }
            Schema::Map(x) => CachedSchema::Map(CachedSchemaMap::new_from_schema(x)),
            //Schema::RecordRef(x) => CachedSchemaStaticArray::new_from_schema(x),
            Schema::AssetRef(x) => CachedSchema::ObjectRef(x.as_uuid()),
            Schema::NamedType(x) => CachedSchema::NamedType(x.as_uuid()),
        }
    }

    fn to_schema(self) -> Schema {
        match self {
            CachedSchema::Nullable(x) => Schema::Nullable(Box::new(x.to_schema())),
            CachedSchema::Boolean => Schema::Boolean,
            CachedSchema::I32 => Schema::I32,
            CachedSchema::I64 => Schema::I64,
            CachedSchema::U32 => Schema::U32,
            CachedSchema::U64 => Schema::U64,
            CachedSchema::F32 => Schema::F32,
            CachedSchema::F64 => Schema::F64,
            CachedSchema::Bytes => Schema::Bytes,
            CachedSchema::Buffer => Schema::Buffer,
            CachedSchema::String => Schema::String,
            CachedSchema::StaticArray(x) => Schema::StaticArray(x.to_schema()),
            CachedSchema::DynamicArray(x) => Schema::DynamicArray(x.to_schema()),
            CachedSchema::Map(x) => Schema::Map(x.to_schema()),
            CachedSchema::ObjectRef(x) => Schema::AssetRef(SchemaFingerprint(x.as_u128())),
            CachedSchema::NamedType(x) => Schema::NamedType(SchemaFingerprint(x.as_u128())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaCacheSingleFile {
    cached_schemas: Vec<CachedSchemaNamedType>,
}

impl SchemaCacheSingleFile {
    pub fn store_string(schemas: &HashMap<SchemaFingerprint, SchemaNamedType>) -> String {
        let mut cached_schemas: Vec<CachedSchemaNamedType> = Default::default();

        for (_, schema) in schemas {
            cached_schemas.push(CachedSchemaNamedType::new_from_schema(schema));
        }

        cached_schemas.sort_by_key(|x| x.fingerprint());

        let cache = SchemaCacheSingleFile { cached_schemas };

        serde_json::to_string_pretty(&cache).unwrap()
    }

    pub fn load_string(cache: &str) -> Vec<SchemaNamedType> {
        let cache: SchemaCacheSingleFile = serde_json::from_str(cache).unwrap();
        cache
            .cached_schemas
            .into_iter()
            .map(|x| x.to_schema())
            .collect()
    }
}
