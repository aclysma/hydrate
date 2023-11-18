use crate::value::ValueEnum;
use crate::{
    AssetId, DataContainer, DataContainerMut, DataSetError, DataSetResult, NullOverride, SchemaSet,
    SingleObject, Value,
};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use uuid::Uuid;
use crate::data_set_view::DataContainerOwned;

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

pub trait FieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self;
}

pub trait FieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self;
}

pub trait FieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self;
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

pub trait RecordReader {
    fn schema_name() -> &'static str;
}

pub trait RecordWriter {
    fn schema_name() -> &'static str;
}

pub trait RecordOwned {
    fn schema_name() -> &'static str;

    fn new_single_object(schema_set: &SchemaSet) -> Option<SingleObject> {
        let schema = schema_set
            .find_named_type(Self::schema_name())
            .unwrap()
            .as_record()?;

        Some(SingleObject::new(schema))
    }
}


pub struct RecordBuilder<T: RecordOwned + FieldOwned>(Rc<RefCell<Option<DataContainerOwned>>>, T, PhantomData<T>);

impl<T: RecordOwned + FieldOwned> RecordBuilder<T> {
    pub fn new(schema_set: &SchemaSet) -> Self {
        let single_object =
            T::new_single_object(schema_set).unwrap();
        let data_container =
            DataContainerOwned::from_single_object(single_object, schema_set.clone());
        let data_container = Rc::new(RefCell::new(Some(data_container)));
        let owned = T::new(Default::default(), &data_container);
        Self(data_container, owned, Default::default())
    }

    pub fn into_inner(self) -> DataSetResult<SingleObject> {
        // We are unwrapping an Rc, the RefCell, Option, and the DataContainer
        Ok(self.0.borrow_mut().take().ok_or(DataSetError::DataTaken)?.into_inner())
    }
}

impl<T: RecordOwned + FieldOwned> Deref for RecordBuilder<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T: RecordOwned + FieldOwned> DerefMut for RecordBuilder<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

pub struct EnumField<T: Enum>(PropertyPath, PhantomData<T>);

impl<T: Enum> Field for EnumField<T> {
    fn new(property_path: PropertyPath) -> Self {
        EnumField(property_path, PhantomData::default())
    }
}

impl<T: Enum> EnumField<T> {
    pub fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<T> {
        let e = data_container.resolve_property(property_path.path())?;
        T::from_symbol_name(e.as_enum().unwrap().symbol_name())
            .ok_or(DataSetError::UnexpectedEnumSymbol)
    }

    pub fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(
            property_path.path(),
            Some(Value::Enum(ValueEnum::new(
                value.to_symbol_name().to_string(),
            ))),
        )
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<T> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct EnumFieldReader<'a, T>(pub PropertyPath, DataContainer<'a>, PhantomData<T>);

impl<'a, T: Enum> FieldReader<'a> for EnumFieldReader<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        EnumFieldReader(property_path, data_container, PhantomData)
    }
}

impl<'a, T: Enum> EnumFieldReader<'a, T> {
    pub fn get(&self) -> DataSetResult<T> {
        EnumField::<T>::do_get(&self.0, self.1)
    }
}

pub struct EnumFieldWriter<'a, T: Enum>(
    pub PropertyPath,
    Rc<RefCell<DataContainerMut<'a>>>,
    PhantomData<T>,
);

