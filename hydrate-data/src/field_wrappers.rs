use crate::data_set_view::DataContainer;
use crate::value::ValueEnum;
use crate::{
    AssetId, DataContainerRef, DataContainerRefMut, DataSetError, DataSetResult, NullOverride,
    SchemaSet, SingleObject, Value,
};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Default, Clone)]
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

pub trait FieldAccessor {
    fn new(property_path: PropertyPath) -> Self;
}

pub trait FieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self;
}

pub trait FieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self;
}

pub trait Field {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self;
}

pub trait Enum: Sized {
    fn to_symbol_name(&self) -> &'static str;
    fn from_symbol_name(str: &str) -> Option<Self>;
}

pub trait RecordAccessor {
    fn schema_name() -> &'static str;

    fn new_single_object(schema_set: &SchemaSet) -> DataSetResult<SingleObject> {
        let schema = schema_set
            .find_named_type(Self::schema_name())
            .unwrap()
            .as_record()?;

        Ok(SingleObject::new(schema))
    }
}

pub trait RecordRef {
    fn schema_name() -> &'static str;

    //fn new(property_path: PropertyPath, data_container: DataContainerRef) -> Self;
}

pub trait RecordRefMut {
    fn schema_name() -> &'static str;
}

pub trait Record: Sized + Field {
    type Accessor: RecordAccessor + FieldAccessor;
    type Reader<'a>: RecordRef + FieldRef<'a>;
    type Writer<'a>: RecordRefMut + FieldRefMut<'a>;

    fn schema_name() -> &'static str;

    fn new_single_object(schema_set: &SchemaSet) -> DataSetResult<SingleObject> {
        let schema = schema_set
            .find_named_type(Self::schema_name())?
            .as_record()?;

        Ok(SingleObject::new(schema))
    }

    fn new_builder(schema_set: &SchemaSet) -> RecordBuilder<Self> {
        RecordBuilder::new(schema_set)
    }
}

pub struct RecordBuilder<T: Record + Field>(Rc<RefCell<Option<DataContainer>>>, T, PhantomData<T>);

impl<T: Record + Field> RecordBuilder<T> {
    pub fn new(schema_set: &SchemaSet) -> Self {
        let single_object = T::new_single_object(schema_set).unwrap();
        let data_container = DataContainer::from_single_object(single_object, schema_set.clone());
        let data_container = Rc::new(RefCell::new(Some(data_container)));
        let owned = T::new(Default::default(), &data_container);
        Self(data_container, owned, Default::default())
    }

    pub fn into_inner(self) -> DataSetResult<SingleObject> {
        // We are unwrapping an Rc, the RefCell, Option, and the DataContainer
        Ok(self
            .0
            .borrow_mut()
            .take()
            .ok_or(DataSetError::DataTaken)?
            .into_inner())
    }
}

impl<T: Record + Field> Deref for RecordBuilder<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl<T: Record + Field> DerefMut for RecordBuilder<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

pub struct EnumFieldAccessor<T: Enum>(PropertyPath, PhantomData<T>);

impl<T: Enum> FieldAccessor for EnumFieldAccessor<T> {
    fn new(property_path: PropertyPath) -> Self {
        EnumFieldAccessor(property_path, PhantomData::default())
    }
}

impl<T: Enum> EnumFieldAccessor<T> {
    pub fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<T> {
        let e = data_container.resolve_property(property_path.path())?;
        T::from_symbol_name(e.as_enum().unwrap().symbol_name())
            .ok_or(DataSetError::UnexpectedEnumSymbol)
    }

    pub fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
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
        data_container: DataContainerRef,
    ) -> DataSetResult<T> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct EnumFieldRef<'a, T>(pub PropertyPath, DataContainerRef<'a>, PhantomData<T>);

impl<'a, T: Enum> FieldRef<'a> for EnumFieldRef<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        EnumFieldRef(property_path, data_container, PhantomData)
    }
}

impl<'a, T: Enum> EnumFieldRef<'a, T> {
    pub fn get(&self) -> DataSetResult<T> {
        EnumFieldAccessor::<T>::do_get(&self.0, self.1.clone())
    }
}

pub struct EnumFieldRefMut<'a, T: Enum>(
    pub PropertyPath,
    Rc<RefCell<DataContainerRefMut<'a>>>,
    PhantomData<T>,
);

