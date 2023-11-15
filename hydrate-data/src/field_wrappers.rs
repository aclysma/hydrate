use crate::value::ValueEnum;
use crate::{
    DataContainer, DataContainerMut, DataSetError, DataSetResult, NullOverride,
    AssetId, SchemaSet, SingleObject, Value,
};
use std::marker::PhantomData;
use uuid::Uuid;

#[derive(Default)]
pub struct PropertyPath(String);

impl PropertyPath {
    pub fn push(
        &self,
        str: &str,
    ) -> PropertyPath {
        if self.0.is_empty() {
            PropertyPath(str.to_string())
        } else if str.is_empty() {
            PropertyPath(self.0.to_string())
        } else {
            PropertyPath(format!("{}.{}", self.0, str))
        }
    }

    pub fn path(&self) -> &str {
        &self.0
    }
}

pub trait Field {
    fn new(property_path: PropertyPath) -> Self;
}

pub trait Enum: Sized {
    fn to_symbol_name(&self) -> &'static str;
    fn from_symbol_name(str: &str) -> Option<Self>;
}

pub trait Record {
    fn schema_name() -> &'static str;

    fn new_single_object(schema_set: &SchemaSet) -> Option<SingleObject> {
        let schema = schema_set
            .find_named_type(Self::schema_name())
            .unwrap()
            .as_record()?;

        Some(SingleObject::new(schema))
    }
}

pub struct EnumField<T: Enum>(PropertyPath, PhantomData<T>);

impl<T: Enum> Field for EnumField<T> {
    fn new(property_path: PropertyPath) -> Self {
        EnumField(property_path, PhantomData::default())
    }
}

impl<T: Enum> EnumField<T> {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<T> {
        let e = data_container
            .resolve_property(self.0.path())?;
        T::from_symbol_name(e.as_enum().unwrap().symbol_name())
            .ok_or(DataSetError::UnexpectedEnumSymbol)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(
            self.0.path(),
            Some(Value::Enum(ValueEnum::new(value.to_symbol_name().to_string()))),
        )
    }
}

pub struct DynamicArrayField<T: Field>(pub PropertyPath, PhantomData<T>);

impl<T: Field> Field for DynamicArrayField<T> {
    fn new(property_path: PropertyPath) -> Self {
        DynamicArrayField(property_path, PhantomData::default())
    }
}

impl<T: Field> DynamicArrayField<T> {
    pub fn resolve_entries(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<Box<[Uuid]>> {
        data_container.resolve_dynamic_array(self.0.path())
    }

    pub fn entry(
        &self,
        entry_uuid: Uuid,
    ) -> T {
        T::new(self.0.push(&entry_uuid.to_string()))
    }

    pub fn add_entry(
        &self,
        data_container: &mut DataContainerMut,
    ) -> DataSetResult<Uuid> {
        data_container.add_dynamic_array_override(self.0.path())
    }
}

pub struct NullableField<T: Field>(pub PropertyPath, PhantomData<T>);

impl<T: Field> Field for NullableField<T> {
    fn new(property_path: PropertyPath) -> Self {
        NullableField(property_path, PhantomData::default())
    }
}

impl<T: Field> NullableField<T> {
    pub fn resolve_null(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<Option<T>> {
        if self.resolve_null_override(data_container)? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"))))
        } else {
            Ok(None)
        }
    }

    pub fn resolve_null_override(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<NullOverride> {
        data_container.resolve_null_override(self.0.path())
    }

    pub fn set_null_override(
        &self,
        data_container: &mut DataContainerMut,
        null_override: NullOverride,
    ) -> DataSetResult<()> {
        data_container.set_null_override(self.0.path(), null_override)
    }
}

pub struct BytesField(pub PropertyPath);

impl Field for BytesField {
    fn new(property_path: PropertyPath) -> Self {
        BytesField(property_path)
    }
}

impl BytesField {
    pub fn get<'a, 'b>(
        &'a self,
        data_container: &'b DataContainer,
    ) -> DataSetResult<&'b Vec<u8>> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_bytes()
            .unwrap())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: Vec<u8>,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::Bytes(value)))
    }
}

pub struct F32Field(pub PropertyPath);

impl Field for F32Field {
    fn new(property_path: PropertyPath) -> Self {
        F32Field(property_path)
    }
}

impl F32Field {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<f32> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_f32()
            .unwrap())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: f32,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::F32(value)))
    }
}

pub struct F64Field(pub PropertyPath);

impl Field for F64Field {
    fn new(property_path: PropertyPath) -> Self {
        F64Field(property_path)
    }
}

impl F64Field {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<f64> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_f64()
            .unwrap())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: f64,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::F64(value)))
    }
}

pub struct I32Field(pub PropertyPath);

impl Field for I32Field {
    fn new(property_path: PropertyPath) -> Self {
        I32Field(property_path)
    }
}

impl I32Field {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<i32> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_i32()
            .unwrap())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: i32,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::I32(value)))
    }
}

pub struct I64Field(pub PropertyPath);

impl Field for I64Field {
    fn new(property_path: PropertyPath) -> Self {
        I64Field(property_path)
    }
}

impl I64Field {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<i64> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_i64()
            .unwrap())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: i64,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::I64(value)))
    }
}

pub struct U32Field(pub PropertyPath);

impl Field for U32Field {
    fn new(property_path: PropertyPath) -> Self {
        U32Field(property_path)
    }
}

impl U32Field {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<u32> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_u32()
            .unwrap())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: u32,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::U32(value)))
    }
}

pub struct U64Field(pub PropertyPath);

impl Field for U64Field {
    fn new(property_path: PropertyPath) -> Self {
        U64Field(property_path)
    }
}

impl U64Field {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<u64> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_u64()
            .unwrap())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: u64,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::U64(value)))
    }
}

pub struct BooleanField(pub PropertyPath);

impl Field for BooleanField {
    fn new(property_path: PropertyPath) -> Self {
        BooleanField(property_path)
    }
}

impl BooleanField {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<bool> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_boolean()
            .unwrap())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: bool,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::Boolean(value)))
    }
}

pub struct StringField(pub PropertyPath);

impl Field for StringField {
    fn new(property_path: PropertyPath) -> Self {
        StringField(property_path)
    }
}

impl StringField {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<String> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_string()
            .unwrap()
            .to_string())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: String,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::String(value)))
    }
}

pub struct AssetRefField(pub PropertyPath);

impl Field for AssetRefField {
    fn new(property_path: PropertyPath) -> Self {
        AssetRefField(property_path)
    }
}

impl AssetRefField {
    pub fn get(
        &self,
        data_container: &DataContainer,
    ) -> DataSetResult<AssetId> {
        Ok(data_container
            .resolve_property(self.0.path())?
            .as_asset_ref()
            .unwrap())
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: AssetId,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(self.0.path(), Some(Value::AssetRef(value)))
    }
}