impl<'a, T: Enum> FieldWriter<'a> for EnumFieldWriter<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        EnumFieldWriter(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: Enum> EnumFieldWriter<'a, T> {
    pub fn get(&self) -> DataSetResult<T> {
        EnumField::<T>::do_get(&self.0, self.1.borrow().read())
    }

    pub fn set(
        &self,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        EnumField::<T>::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct EnumFieldOwned<T: Enum>(
    pub PropertyPath,
    Rc<RefCell<Option<DataContainerOwned>>>,
    PhantomData<T>,
);

impl<T: Enum> FieldOwned for EnumFieldOwned<T> {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        EnumFieldOwned(property_path, data_container.clone(), PhantomData)
    }
}

impl<T: Enum> EnumFieldOwned<T> {
    pub fn get(&self) -> DataSetResult<T> {
        EnumField::<T>::do_get(&self.0, self.1.borrow().as_ref().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        EnumField::<T>::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
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
        data_container: DataContainer,
    ) -> DataSetResult<Option<T>> {
        if self.resolve_null_override(data_container)? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"))))
        } else {
            Ok(None)
        }
    }

    pub fn resolve_null_override(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<NullOverride> {
        data_container.resolve_null_override(self.0.path())
    }

    pub fn set_null_override(
        &self,
        data_container: &mut DataContainerMut,
        null_override: NullOverride,
    ) -> DataSetResult<Option<T>> {
        let path = self.0.path();
        data_container.set_null_override(path, null_override)?;
        if data_container.resolve_null_override(path)? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"))))
        } else {
            Ok(None)
        }
    }
}

pub struct NullableFieldReader<'a, T>(pub PropertyPath, DataContainer<'a>, PhantomData<T>);

impl<'a, T: FieldReader<'a>> FieldReader<'a> for NullableFieldReader<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        NullableFieldReader(property_path, data_container, PhantomData)
    }
}

impl<'a, T: FieldReader<'a>> NullableFieldReader<'a, T> {
    pub fn resolve_null(&self) -> DataSetResult<Option<T>> {
        if self.resolve_null_override()? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"), self.1)))
        } else {
            Ok(None)
        }
    }

    pub fn resolve_null_override(&self) -> DataSetResult<NullOverride> {
        self.1.resolve_null_override(self.0.path())
    }
}

pub struct NullableFieldWriter<'a, T: FieldWriter<'a>>(
    pub PropertyPath,
    Rc<RefCell<DataContainerMut<'a>>>,
    PhantomData<T>,
);

impl<'a, T: FieldWriter<'a>> FieldWriter<'a> for NullableFieldWriter<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        NullableFieldWriter(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: FieldWriter<'a>> NullableFieldWriter<'a, T> {
    pub fn resolve_null(&'a self) -> DataSetResult<Option<T>> {
        if self.resolve_null_override()? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"), &self.1)))
        } else {
            Ok(None)
        }
    }

    pub fn resolve_null_override(&self) -> DataSetResult<NullOverride> {
        self.1.borrow_mut().resolve_null_override(self.0.path())
    }

    pub fn set_null_override(
        &'a self,
        null_override: NullOverride,
    ) -> DataSetResult<Option<T>> {
        let path = self.0.path();
        self.1.borrow_mut().set_null_override(path, null_override)?;
        if self.1.borrow_mut().resolve_null_override(path)? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"), &self.1)))
        } else {
            Ok(None)
        }
    }
}


pub struct NullableFieldOwned<T: FieldOwned>(
    pub PropertyPath,
    Rc<RefCell<Option<DataContainerOwned>>>,
    PhantomData<T>,
);

impl<T: FieldOwned> FieldOwned for NullableFieldOwned<T> {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        NullableFieldOwned(property_path, data_container.clone(), PhantomData)
    }
}

impl<T: FieldOwned> NullableFieldOwned<T> {
    pub fn resolve_null(self) -> DataSetResult<Option<T>> {
        if self.resolve_null_override()? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"), &self.1)))
        } else {
            Ok(None)
        }
    }

    pub fn resolve_null_override(&self) -> DataSetResult<NullOverride> {
        self.1.borrow_mut().as_ref().ok_or(DataSetError::DataTaken)?.resolve_null_override(self.0.path())
    }

    pub fn set_null_override(
        &self,
        null_override: NullOverride,
    ) -> DataSetResult<Option<T>> {
        let path = self.0.path();
        self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.set_null_override(path, null_override)?;
        if self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.resolve_null_override(path)? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"), &self.1)))
        } else {
            Ok(None)
        }
    }
}




