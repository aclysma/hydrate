use crate::{
    HashMap, HashSet, Schema, SchemaDynamicArray, SchemaEnum, SchemaEnumSymbol, SchemaFingerprint,
    SchemaMap, SchemaNamedType, SchemaRecord, SchemaRecordField, SchemaStaticArray,
};
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub enum SchemaDefValidationError {
    DuplicateFieldName(String, String),
    ReferencedNamedTypeNotFound(String, String),
    // Map keys cannot be f32/f64, containers, nullables, records, etc.
    InvalidMapKeyType(String, String),
    // AssetRef can only reference named types that are records
    InvalidAssetRefInnerType(String, String),
}

impl std::fmt::Display for SchemaDefValidationError {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            SchemaDefValidationError::DuplicateFieldName(schema_name, duplicate_field_name) => {
                write!(
                    f,
                    "Schema {} has a duplicate field {}",
                    schema_name, duplicate_field_name
                )
            }
            SchemaDefValidationError::ReferencedNamedTypeNotFound(
                schema_name,
                referenced_named_type_not_found,
            ) => write!(
                f,
                "Schema {} references a type {} that wasn't found",
                schema_name, referenced_named_type_not_found
            ),
            SchemaDefValidationError::InvalidMapKeyType(schema_name, invalid_map_key_type) => {
                write!(
                    f,
                    "Schema {} has map with key of type {}, but this type cannot be used as a key",
                    schema_name, invalid_map_key_type
                )
            }
            SchemaDefValidationError::InvalidAssetRefInnerType(
                schema_name,
                invalid_asset_ref_inner_type,
            ) => write!(
                f,
                "Schema {} references an AssetRef that references {} but it is not a record",
                schema_name, invalid_asset_ref_inner_type
            ),
        }
    }
}

pub type SchemaDefValidationResult<T> = Result<T, SchemaDefValidationError>;

#[derive(Debug)]
pub enum SchemaDefParserError {
    Str(&'static str),
    String(String),
    ValidationError(SchemaDefValidationError),
}

impl From<SchemaDefValidationError> for SchemaDefParserError {
    fn from(validation_error: SchemaDefValidationError) -> Self {
        SchemaDefParserError::ValidationError(validation_error)
    }
}

pub type SchemaDefParserResult<T> = Result<T, SchemaDefParserError>;

#[derive(Debug)]
pub struct SchemaDefStaticArray {
    pub(super) item_type: Box<SchemaDefType>,
    pub(super) length: usize,
}

impl SchemaDefStaticArray {
    fn apply_type_aliases(
        &mut self,
        aliases: &HashMap<String, String>,
    ) {
        self.item_type.apply_type_aliases(aliases);
    }

    fn collect_all_related_types(
        &self,
        types: &mut HashSet<String>,
    ) {
        self.item_type.collect_all_related_types(types);
    }

    fn partial_hash<T: Hasher>(
        &self,
        hasher: &mut T,
    ) {
        self.item_type.partial_hash(hasher);
        self.length.hash(hasher);
    }

    fn to_schema(
        &self,
        named_types: &HashMap<String, SchemaDefNamedType>,
        fingerprints: &HashMap<String, SchemaFingerprint>,
    ) -> SchemaStaticArray {
        SchemaStaticArray::new(
            Box::new(self.item_type.to_schema(named_types, fingerprints)),
            self.length,
        )
    }
}

#[derive(Debug)]
pub struct SchemaDefDynamicArray {
    pub(super) item_type: Box<SchemaDefType>,
}

impl SchemaDefDynamicArray {
    pub fn new(item_type: Box<SchemaDefType>) -> Self {
        SchemaDefDynamicArray { item_type }
    }

    fn apply_type_aliases(
        &mut self,
        aliases: &HashMap<String, String>,
    ) {
        self.item_type.apply_type_aliases(aliases);
    }

    fn collect_all_related_types(
        &self,
        types: &mut HashSet<String>,
    ) {
        self.item_type.collect_all_related_types(types);
    }

    fn partial_hash<T: Hasher>(
        &self,
        hasher: &mut T,
    ) {
        self.item_type.partial_hash(hasher);
    }