impl<'a, T: Enum> FieldRefMut<'a> for EnumFieldRefMut<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        EnumFieldRefMut(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: Enum> EnumFieldRefMut<'a, T> {
    pub fn get(&self) -> DataSetResult<T> {
        EnumFieldAccessor::<T>::do_get(&self.0, self.1.borrow().read())
    }

    pub fn set(
        &self,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        EnumFieldAccessor::<T>::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct EnumField<T: Enum>(
    pub PropertyPath,
    Rc<RefCell<Option<DataContainer>>>,
    PhantomData<T>,
);

impl<T: Enum> Field for EnumField<T> {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        EnumField(property_path, data_container.clone(), PhantomData)
    }
}

impl<T: Enum> EnumField<T> {
    pub fn get(&self) -> DataSetResult<T> {
        EnumFieldAccessor::<T>::do_get(
            &self.0,
            self.1
                .borrow()
                .as_ref()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set(
        &self,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        EnumFieldAccessor::<T>::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}

pub struct NullableFieldAccessor<T: FieldAccessor>(pub PropertyPath, PhantomData<T>);

impl<T: FieldAccessor> FieldAccessor for NullableFieldAccessor<T> {
    fn new(property_path: PropertyPath) -> Self {
        NullableFieldAccessor(property_path, PhantomData::default())
    }
}

impl<T: FieldAccessor> NullableFieldAccessor<T> {
    pub fn resolve_null(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<Option<T>> {
        if self.resolve_null_override(data_container)? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"))))
        } else {
            Ok(None)
        }
    }

    pub fn resolve_null_override(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<NullOverride> {
        data_container.resolve_null_override(self.0.path())
    }

    pub fn set_null_override(
        &self,
        data_container: &mut DataContainerRefMut,
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

pub struct NullableFieldRef<'a, T>(pub PropertyPath, DataContainerRef<'a>, PhantomData<T>);

impl<'a, T: FieldRef<'a>> FieldRef<'a> for NullableFieldRef<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        NullableFieldRef(property_path, data_container, PhantomData)
    }
}

impl<'a, T: FieldRef<'a>> NullableFieldRef<'a, T> {
    pub fn resolve_null(&self) -> DataSetResult<Option<T>> {
        if self.resolve_null_override()? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"), self.1.clone())))
        } else {
            Ok(None)
        }
    }

    pub fn resolve_null_override(&self) -> DataSetResult<NullOverride> {
        self.1.resolve_null_override(self.0.path())
    }
}

pub struct NullableFieldRefMut<'a, T: FieldRefMut<'a>>(
    pub PropertyPath,
    Rc<RefCell<DataContainerRefMut<'a>>>,
    PhantomData<T>,
);

impl<'a, T: FieldRefMut<'a>> FieldRefMut<'a> for NullableFieldRefMut<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        NullableFieldRefMut(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: FieldRefMut<'a>> NullableFieldRefMut<'a, T> {
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

pub struct NullableField<T: Field>(
    pub PropertyPath,
    Rc<RefCell<Option<DataContainer>>>,
    PhantomData<T>,
);

impl<T: Field> Field for NullableField<T> {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        NullableField(property_path, data_container.clone(), PhantomData)
    }
}

impl<T: Field> NullableField<T> {
    pub fn resolve_null(self) -> DataSetResult<Option<T>> {
        if self.resolve_null_override()? == NullOverride::SetNonNull {
            Ok(Some(T::new(self.0.push("value"), &self.1)))
        } else {
            Ok(None)
        }
    }

    pub fn resolve_null_override(&self) -> DataSetResult<NullOverride> {
        self.1
            .borrow_mut()
            .as_ref()
            .ok_or(DataSetError::DataTaken)?
            .resolve_null_override(self.0.path())
    }

    pub fn set_null_override(
        &self,
        null_override: NullOverride,
    ) -> DataSetResult<Option<T>> {
        let path = self.0.path();
        self.1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .set_null_override(path, null_override)?;
        if self
            .1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .resolve_null_override(path)?
            == NullOverride::SetNonNull
        {
            Ok(Some(T::new(self.0.push("value"), &self.1)))
        } else {
            Ok(None)
        }
    }
}

pub struct BooleanFieldAccessor(pub PropertyPath);

impl FieldAccessor for BooleanFieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        BooleanFieldAccessor(property_path)
    }
}

impl BooleanFieldAccessor {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<bool> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_boolean()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: bool,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::Boolean(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<bool> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: bool,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct BooleanFieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for BooleanFieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        BooleanFieldRef(property_path, data_container)
    }
}