pub struct BooleanField(pub PropertyPath);

impl Field for BooleanField {
    fn new(property_path: PropertyPath) -> Self {
        BooleanField(property_path)
    }
}

impl BooleanField {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<bool> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_boolean()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: bool,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::Boolean(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<bool> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: bool,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct BooleanFieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for BooleanFieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        BooleanFieldReader(property_path, data_container)
    }
}

impl<'a> BooleanFieldReader<'a> {
    pub fn get(&self) -> DataSetResult<bool> {
        BooleanField::do_get(&self.0, self.1)
    }
}

pub struct BooleanFieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for BooleanFieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        BooleanFieldWriter(property_path, data_container.clone())
    }
}

impl<'a> BooleanFieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<bool> {
        BooleanField::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: bool,
    ) -> DataSetResult<Option<Value>> {
        BooleanField::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct BooleanFieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for BooleanFieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        BooleanFieldOwned(property_path, data_container.clone())
    }
}

impl BooleanFieldOwned {
    pub fn get(&self) -> DataSetResult<bool> {
        BooleanField::do_get(&self.0, self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: bool,
    ) -> DataSetResult<Option<Value>> {
        BooleanField::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
    }
}

pub struct I32Field(pub PropertyPath);

impl Field for I32Field {
    fn new(property_path: PropertyPath) -> Self {
        I32Field(property_path)
    }
}

impl I32Field {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<i32> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_i32()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: i32,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::I32(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<i32> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: i32,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct I32FieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for I32FieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        I32FieldReader(property_path, data_container)
    }
}

impl<'a> I32FieldReader<'a> {
    pub fn get(&self) -> DataSetResult<i32> {
        I32Field::do_get(&self.0, self.1)
    }
}

pub struct I32FieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for I32FieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        I32FieldWriter(property_path, data_container.clone())
    }
}

impl<'a> I32FieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<i32> {
        I32Field::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: i32,
    ) -> DataSetResult<Option<Value>> {
        I32Field::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct I32FieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for I32FieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        I32FieldOwned(property_path, data_container.clone())
    }
}

impl I32FieldOwned {
    pub fn get(&self) -> DataSetResult<i32> {
        I32Field::do_get(&self.0, self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: i32,
    ) -> DataSetResult<Option<Value>> {
        I32Field::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
    }
}


pub struct I64Field(pub PropertyPath);

impl Field for I64Field {
    fn new(property_path: PropertyPath) -> Self {
        I64Field(property_path)
    }
}

impl I64Field {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<i64> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_i64()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: i64,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::I64(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<i64> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: i64,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct I64FieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for I64FieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        I64FieldReader(property_path, data_container)
    }
}

impl<'a> I64FieldReader<'a> {
    pub fn get(&self) -> DataSetResult<i64> {
        I64Field::do_get(&self.0, self.1)
    }
}

pub struct I64FieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for I64FieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        I64FieldWriter(property_path, data_container.clone())
    }
}

impl<'a> I64FieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<i64> {
        I64Field::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: i64,
    ) -> DataSetResult<Option<Value>> {
        I64Field::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct I64FieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for I64FieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        I64FieldOwned(property_path, data_container.clone())
    }
}

impl I64FieldOwned {
    pub fn get(&self) -> DataSetResult<i64> {
        I64Field::do_get(&self.0, self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: i64,
    ) -> DataSetResult<Option<Value>> {
        I64Field::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
    }
}

pub struct U32Field(pub PropertyPath);

impl Field for U32Field {
    fn new(property_path: PropertyPath) -> Self {
        U32Field(property_path)
    }
}

impl U32Field {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<u32> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_u32()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: u32,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::U32(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<u32> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: u32,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct U32FieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for U32FieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        U32FieldReader(property_path, data_container)
    }
}

impl<'a> U32FieldReader<'a> {
    pub fn get(&self) -> DataSetResult<u32> {
        U32Field::do_get(&self.0, self.1)
    }
}

pub struct U32FieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for U32FieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        U32FieldWriter(property_path, data_container.clone())
    }
}