    fn to_schema(
        &self,
        named_types: &HashMap<String, SchemaDefNamedType>,
        fingerprints: &HashMap<String, SchemaFingerprint>,
    ) -> SchemaDynamicArray {
        SchemaDynamicArray::new(Box::new(
            self.item_type.to_schema(named_types, fingerprints),
        ))
    }
}

#[derive(Debug)]
pub struct SchemaDefMap {
    pub(super) key_type: Box<SchemaDefType>,
    pub(super) value_type: Box<SchemaDefType>,
}

impl SchemaDefMap {
    fn apply_type_aliases(
        &mut self,
        aliases: &HashMap<String, String>,
    ) {
        self.key_type.apply_type_aliases(aliases);
        self.value_type.apply_type_aliases(aliases);
    }

    fn collect_all_related_types(
        &self,
        types: &mut HashSet<String>,
    ) {
        self.key_type.collect_all_related_types(types);
        self.value_type.collect_all_related_types(types);
    }

    fn partial_hash<T: Hasher>(
        &self,
        hasher: &mut T,
    ) {
        self.key_type.partial_hash(hasher);
        self.value_type.partial_hash(hasher);
    }

    fn to_schema(
        &self,
        named_types: &HashMap<String, SchemaDefNamedType>,
        fingerprints: &HashMap<String, SchemaFingerprint>,
    ) -> SchemaMap {
        SchemaMap::new(
            Box::new(self.key_type.to_schema(named_types, fingerprints)),
            Box::new(self.value_type.to_schema(named_types, fingerprints)),
        )
    }
}

//prior art: https://benui.ca/unreal/uproperty/#general-points
//display name
//category
//tooltip
//min
//max
//clamp min
//clamp max
//units
//array fixed size?
//which field to use to describe the element (for arrays that can't be inlined). Could be like "Field {x} is {y}"
//rgb/xyz/rgba/xyzw
//bitmasks/bitflags?
//deprecation
//allowed classes for references. can we have some concept of interfaces?
//code generation
//if it is an asset that can be created in ui
//if it is import data
//hide in inspector?

#[derive(Default, Debug, Clone)]
pub struct SchemaDefRecordFieldMarkup {
    // If set, we use this name instead of the class name in most UI
    pub display_name: Option<String>,
    // Additional documentation that may show in a tooltip, for example
    pub description: Option<String>,

    // Groups related fields together
    pub category: Option<String>,

    // Clamp min/max cause the UI to force numeric values to be within the given range
    pub clamp_min: Option<f64>,
    pub clamp_max: Option<f64>,

    // ui min/max sets the begin/end point for sliding values but user can still set a value outside
    // this range
    pub ui_min: Option<f64>,
    pub ui_max: Option<f64>,
}

impl SchemaDefRecordFieldMarkup {
    pub fn clamp_min(&self) -> f64 {
        self.clamp_min.unwrap_or(f64::MIN)
    }

    pub fn clamp_max(&self) -> f64 {
        self.clamp_max.unwrap_or(f64::MAX)
    }

    pub fn ui_min(&self) -> f64 {
        // The greater of clamp/ui min
        self.clamp_min
            .unwrap_or(f64::MIN)
            .max(self.ui_min.unwrap_or(f64::MIN))
    }

    pub fn ui_max(&self) -> f64 {
        // The lesser of clamp/ui max
        self.clamp_max
            .unwrap_or(f64::MAX)
            .min(self.ui_max.unwrap_or(f64::MAX))
    }

    pub fn has_min_bound(&self) -> bool {
        self.ui_min.is_some() || self.clamp_min.is_some()
    }

    pub fn has_max_bound(&self) -> bool {
        self.ui_max.is_some() || self.clamp_max.is_some()
    }
}

#[derive(Debug)]
pub struct SchemaDefRecordField {
    pub(super) field_name: String,
    pub(super) aliases: Vec<String>,
    pub(super) field_type: SchemaDefType,
    pub(super) markup: SchemaDefRecordFieldMarkup,
}

impl SchemaDefRecordField {
    pub fn new(
        field_name: String,
        aliases: Vec<String>,
        field_type: SchemaDefType,
        markup: SchemaDefRecordFieldMarkup,
    ) -> SchemaDefValidationResult<Self> {
        Ok(SchemaDefRecordField {
            field_name,
            aliases,
            field_type,
            markup,
        })
    }

    fn apply_type_aliases(
        &mut self,
        aliases: &HashMap<String, String>,
    ) {
        self.field_type.apply_type_aliases(aliases);
    }

    fn collect_all_related_types(
        &self,
        types: &mut HashSet<String>,
    ) {
        self.field_type.collect_all_related_types(types);
    }

    fn partial_hash<T: Hasher>(
        &self,
        hasher: &mut T,
    ) {
        self.field_name.hash(hasher);
        self.field_type.partial_hash(hasher);
    }