impl<'a> BooleanFieldRef<'a> {
    pub fn get(&self) -> DataSetResult<bool> {
        BooleanFieldAccessor::do_get(&self.0, self.1.clone())
    }
}

pub struct BooleanFieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for BooleanFieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        BooleanFieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> BooleanFieldRefMut<'a> {
    pub fn get(&self) -> DataSetResult<bool> {
        BooleanFieldAccessor::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: bool,
    ) -> DataSetResult<Option<Value>> {
        BooleanFieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct BooleanField(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for BooleanField {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        BooleanField(property_path, data_container.clone())
    }
}

impl BooleanField {
    pub fn get(&self) -> DataSetResult<bool> {
        BooleanFieldAccessor::do_get(
            &self.0,
            self.1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set(
        &self,
        value: bool,
    ) -> DataSetResult<Option<Value>> {
        BooleanFieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}

pub struct I32FieldAccessor(pub PropertyPath);

impl FieldAccessor for I32FieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        I32FieldAccessor(property_path)
    }
}

impl I32FieldAccessor {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<i32> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_i32()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: i32,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::I32(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<i32> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: i32,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct I32FieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for I32FieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        I32FieldRef(property_path, data_container)
    }
}

impl<'a> I32FieldRef<'a> {
    pub fn get(&self) -> DataSetResult<i32> {
        I32FieldAccessor::do_get(&self.0, self.1.clone())
    }
}

pub struct I32FieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for I32FieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        I32FieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> I32FieldRefMut<'a> {
    pub fn get(&self) -> DataSetResult<i32> {
        I32FieldAccessor::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: i32,
    ) -> DataSetResult<Option<Value>> {
        I32FieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct I32Field(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for I32Field {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        I32Field(property_path, data_container.clone())
    }
}

impl I32Field {
    pub fn get(&self) -> DataSetResult<i32> {
        I32FieldAccessor::do_get(
            &self.0,
            self.1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set(
        &self,
        value: i32,
    ) -> DataSetResult<Option<Value>> {
        I32FieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}

pub struct I64FieldAccessor(pub PropertyPath);

impl FieldAccessor for I64FieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        I64FieldAccessor(property_path)
    }
}

impl I64FieldAccessor {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<i64> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_i64()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: i64,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::I64(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<i64> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: i64,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct I64FieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for I64FieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        I64FieldRef(property_path, data_container)
    }
}

impl<'a> I64FieldRef<'a> {
    pub fn get(&self) -> DataSetResult<i64> {
        I64FieldAccessor::do_get(&self.0, self.1.clone())
    }
}

pub struct I64FieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for I64FieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        I64FieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> I64FieldRefMut<'a> {
    pub fn get(&self) -> DataSetResult<i64> {
        I64FieldAccessor::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: i64,
    ) -> DataSetResult<Option<Value>> {
        I64FieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct I64Field(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for I64Field {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        I64Field(property_path, data_container.clone())
    }
}

impl I64Field {
    pub fn get(&self) -> DataSetResult<i64> {
        I64FieldAccessor::do_get(
            &self.0,
            self.1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set(
        &self,
        value: i64,
    ) -> DataSetResult<Option<Value>> {
        I64FieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}

pub struct U32FieldAccessor(pub PropertyPath);

impl FieldAccessor for U32FieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        U32FieldAccessor(property_path)
    }
}

impl U32FieldAccessor {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<u32> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_u32()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: u32,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::U32(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<u32> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: u32,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct U32FieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for U32FieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        U32FieldRef(property_path, data_container)
    }
}

impl<'a> U32FieldRef<'a> {
    pub fn get(&self) -> DataSetResult<u32> {
        U32FieldAccessor::do_get(&self.0, self.1.clone())
    }
}

pub struct U32FieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for U32FieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        U32FieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> U32FieldRefMut<'a> {
    pub fn get(&self) -> DataSetResult<u32> {
        U32FieldAccessor::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: u32,
    ) -> DataSetResult<Option<Value>> {
        U32FieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct U32Field(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for U32Field {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        U32Field(property_path, data_container.clone())
    }
}

impl U32Field {
    pub fn get(&self) -> DataSetResult<u32> {
        U32FieldAccessor::do_get(
            &self.0,
            self.1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set(
        &self,
        value: u32,
    ) -> DataSetResult<Option<Value>> {
        U32FieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}

pub struct U64FieldAccessor(pub PropertyPath);

impl FieldAccessor for U64FieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        U64FieldAccessor(property_path)
    }
}

impl U64FieldAccessor {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<u64> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_u64()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: u64,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::U64(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<u64> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: u64,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct U64FieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for U64FieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        U64FieldRef(property_path, data_container)
    }
}

impl<'a> U64FieldRef<'a> {
    pub fn get(&self) -> DataSetResult<u64> {
        U64FieldAccessor::do_get(&self.0, self.1.clone())
    }
}

pub struct U64FieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for U64FieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        U64FieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> U64FieldRefMut<'a> {
    pub fn get(&self) -> DataSetResult<u64> {
        U64FieldAccessor::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: u64,
    ) -> DataSetResult<Option<Value>> {
        U64FieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct U64Field(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for U64Field {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        U64Field(property_path, data_container.clone())
    }
}

impl U64Field {
    pub fn get(&self) -> DataSetResult<u64> {
        U64FieldAccessor::do_get(
            &self.0,
            self.1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set(
        &self,
        value: u64,
    ) -> DataSetResult<Option<Value>> {
        U64FieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}

pub struct F32FieldAccessor(pub PropertyPath);

impl FieldAccessor for F32FieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        F32FieldAccessor(property_path)
    }
}

impl F32FieldAccessor {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<f32> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_f32()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: f32,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::F32(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<f32> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: f32,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct F32FieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for F32FieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        F32FieldRef(property_path, data_container)
    }
}

impl<'a> F32FieldRef<'a> {
    pub fn get(&self) -> DataSetResult<f32> {
        F32FieldAccessor::do_get(&self.0, self.1.clone())
    }
}

pub struct F32FieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for F32FieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        F32FieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> F32FieldRefMut<'a> {
    pub fn get(&self) -> DataSetResult<f32> {
        F32FieldAccessor::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: f32,
    ) -> DataSetResult<Option<Value>> {
        F32FieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct F32Field(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for F32Field {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        F32Field(property_path, data_container.clone())
    }
}

impl F32Field {
    pub fn get(&self) -> DataSetResult<f32> {
        F32FieldAccessor::do_get(
            &self.0,
            self.1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set(
        &self,
        value: f32,
    ) -> DataSetResult<Option<Value>> {
        F32FieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}

pub struct F64FieldAccessor(pub PropertyPath);

impl FieldAccessor for F64FieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        F64FieldAccessor(property_path)
    }
}

impl F64FieldAccessor {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<f64> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_f64()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: f64,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::F64(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<f64> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: f64,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct F64FieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for F64FieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        F64FieldRef(property_path, data_container)
    }
}

impl<'a> F64FieldRef<'a> {
    pub fn get(&self) -> DataSetResult<f64> {
        F64FieldAccessor::do_get(&self.0, self.1.clone())
    }
}

pub struct F64FieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for F64FieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        F64FieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> F64FieldRefMut<'a> {
    pub fn get(&self) -> DataSetResult<f64> {
        F64FieldAccessor::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: f64,
    ) -> DataSetResult<Option<Value>> {
        F64FieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct F64Field(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for F64Field {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        F64Field(property_path, data_container.clone())
    }
}

impl F64Field {
    pub fn get(&self) -> DataSetResult<f64> {
        F64FieldAccessor::do_get(
            &self.0,
            self.1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set(
        &self,
        value: f64,
    ) -> DataSetResult<Option<Value>> {
        F64FieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}

pub struct BytesFieldAccessor(pub PropertyPath);

impl FieldAccessor for BytesFieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        BytesFieldAccessor(property_path)
    }
}

impl BytesFieldAccessor {
    fn do_get<'a>(
        property_path: &PropertyPath,
        data_container: &'a DataContainerRef<'a>,
    ) -> DataSetResult<&'a Arc<Vec<u8>>> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_bytes()
            .unwrap())
    }

    fn do_set<T: Into<Arc<Vec<u8>>>>(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::Bytes(value.into())))
    }

    pub fn get<'a, 'b>(
        &'a self,
        data_container: &'b DataContainerRef<'b>,
    ) -> DataSetResult<&'b Arc<Vec<u8>>> {
        Self::do_get(&self.0, &data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: Arc<Vec<u8>>,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct BytesFieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for BytesFieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        BytesFieldRef(property_path, data_container)
    }
}

impl<'a> BytesFieldRef<'a> {
    pub fn get(&self) -> DataSetResult<&Arc<Vec<u8>>> {
        BytesFieldAccessor::do_get(&self.0, &self.1)
    }
}

pub struct BytesFieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for BytesFieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        BytesFieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> BytesFieldRefMut<'a> {
    pub fn get(&self) -> DataSetResult<Arc<Vec<u8>>> {
        // The RefMut has to clone because we can't return a reference to the interior of the Rc<RefCell<T>>
        // We could fix this by making the bytes type be an Arc<[u8]>
        Ok(self
            .1
            .borrow_mut()
            .resolve_property(self.0.path())?
            .as_bytes()
            .unwrap()
            .clone())
    }

    pub fn set<T: Into<Arc<Vec<u8>>>>(
        &self,
        value: Arc<Vec<u8>>,
    ) -> DataSetResult<Option<Value>> {
        BytesFieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct BytesField(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for BytesField {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        BytesField(property_path, data_container.clone())
    }
}

impl BytesField {
    pub fn get(&self) -> DataSetResult<Arc<Vec<u8>>> {
        // The RefMut has to clone because we can't return a reference to the interior of the Rc<RefCell<T>>
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

    pub fn set<T: Into<Arc<Vec<u8>>>>(
        &self,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        BytesFieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}

pub struct StringFieldAccessor(pub PropertyPath);

impl FieldAccessor for StringFieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        StringFieldAccessor(property_path)
    }
}

impl StringFieldAccessor {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<Arc<String>> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_string()
            .unwrap()
            .clone())
    }

    fn do_set<T: Into<Arc<String>>>(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(
            property_path.path(),
            Some(Value::String(value.into().clone())),
        )
    }

    pub fn get(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<Arc<String>> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set<'a, T: Into<Arc<String>>>(
        &self,
        data_container: &'a mut DataContainerRefMut,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct StringFieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for StringFieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        StringFieldRef(property_path, data_container)
    }
}

impl<'a> StringFieldRef<'a> {
    pub fn get(&'a self) -> DataSetResult<Arc<String>> {
        StringFieldAccessor::do_get(&self.0, self.1.clone())
    }
}

pub struct StringFieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for StringFieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        StringFieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> StringFieldRefMut<'a> {
    pub fn get(&'a self) -> DataSetResult<Arc<String>> {
        StringFieldAccessor::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set<T: Into<Arc<String>>>(
        &self,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        StringFieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct StringField(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for StringField {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        StringField(property_path, data_container.clone())
    }
}

impl StringField {
    pub fn get(&self) -> DataSetResult<Arc<String>> {
        StringFieldAccessor::do_get(
            &self.0,
            self.1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set<T: Into<Arc<String>>>(
        &self,
        value: T,
    ) -> DataSetResult<Option<Value>> {
        StringFieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}











pub struct StaticArrayFieldAccessor<T: FieldAccessor>(pub PropertyPath, PhantomData<T>);

impl<T: FieldAccessor> FieldAccessor for StaticArrayFieldAccessor<T> {
    fn new(property_path: PropertyPath) -> Self {
        StaticArrayFieldAccessor(property_path, PhantomData::default())
    }
}

impl<T: FieldAccessor> StaticArrayFieldAccessor<T> {
    pub fn resolve_entries(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<Box<[Uuid]>> {
        data_container.resolve_dynamic_array_entries(self.0.path())
    }

    pub fn entry(
        &self,
        index: usize,
    ) -> T {
        T::new(self.0.push(&index.to_string()))
    }
}

pub struct StaticArrayFieldRef<'a, T: FieldRef<'a>>(
    pub PropertyPath,
    DataContainerRef<'a>,
    PhantomData<T>,
);

impl<'a, T: FieldRef<'a>> FieldRef<'a> for StaticArrayFieldRef<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        StaticArrayFieldRef(property_path, data_container, PhantomData)
    }
}

impl<'a, T: FieldRef<'a>> StaticArrayFieldRef<'a, T> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1.resolve_dynamic_array_entries(self.0.path())
    }

    pub fn entry(
        &self,
        index: usize,
    ) -> T {
        T::new(self.0.push(&index.to_string()), self.1.clone())
    }
}

pub struct StaticArrayFieldRefMut<'a, T: FieldRefMut<'a>>(
    pub PropertyPath,
    Rc<RefCell<DataContainerRefMut<'a>>>,
    PhantomData<T>,
);

impl<'a, T: FieldRefMut<'a>> FieldRefMut<'a> for StaticArrayFieldRefMut<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        StaticArrayFieldRefMut(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: FieldRefMut<'a>> StaticArrayFieldRefMut<'a, T> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1.borrow_mut().resolve_dynamic_array_entries(self.0.path())
    }

    pub fn entry(
        &'a self,
        index: usize,
    ) -> T {
        T::new(self.0.push(&index.to_string()), &self.1)
    }
}

pub struct StaticArrayField<T: Field>(
    pub PropertyPath,
    Rc<RefCell<Option<DataContainer>>>,
    PhantomData<T>,
);

impl<'a, T: Field> Field for StaticArrayField<T> {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        StaticArrayField(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: Field> StaticArrayField<T> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .resolve_dynamic_array_entries(self.0.path())
    }

    pub fn entry(
        &'a self,
        index: usize,
    ) -> T {
        T::new(self.0.push(&index.to_string()), &self.1)
    }
}




















pub struct DynamicArrayFieldAccessor<T: FieldAccessor>(pub PropertyPath, PhantomData<T>);

impl<T: FieldAccessor> FieldAccessor for DynamicArrayFieldAccessor<T> {
    fn new(property_path: PropertyPath) -> Self {
        DynamicArrayFieldAccessor(property_path, PhantomData::default())
    }
}

impl<T: FieldAccessor> DynamicArrayFieldAccessor<T> {
    pub fn resolve_entries(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<Box<[Uuid]>> {
        data_container.resolve_dynamic_array_entries(self.0.path())
    }

    pub fn entry(
        &self,
        entry_uuid: Uuid,
    ) -> T {
        T::new(self.0.push(&entry_uuid.to_string()))
    }

    pub fn add_entry(
        &self,
        data_container: &mut DataContainerRefMut,
    ) -> DataSetResult<Uuid> {
        data_container.add_dynamic_array_entry(self.0.path())
    }

    pub fn remove_entry(
        &self,
        data_container: &mut DataContainerRefMut,
        entry_id: Uuid
    ) -> DataSetResult<bool> {
        data_container.remove_dynamic_array_entry(self.0.path(), entry_id)
    }
}

pub struct DynamicArrayFieldRef<'a, T: FieldRef<'a>>(
    pub PropertyPath,
    DataContainerRef<'a>,
    PhantomData<T>,
);

impl<'a, T: FieldRef<'a>> FieldRef<'a> for DynamicArrayFieldRef<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        DynamicArrayFieldRef(property_path, data_container, PhantomData)
    }
}

impl<'a, T: FieldRef<'a>> DynamicArrayFieldRef<'a, T> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1.resolve_dynamic_array_entries(self.0.path())
    }

    pub fn entry(
        &self,
        entry_uuid: Uuid,
    ) -> T {
        T::new(self.0.push(&entry_uuid.to_string()), self.1.clone())
    }
}

pub struct DynamicArrayFieldRefMut<'a, T: FieldRefMut<'a>>(
    pub PropertyPath,
    Rc<RefCell<DataContainerRefMut<'a>>>,
    PhantomData<T>,
);

impl<'a, T: FieldRefMut<'a>> FieldRefMut<'a> for DynamicArrayFieldRefMut<'a, T> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        DynamicArrayFieldRefMut(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: FieldRefMut<'a>> DynamicArrayFieldRefMut<'a, T> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1.borrow_mut().resolve_dynamic_array_entries(self.0.path())
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
            .add_dynamic_array_entry(self.0.path())
    }

    pub fn remove_entry(&self, entry_id: Uuid) -> DataSetResult<bool> {
        self.1
            .borrow_mut()
            .remove_dynamic_array_entry(self.0.path(), entry_id)
    }
}

pub struct DynamicArrayField<T: Field>(
    pub PropertyPath,
    Rc<RefCell<Option<DataContainer>>>,
    PhantomData<T>,
);

impl<'a, T: Field> Field for DynamicArrayField<T> {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        DynamicArrayField(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, T: Field> DynamicArrayField<T> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .resolve_dynamic_array_entries(self.0.path())
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
            .add_dynamic_array_entry(self.0.path())
    }

    pub fn remove_entry(&self, entry_id: Uuid) -> DataSetResult<bool> {
        self.1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .remove_dynamic_array_entry(self.0.path(), entry_id)
    }
}