impl<'a> U32FieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<u32> {
        U32Field::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: u32,
    ) -> DataSetResult<Option<Value>> {
        U32Field::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct U32FieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for U32FieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        U32FieldOwned(property_path, data_container.clone())
    }
}

impl U32FieldOwned {
    pub fn get(&self) -> DataSetResult<u32> {
        U32Field::do_get(&self.0, self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: u32,
    ) -> DataSetResult<Option<Value>> {
        U32Field::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
    }
}

pub struct U64Field(pub PropertyPath);

impl Field for U64Field {
    fn new(property_path: PropertyPath) -> Self {
        U64Field(property_path)
    }
}

impl U64Field {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<u64> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_u64()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: u64,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::U64(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<u64> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: u64,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct U64FieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for U64FieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        U64FieldReader(property_path, data_container)
    }
}

impl<'a> U64FieldReader<'a> {
    pub fn get(&self) -> DataSetResult<u64> {
        U64Field::do_get(&self.0, self.1)
    }
}

pub struct U64FieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for U64FieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        U64FieldWriter(property_path, data_container.clone())
    }
}

impl<'a> U64FieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<u64> {
        U64Field::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: u64,
    ) -> DataSetResult<Option<Value>> {
        U64Field::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct U64FieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for U64FieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        U64FieldOwned(property_path, data_container.clone())
    }
}

impl U64FieldOwned {
    pub fn get(&self) -> DataSetResult<u64> {
        U64Field::do_get(&self.0, self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: u64,
    ) -> DataSetResult<Option<Value>> {
        U64Field::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
    }
}

pub struct F32Field(pub PropertyPath);

impl Field for F32Field {
    fn new(property_path: PropertyPath) -> Self {
        F32Field(property_path)
    }
}

impl F32Field {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<f32> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_f32()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: f32,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::F32(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<f32> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: f32,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct F32FieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for F32FieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        F32FieldReader(property_path, data_container)
    }
}

impl<'a> F32FieldReader<'a> {
    pub fn get(&self) -> DataSetResult<f32> {
        F32Field::do_get(&self.0, self.1)
    }
}

pub struct F32FieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for F32FieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        F32FieldWriter(property_path, data_container.clone())
    }
}

impl<'a> F32FieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<f32> {
        F32Field::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: f32,
    ) -> DataSetResult<Option<Value>> {
        F32Field::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct F32FieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for F32FieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        F32FieldOwned(property_path, data_container.clone())
    }
}

impl F32FieldOwned {
    pub fn get(&self) -> DataSetResult<f32> {
        F32Field::do_get(&self.0, self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: f32,
    ) -> DataSetResult<Option<Value>> {
        F32Field::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
    }
}

pub struct F64Field(pub PropertyPath);

impl Field for F64Field {
    fn new(property_path: PropertyPath) -> Self {
        F64Field(property_path)
    }
}

impl F64Field {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<f64> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_f64()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: f64,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::F64(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<f64> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: f64,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct F64FieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for F64FieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        F64FieldReader(property_path, data_container)
    }
}

impl<'a> F64FieldReader<'a> {
    pub fn get(&self) -> DataSetResult<f64> {
        F64Field::do_get(&self.0, self.1)
    }
}

pub struct F64FieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for F64FieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        F64FieldWriter(property_path, data_container.clone())
    }
}

impl<'a> F64FieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<f64> {
        F64Field::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: f64,
    ) -> DataSetResult<Option<Value>> {
        F64Field::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct F64FieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for F64FieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        F64FieldOwned(property_path, data_container.clone())
    }
}