    fn to_schema(
        &self,
        named_types: &HashMap<String, SchemaDefNamedType>,
        fingerprints: &HashMap<String, SchemaFingerprint>,
    ) -> SchemaRecordField {
        SchemaRecordField::new(
            self.field_name.clone(),
            self.aliases.clone().into_boxed_slice(),
            self.field_type.to_schema(named_types, fingerprints),
            self.markup.clone(),
        )
    }
}

#[derive(Default, Debug, Clone)]
pub struct SchemaDefRecordMarkup {
    // If set, we use this name instead of the class name in most UI
    pub display_name: Option<String>,

    // Tags can be used to query for a list of records that meet some criteria
    pub tags: HashSet<String>,
    //description: String,
}

//TODO: Verify we don't have dupe field names
#[derive(Debug)]
pub struct SchemaDefRecord {
    pub(super) type_name: String,
    pub(super) aliases: Vec<String>,
    pub(super) fields: Vec<SchemaDefRecordField>,
    pub(super) markup: SchemaDefRecordMarkup,
}

impl SchemaDefRecord {
    pub fn new(
        type_name: String,
        aliases: Vec<String>,
        fields: Vec<SchemaDefRecordField>,
        markup: SchemaDefRecordMarkup,
    ) -> SchemaDefValidationResult<Self> {
        // Check names are unique
        for i in 0..fields.len() {
            for j in 0..i {
                if fields[i].field_name == fields[j].field_name {
                    Err(SchemaDefValidationError::DuplicateFieldName(
                        type_name.clone(),
                        fields[i].field_name.to_string(),
                    ))?;
                }
            }
        }

        Ok(SchemaDefRecord {
            type_name,
            aliases,
            fields,
            markup,
        })
    }

    pub(crate) fn fields(&self) -> &Vec<SchemaDefRecordField> {
        &self.fields
    }

    fn apply_type_aliases(
        &mut self,
        aliases: &HashMap<String, String>,
    ) {
        for field in &mut self.fields {
            field.apply_type_aliases(aliases);
        }
    }

    fn collect_all_related_types(
        &self,
        types: &mut HashSet<String>,
    ) {
        types.insert(self.type_name.clone());
        for field in &self.fields {
            field.collect_all_related_types(types);
        }
    }

    fn partial_hash<T: Hasher>(
        &self,
        hasher: &mut T,
    ) {
        self.type_name.hash(hasher);

        let mut sorted_fields: Vec<_> = self.fields.iter().collect();
        sorted_fields.sort_by_key(|x| &x.field_name);

        for field in sorted_fields {
            //println!("field {}", field.field_name);
            field.partial_hash(hasher);
        }
    }

    fn to_schema(
        &self,
        named_types: &HashMap<String, SchemaDefNamedType>,
        fingerprints: &HashMap<String, SchemaFingerprint>,
    ) -> SchemaRecord {
        let fingerprint = *fingerprints.get(&self.type_name).unwrap();

        let mut fields = Vec::with_capacity(self.fields.len());
        for field in &self.fields {
            fields.push(field.to_schema(named_types, fingerprints));
        }

        SchemaRecord::new(
            self.type_name.clone(),
            fingerprint,
            self.aliases.clone().into_boxed_slice(),
            fields,
            self.markup.clone(),
        )
    }
}

#[derive(Debug)]
pub struct SchemaDefEnumSymbol {
    pub(super) symbol_name: String,
    pub(super) aliases: Vec<String>,
}

impl SchemaDefEnumSymbol {
    pub fn new(
        symbol_name: String,
        aliases: Vec<String>,
    ) -> SchemaDefValidationResult<Self> {
        Ok(SchemaDefEnumSymbol {
            symbol_name,
            aliases,
        })
    }

    fn partial_hash<T: Hasher>(
        &self,
        hasher: &mut T,
    ) {
        self.symbol_name.hash(hasher);
    }

    fn to_schema(&self) -> SchemaEnumSymbol {
        SchemaEnumSymbol::new(
            self.symbol_name.clone(),
            self.aliases.clone().into_boxed_slice(),
        )
    }
}

//TODO: Verify that we don't have dupe symbol names or values
#[derive(Debug)]
pub struct SchemaDefEnum {
    pub(super) type_name: String,
    pub(super) aliases: Vec<String>,
    pub(super) symbols: Vec<SchemaDefEnumSymbol>,
}

impl SchemaDefEnum {
    pub fn new(
        type_name: String,
        aliases: Vec<String>,
        symbols: Vec<SchemaDefEnumSymbol>,
    ) -> SchemaDefValidationResult<Self> {
        Ok(SchemaDefEnum {
            type_name,
            aliases,
            symbols,
        })
    }