pub struct MapFieldAccessor<KeyT: FieldAccessor, ValueT: FieldAccessor>(pub PropertyPath, PhantomData<(KeyT, ValueT)>);

impl<KeyT: FieldAccessor, ValueT: FieldAccessor> FieldAccessor for MapFieldAccessor<KeyT, ValueT> {
    fn new(property_path: PropertyPath) -> Self {
        MapFieldAccessor(property_path, PhantomData::default())
    }
}

impl<KeyT: FieldAccessor, ValueT: FieldAccessor> MapFieldAccessor<KeyT, ValueT> {
    pub fn resolve_entries(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<Box<[Uuid]>> {
        data_container.resolve_map_entries(self.0.path())
    }

    pub fn key(
        &self,
        entry_uuid: Uuid,
    ) -> KeyT {
        KeyT::new(self.0.push(&entry_uuid.to_string()))
    }

    pub fn value(
        &self,
        entry_uuid: Uuid,
    ) -> ValueT {
        ValueT::new(self.0.push(&entry_uuid.to_string()))
    }

    pub fn add_entry(
        &self,
        data_container: &mut DataContainerRefMut,
    ) -> DataSetResult<Uuid> {
        data_container.add_map_entry(self.0.path())
    }

    pub fn remove_entry(
        &self,
        data_container: &mut DataContainerRefMut,
        entry_id: Uuid
    ) -> DataSetResult<bool> {
        data_container.remove_map_entry(self.0.path(), entry_id)
    }
}

pub struct MapFieldRef<'a, KeyT: FieldRef<'a>, ValueT: FieldRef<'a>>(
    pub PropertyPath,
    DataContainerRef<'a>,
    PhantomData<(KeyT, ValueT)>,
);