impl F64FieldOwned {
    pub fn get(&self) -> DataSetResult<f64> {
        F64Field::do_get(&self.0, self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: f64,
    ) -> DataSetResult<Option<Value>> {
        F64Field::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
    }
}

pub struct BytesField(pub PropertyPath);

impl Field for BytesField {
    fn new(property_path: PropertyPath) -> Self {
        BytesField(property_path)
    }
}

impl BytesField {
    fn do_get<'a>(
        property_path: &PropertyPath,
        data_container: &'a DataContainer<'a>,
    ) -> DataSetResult<&'a Vec<u8>> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_bytes()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: Vec<u8>,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::Bytes(value)))
    }

    pub fn get<'a, 'b>(
        &'a self,
        data_container: &'b DataContainer<'b>,
    ) -> DataSetResult<&'b Vec<u8>> {
        Self::do_get(&self.0, &data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: Vec<u8>,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct BytesFieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for BytesFieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        BytesFieldReader(property_path, data_container)
    }
}

impl<'a> BytesFieldReader<'a> {
    pub fn get(&self) -> DataSetResult<&Vec<u8>> {
        BytesField::do_get(&self.0, &self.1)
    }
}

pub struct BytesFieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for BytesFieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        BytesFieldWriter(property_path, data_container.clone())
    }
}

impl<'a> BytesFieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<Vec<u8>> {
        // The writer has to clone because we can't return a reference to the interior of the Rc<RefCell<T>>
        // We could fix this by making the bytes type be an Arc<[u8]>
        Ok(self
            .1
            .borrow_mut()
            .resolve_property(self.0.path())?
            .as_bytes()
            .unwrap()
            .clone())
    }

    pub fn set(
        &self,
        value: Vec<u8>,
    ) -> DataSetResult<Option<Value>> {
        BytesField::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct BytesFieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for BytesFieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        BytesFieldOwned(property_path, data_container.clone())
    }
}

impl BytesFieldOwned {
    pub fn get(&self) -> DataSetResult<Vec<u8>> {
        // The writer has to clone because we can't return a reference to the interior of the Rc<RefCell<T>>
        // We could fix this by making the bytes type be an Arc<[u8]>
        Ok(self
            .1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .resolve_property(self.0.path())?
            .as_bytes()
            .unwrap()
            .clone())
    }

    pub fn set(
        &self,
        value: Vec<u8>,
    ) -> DataSetResult<Option<Value>> {
        BytesField::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
    }
}

pub struct StringField(pub PropertyPath);

impl Field for StringField {
    fn new(property_path: PropertyPath) -> Self {
        StringField(property_path)
    }
}

impl StringField {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<String> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_boolean()
            .unwrap()
            .to_string())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: String,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::String(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<String> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: String,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct StringFieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for StringFieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        StringFieldReader(property_path, data_container)
    }
}

impl<'a> StringFieldReader<'a> {
    pub fn get(&self) -> DataSetResult<String> {
        StringField::do_get(&self.0, self.1)
    }
}

pub struct StringFieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for StringFieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        StringFieldWriter(property_path, data_container.clone())
    }
}

impl<'a> StringFieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<String> {
        StringField::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: String,
    ) -> DataSetResult<Option<Value>> {
        StringField::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct StringFieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for StringFieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        StringFieldOwned(property_path, data_container.clone())
    }
}

impl StringFieldOwned {
    pub fn get(&self) -> DataSetResult<String> {
        StringField::do_get(&self.0, self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: String,
    ) -> DataSetResult<Option<Value>> {
        StringField::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
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
        data_container: DataContainer,
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

pub struct DynamicArrayFieldReader<'a, T: FieldReader<'a>>(
    pub PropertyPath,
    DataContainer<'a>,
    PhantomData<T>,
);

impl<'a, T: FieldReader<'a>> FieldReader<'a> for DynamicArrayFieldReader<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        DynamicArrayFieldReader(property_path, data_container, PhantomData)
    }
}

impl<'a, T: FieldReader<'a>> DynamicArrayFieldReader<'a, T> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1.resolve_dynamic_array(self.0.path())
    }

    pub fn entry(
        &self,
        entry_uuid: Uuid,
    ) -> T {
        T::new(self.0.push(&entry_uuid.to_string()), self.1)
    }
}