    fn apply_type_aliases(
        &mut self,
        _aliases: &HashMap<String, String>,
    ) {
    }

    fn collect_all_related_types(
        &self,
        types: &mut HashSet<String>,
    ) {
        types.insert(self.type_name.clone());
    }

    fn partial_hash<T: Hasher>(
        &self,
        hasher: &mut T,
    ) {
        self.type_name.hash(hasher);

        let mut sorted_symbols: Vec<_> = self.symbols.iter().collect();
        sorted_symbols.sort_by(|a, b| a.symbol_name.cmp(&b.symbol_name));

        for symbol in sorted_symbols {
            symbol.partial_hash(hasher);
        }
    }

    fn to_schema(
        &self,
        named_types: &HashMap<String, SchemaFingerprint>,
    ) -> SchemaEnum {
        let fingerprint = *named_types.get(&self.type_name).unwrap();

        let mut symbols = Vec::with_capacity(self.symbols.len());
        for symbol in &self.symbols {
            symbols.push(symbol.to_schema());
        }

        SchemaEnum::new(
            self.type_name.clone(),
            fingerprint,
            self.aliases.clone().into_boxed_slice(),
            symbols.into_boxed_slice(),
        )
    }
}

#[derive(Debug)]
pub enum SchemaDefType {
    Nullable(Box<SchemaDefType>),
    Boolean,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    Bytes,
    String,
    StaticArray(SchemaDefStaticArray),
    DynamicArray(SchemaDefDynamicArray),
    Map(SchemaDefMap),
    AssetRef(String),  // name of the type
    NamedType(String), // name of the type
}

impl SchemaDefType {
    fn apply_type_aliases(
        &mut self,
        aliases: &HashMap<String, String>,
    ) {
        match self {
            SchemaDefType::Nullable(x) => x.apply_type_aliases(aliases),
            SchemaDefType::Boolean => {}
            SchemaDefType::I32 => {}
            SchemaDefType::I64 => {}
            SchemaDefType::U32 => {}
            SchemaDefType::U64 => {}
            SchemaDefType::F32 => {}
            SchemaDefType::F64 => {}
            SchemaDefType::Bytes => {}
            SchemaDefType::String => {}
            SchemaDefType::StaticArray(x) => x.apply_type_aliases(aliases),
            SchemaDefType::DynamicArray(x) => x.apply_type_aliases(aliases),
            SchemaDefType::Map(x) => x.apply_type_aliases(aliases),
            SchemaDefType::AssetRef(x) => {
                let alias = aliases.get(x);
                if let Some(alias) = alias {
                    *x = alias.clone();
                }
            }
            SchemaDefType::NamedType(x) => {
                let alias = aliases.get(x);
                if let Some(alias) = alias {
                    *x = alias.clone();
                }
            }
        }
    }

    fn collect_all_related_types(
        &self,
        types: &mut HashSet<String>,
    ) {
        match self {
            SchemaDefType::Nullable(x) => x.collect_all_related_types(types),
            SchemaDefType::Boolean => {}
            SchemaDefType::I32 => {}
            SchemaDefType::I64 => {}
            SchemaDefType::U32 => {}
            SchemaDefType::U64 => {}
            SchemaDefType::F32 => {}
            SchemaDefType::F64 => {}
            SchemaDefType::Bytes => {}
            SchemaDefType::String => {}
            SchemaDefType::StaticArray(x) => x.collect_all_related_types(types),
            SchemaDefType::DynamicArray(x) => x.collect_all_related_types(types),
            SchemaDefType::Map(x) => x.collect_all_related_types(types),
            SchemaDefType::AssetRef(x) => {
                types.insert(x.clone());
            }
            SchemaDefType::NamedType(x) => {
                types.insert(x.clone());
            }
        }
    }