impl<'a, KeyT: FieldRef<'a>, ValueT: FieldRef<'a>> FieldRef<'a> for MapFieldRef<'a, KeyT, ValueT> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        MapFieldRef(property_path, data_container, PhantomData)
    }
}

impl<'a, KeyT: FieldRef<'a>, ValueT: FieldRef<'a>> MapFieldRef<'a, KeyT, ValueT> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1.resolve_map_entries(self.0.path())
    }

    pub fn key(
        &self,
        entry_uuid: Uuid,
    ) -> KeyT {
        KeyT::new(self.0.push(&entry_uuid.to_string()), self.1.clone())
    }

    pub fn value(
        &self,
        entry_uuid: Uuid,
    ) -> ValueT {
        ValueT::new(self.0.push(&entry_uuid.to_string()), self.1.clone())
    }
}

pub struct MapFieldRefMut<'a, KeyT: FieldRefMut<'a>, ValueT: FieldRefMut<'a>>(
    pub PropertyPath,
    Rc<RefCell<DataContainerRefMut<'a>>>,
    PhantomData<(KeyT, ValueT)>,
);

impl<'a, KeyT: FieldRefMut<'a>, ValueT: FieldRefMut<'a>> FieldRefMut<'a> for MapFieldRefMut<'a, KeyT, ValueT> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        MapFieldRefMut(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, KeyT: FieldRefMut<'a>, ValueT: FieldRefMut<'a>> MapFieldRefMut<'a, KeyT, ValueT> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1.borrow_mut().resolve_map_entries(self.0.path())
    }

    pub fn key(
        &'a self,
        entry_uuid: Uuid,
    ) -> KeyT {
        KeyT::new(self.0.push(&entry_uuid.to_string()), &self.1)
    }

    pub fn value(
        &'a self,
        entry_uuid: Uuid,
    ) -> ValueT {
        ValueT::new(self.0.push(&entry_uuid.to_string()), &self.1)
    }

    pub fn add_entry(&self) -> DataSetResult<Uuid> {
        self.1
            .borrow_mut()
            .add_map_entry(self.0.path())
    }

    pub fn remove_entry(&self, entry_id: Uuid) -> DataSetResult<bool> {
        self.1
            .borrow_mut()
            .remove_map_entry(self.0.path(), entry_id)
    }
}