pub struct DynamicArrayFieldWriter<'a, T: FieldWriter<'a>>(
    pub PropertyPath,
    Rc<RefCell<DataContainerMut<'a>>>,
    PhantomData<T>,
);

impl<'a, T: FieldWriter<'a>> FieldWriter<'a> for DynamicArrayFieldWriter<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        DynamicArrayFieldWriter(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: FieldWriter<'a>> DynamicArrayFieldWriter<'a, T> {
    pub fn resolve_entries(
        &self,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.1.borrow_mut().resolve_dynamic_array(self.0.path())
    }

    pub fn entry(
        &'a self,
        entry_uuid: Uuid,
    ) -> T {
        T::new(self.0.push(&entry_uuid.to_string()), &self.1)
    }

    pub fn add_entry(&self) -> DataSetResult<Uuid> {
        self.1
            .borrow_mut()
            .add_dynamic_array_override(self.0.path())
    }
}

pub struct DynamicArrayFieldOwned<T: FieldOwned>(
    pub PropertyPath,
    Rc<RefCell<Option<DataContainerOwned>>>,
    PhantomData<T>,
);

impl<'a, T: FieldOwned> FieldOwned for DynamicArrayFieldOwned<T> {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        DynamicArrayFieldOwned(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: FieldOwned> DynamicArrayFieldOwned<T> {
    pub fn resolve_entries(
        &self,
    ) -> DataSetResult<Box<[Uuid]>> {
        self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.resolve_dynamic_array(self.0.path())
    }

    pub fn entry(
        &'a self,
        entry_uuid: Uuid,
    ) -> T {
        T::new(self.0.push(&entry_uuid.to_string()), &self.1)
    }

    pub fn add_entry(&self) -> DataSetResult<Uuid> {
        self.1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .add_dynamic_array_override(self.0.path())
    }
}

pub struct AssetRefField(pub PropertyPath);

impl Field for AssetRefField {
    fn new(property_path: PropertyPath) -> Self {
        AssetRefField(property_path)
    }
}

impl AssetRefField {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainer,
    ) -> DataSetResult<AssetId> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_asset_ref()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerMut,
        value: AssetId,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::AssetRef(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainer,
    ) -> DataSetResult<AssetId> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerMut,
        value: AssetId,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct AssetRefFieldReader<'a>(pub PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for AssetRefFieldReader<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainer<'a>,
    ) -> Self {
        AssetRefFieldReader(property_path, data_container)
    }
}

impl<'a> AssetRefFieldReader<'a> {
    pub fn get(&self) -> DataSetResult<AssetId> {
        AssetRefField::do_get(&self.0, self.1)
    }
}

pub struct AssetRefFieldWriter<'a>(pub PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for AssetRefFieldWriter<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerMut<'a>>>,
    ) -> Self {
        AssetRefFieldWriter(property_path, data_container.clone())
    }
}

impl<'a> AssetRefFieldWriter<'a> {
    pub fn get(&self) -> DataSetResult<AssetId> {
        AssetRefField::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: AssetId,
    ) -> DataSetResult<Option<Value>> {
        AssetRefField::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct AssetRefFieldOwned(pub PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for AssetRefFieldOwned {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainerOwned>>>,
    ) -> Self {
        AssetRefFieldOwned(property_path, data_container.clone())
    }
}

impl AssetRefFieldOwned {
    pub fn get(&self) -> DataSetResult<AssetId> {
        AssetRefField::do_get(&self.0, self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.read())
    }

    pub fn set(
        &self,
        value: AssetId,
    ) -> DataSetResult<Option<Value>> {
        AssetRefField::do_set(&self.0, &mut self.1.borrow_mut().as_mut().ok_or(DataSetError::DataTaken)?.to_mut(), value)
    }
}