    fn partial_hash<T: Hasher>(
        &self,
        hasher: &mut T,
    ) {
        //println!("ty {:?}", self);
        match self {
            SchemaDefType::Nullable(x) => {
                "Nullable".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaDefType::Boolean => "Boolean".hash(hasher),
            SchemaDefType::I32 => "I32".hash(hasher),
            SchemaDefType::I64 => "I64".hash(hasher),
            SchemaDefType::U32 => "U32".hash(hasher),
            SchemaDefType::U64 => "U64".hash(hasher),
            SchemaDefType::F32 => "F32".hash(hasher),
            SchemaDefType::F64 => "F64".hash(hasher),
            SchemaDefType::Bytes => "Bytes".hash(hasher),
            SchemaDefType::String => "String".hash(hasher),
            SchemaDefType::StaticArray(x) => {
                "StaticArray".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaDefType::DynamicArray(x) => {
                "DynamicArray".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaDefType::Map(x) => {
                "Map".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaDefType::AssetRef(x) => {
                "AssetRef".hash(hasher);
                x.hash(hasher);
            }
            SchemaDefType::NamedType(x) => {
                "NamedType".hash(hasher);
                x.hash(hasher);
            }
        }
    }

    fn to_schema(
        &self,
        named_types: &HashMap<String, SchemaDefNamedType>,
        fingerprints: &HashMap<String, SchemaFingerprint>,
    ) -> Schema {
        match self {
            SchemaDefType::Nullable(x) => {
                Schema::Nullable(Box::new(x.to_schema(named_types, fingerprints)))
            }
            SchemaDefType::Boolean => Schema::Boolean,
            SchemaDefType::I32 => Schema::I32,
            SchemaDefType::I64 => Schema::I64,
            SchemaDefType::U32 => Schema::U32,
            SchemaDefType::U64 => Schema::U64,
            SchemaDefType::F32 => Schema::F32,
            SchemaDefType::F64 => Schema::F64,
            SchemaDefType::Bytes => Schema::Bytes,
            SchemaDefType::String => Schema::String,
            SchemaDefType::StaticArray(x) => {
                Schema::StaticArray(x.to_schema(named_types, fingerprints))
            }
            SchemaDefType::DynamicArray(x) => {
                Schema::DynamicArray(x.to_schema(named_types, fingerprints))
            }
            SchemaDefType::Map(x) => Schema::Map(x.to_schema(named_types, fingerprints)),
            SchemaDefType::AssetRef(x) => Schema::AssetRef(*fingerprints.get(x).unwrap()),
            SchemaDefType::NamedType(x) => {
                let named_type = named_types.get(x).unwrap();
                match named_type {
                    SchemaDefNamedType::Record(_) => Schema::Record(*fingerprints.get(x).unwrap()),
                    SchemaDefNamedType::Enum(_) => Schema::Enum(*fingerprints.get(x).unwrap()),
                }
            }
        }
    }
}

pub enum SchemaDefNamedType {
    Record(SchemaDefRecord),
    Enum(SchemaDefEnum),
}

impl SchemaDefNamedType {
    pub(super) fn type_name(&self) -> &str {
        match self {
            SchemaDefNamedType::Record(x) => &x.type_name,
            SchemaDefNamedType::Enum(x) => &x.type_name,
        }
    }

    pub(super) fn aliases(&self) -> &[String] {
        match self {
            SchemaDefNamedType::Record(x) => &x.aliases,
            SchemaDefNamedType::Enum(x) => &x.aliases,
        }
    }

    pub(super) fn apply_type_aliases(
        &mut self,
        aliases: &HashMap<String, String>,
    ) {
        match self {
            SchemaDefNamedType::Record(x) => x.apply_type_aliases(aliases),
            SchemaDefNamedType::Enum(x) => x.apply_type_aliases(aliases),
        }
    }

    pub(super) fn collect_all_related_types(
        &self,
        types: &mut HashSet<String>,
    ) {
        match self {
            SchemaDefNamedType::Record(x) => x.collect_all_related_types(types),
            SchemaDefNamedType::Enum(x) => x.collect_all_related_types(types),
        }
    }

    pub(super) fn partial_hash<T: Hasher>(
        &self,
        hasher: &mut T,
    ) {
        match self {
            SchemaDefNamedType::Record(x) => {
                //println!("record");
                "record".hash(hasher);
                x.partial_hash(hasher);
            }
            SchemaDefNamedType::Enum(x) => {
                "enum".hash(hasher);
                x.partial_hash(hasher);
            }
        }
    }

    pub(super) fn to_schema(
        &self,
        named_types: &HashMap<String, SchemaDefNamedType>,
        fingerprints: &HashMap<String, SchemaFingerprint>,
    ) -> SchemaNamedType {
        match self {
            SchemaDefNamedType::Record(x) => {
                SchemaNamedType::Record(x.to_schema(named_types, fingerprints))
            }
            SchemaDefNamedType::Enum(x) => SchemaNamedType::Enum(x.to_schema(fingerprints)),
        }
    }
}