pub struct MapField<KeyT: Field, ValueT: Field>(
    pub PropertyPath,
    Rc<RefCell<Option<DataContainer>>>,
    PhantomData<(KeyT, ValueT)>,
);

impl<'a, KeyT: Field, ValueT: Field> Field for MapField<KeyT, ValueT> {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        MapField(property_path, data_container.clone(), PhantomData)
    }
}

impl<'a, KeyT: Field, ValueT: Field> MapField<KeyT, ValueT> {
    pub fn resolve_entries(&self) -> DataSetResult<Box<[Uuid]>> {
        self.1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .resolve_map_entries(self.0.path())
    }

    pub fn key(
        &'a self,
        entry_uuid: Uuid,
    ) -> KeyT {
        KeyT::new(self.0.push(&entry_uuid.to_string()), &self.1)
    }

    pub fn value(
        &'a self,
        entry_uuid: Uuid,
    ) -> ValueT {
        ValueT::new(self.0.push(&entry_uuid.to_string()), &self.1)
    }

    pub fn add_entry(&self) -> DataSetResult<Uuid> {
        self.1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .add_map_entry(self.0.path())
    }

    pub fn remove_entry(&self, entry_id: Uuid) -> DataSetResult<bool> {
        self.1
            .borrow_mut()
            .as_mut()
            .ok_or(DataSetError::DataTaken)?
            .remove_map_entry(self.0.path(), entry_id)
    }
}





















pub struct AssetRefFieldAccessor(pub PropertyPath);

impl FieldAccessor for AssetRefFieldAccessor {
    fn new(property_path: PropertyPath) -> Self {
        AssetRefFieldAccessor(property_path)
    }
}

impl AssetRefFieldAccessor {
    fn do_get(
        property_path: &PropertyPath,
        data_container: DataContainerRef,
    ) -> DataSetResult<AssetId> {
        Ok(data_container
            .resolve_property(property_path.path())?
            .as_asset_ref()
            .unwrap())
    }

    fn do_set(
        property_path: &PropertyPath,
        data_container: &mut DataContainerRefMut,
        value: AssetId,
    ) -> DataSetResult<Option<Value>> {
        data_container.set_property_override(property_path.path(), Some(Value::AssetRef(value)))
    }

    pub fn get(
        &self,
        data_container: DataContainerRef,
    ) -> DataSetResult<AssetId> {
        Self::do_get(&self.0, data_container)
    }

    pub fn set(
        &self,
        data_container: &mut DataContainerRefMut,
        value: AssetId,
    ) -> DataSetResult<Option<Value>> {
        Self::do_set(&self.0, data_container, value)
    }
}

pub struct AssetRefFieldRef<'a>(pub PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for AssetRefFieldRef<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: DataContainerRef<'a>,
    ) -> Self {
        AssetRefFieldRef(property_path, data_container)
    }
}

impl<'a> AssetRefFieldRef<'a> {
    pub fn get(&self) -> DataSetResult<AssetId> {
        AssetRefFieldAccessor::do_get(&self.0, self.1.clone())
    }
}

pub struct AssetRefFieldRefMut<'a>(pub PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for AssetRefFieldRefMut<'a> {
    fn new(
        property_path: PropertyPath,
        data_container: &'a Rc<RefCell<DataContainerRefMut<'a>>>,
    ) -> Self {
        AssetRefFieldRefMut(property_path, data_container.clone())
    }
}

impl<'a> AssetRefFieldRefMut<'a> {
    pub fn get(&self) -> DataSetResult<AssetId> {
        AssetRefFieldAccessor::do_get(&self.0, self.1.borrow_mut().read())
    }

    pub fn set(
        &self,
        value: AssetId,
    ) -> DataSetResult<Option<Value>> {
        AssetRefFieldAccessor::do_set(&self.0, &mut *self.1.borrow_mut(), value)
    }
}

pub struct AssetRefField(pub PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for AssetRefField {
    fn new(
        property_path: PropertyPath,
        data_container: &Rc<RefCell<Option<DataContainer>>>,
    ) -> Self {
        AssetRefField(property_path, data_container.clone())
    }
}

impl AssetRefField {
    pub fn get(&self) -> DataSetResult<AssetId> {
        AssetRefFieldAccessor::do_get(
            &self.0,
            self.1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .read(),
        )
    }

    pub fn set(
        &self,
        value: AssetId,
    ) -> DataSetResult<Option<Value>> {
        AssetRefFieldAccessor::do_set(
            &self.0,
            &mut self
                .1
                .borrow_mut()
                .as_mut()
                .ok_or(DataSetError::DataTaken)?
                .to_mut(),
            value,
        )
    }
}
