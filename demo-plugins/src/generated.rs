// This file generated automatically by hydrate-codegen. Do not make manual edits. Use include!() to place these types in the intended location.
#[derive(Default)]
pub struct AllFieldsAccessor(PropertyPath);

impl FieldAccessor for AllFieldsAccessor {
    fn new(property_path: PropertyPath) -> Self {
        AllFieldsAccessor(property_path)
    }
}

impl RecordAccessor for AllFieldsAccessor {
    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl AllFieldsAccessor {
    pub fn boolean(&self) -> BooleanFieldAccessor {
        BooleanFieldAccessor::new(self.0.push("boolean"))
    }

    pub fn color(&self) -> ColorRgbaU8Accessor {
        ColorRgbaU8Accessor::new(self.0.push("color"))
    }

    pub fn dynamic_array_i32(&self) -> DynamicArrayFieldAccessor::<I32FieldAccessor> {
        DynamicArrayFieldAccessor::<I32FieldAccessor>::new(self.0.push("dynamic_array_i32"))
    }

    pub fn dynamic_array_recursive(&self) -> DynamicArrayFieldAccessor::<AllFieldsAccessor> {
        DynamicArrayFieldAccessor::<AllFieldsAccessor>::new(self.0.push("dynamic_array_recursive"))
    }

    pub fn dynamic_array_vec3(&self) -> DynamicArrayFieldAccessor::<Vec3Accessor> {
        DynamicArrayFieldAccessor::<Vec3Accessor>::new(self.0.push("dynamic_array_vec3"))
    }

    pub fn enum_field(&self) -> EnumFieldAccessor::<TestEnumEnum> {
        EnumFieldAccessor::<TestEnumEnum>::new(self.0.push("enum_field"))
    }

    pub fn f32(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("f32"))
    }

    pub fn f64(&self) -> F64FieldAccessor {
        F64FieldAccessor::new(self.0.push("f64"))
    }

    pub fn i32(&self) -> I32FieldAccessor {
        I32FieldAccessor::new(self.0.push("i32"))
    }

    pub fn i64(&self) -> I64FieldAccessor {
        I64FieldAccessor::new(self.0.push("i64"))
    }

    pub fn map_i32_vec3(&self) -> MapFieldAccessor::<I32FieldAccessor, Vec3Accessor> {
        MapFieldAccessor::<I32FieldAccessor, Vec3Accessor>::new(self.0.push("map_i32_vec3"))
    }

    pub fn map_test_enum_all_fields(&self) -> MapFieldAccessor::<EnumFieldAccessor::<TestEnumEnum>, AllFieldsAccessor> {
        MapFieldAccessor::<EnumFieldAccessor::<TestEnumEnum>, AllFieldsAccessor>::new(self.0.push("map_test_enum_all_fields"))
    }

    pub fn nullable_bool(&self) -> NullableFieldAccessor::<BooleanFieldAccessor> {
        NullableFieldAccessor::<BooleanFieldAccessor>::new(self.0.push("nullable_bool"))
    }

    pub fn nullable_recursive(&self) -> NullableFieldAccessor::<AllFieldsAccessor> {
        NullableFieldAccessor::<AllFieldsAccessor>::new(self.0.push("nullable_recursive"))
    }

    pub fn nullable_vec3(&self) -> NullableFieldAccessor::<Vec3Accessor> {
        NullableFieldAccessor::<Vec3Accessor>::new(self.0.push("nullable_vec3"))
    }

    pub fn record_recursive(&self) -> AllFieldsAccessor {
        AllFieldsAccessor::new(self.0.push("record_recursive"))
    }

    pub fn reference(&self) -> AssetRefFieldAccessor {
        AssetRefFieldAccessor::new(self.0.push("reference"))
    }

    pub fn static_array(&self) -> StaticArrayFieldAccessor::<Vec3Accessor> {
        StaticArrayFieldAccessor::<Vec3Accessor>::new(self.0.push("static_array"))
    }

    pub fn static_array_i32(&self) -> StaticArrayFieldAccessor::<I32FieldAccessor> {
        StaticArrayFieldAccessor::<I32FieldAccessor>::new(self.0.push("static_array_i32"))
    }

    pub fn static_array_recursive(&self) -> StaticArrayFieldAccessor::<AllFieldsAccessor> {
        StaticArrayFieldAccessor::<AllFieldsAccessor>::new(self.0.push("static_array_recursive"))
    }

    pub fn string(&self) -> StringFieldAccessor {
        StringFieldAccessor::new(self.0.push("string"))
    }

    pub fn u32(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("u32"))
    }

    pub fn u64(&self) -> U64FieldAccessor {
        U64FieldAccessor::new(self.0.push("u64"))
    }

    pub fn v3(&self) -> Vec3Accessor {
        Vec3Accessor::new(self.0.push("v3"))
    }

    pub fn v4(&self) -> Vec4Accessor {
        Vec4Accessor::new(self.0.push("v4"))
    }
}
pub struct AllFieldsRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for AllFieldsRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        AllFieldsRef(property_path, data_container)
    }
}

impl<'a> RecordRef for AllFieldsRef<'a> {
    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl<'a> AllFieldsRef<'a> {
    pub fn boolean(&self) -> BooleanFieldRef {
        BooleanFieldRef::new(self.0.push("boolean"), self.1.clone())
    }

    pub fn color(&self) -> ColorRgbaU8Ref {
        ColorRgbaU8Ref::new(self.0.push("color"), self.1.clone())
    }

    pub fn dynamic_array_i32(&self) -> DynamicArrayFieldRef::<I32FieldRef> {
        DynamicArrayFieldRef::<I32FieldRef>::new(self.0.push("dynamic_array_i32"), self.1.clone())
    }

    pub fn dynamic_array_recursive(&self) -> DynamicArrayFieldRef::<AllFieldsRef> {
        DynamicArrayFieldRef::<AllFieldsRef>::new(self.0.push("dynamic_array_recursive"), self.1.clone())
    }

    pub fn dynamic_array_vec3(&self) -> DynamicArrayFieldRef::<Vec3Ref> {
        DynamicArrayFieldRef::<Vec3Ref>::new(self.0.push("dynamic_array_vec3"), self.1.clone())
    }

    pub fn enum_field(&self) -> EnumFieldRef::<TestEnumEnum> {
        EnumFieldRef::<TestEnumEnum>::new(self.0.push("enum_field"), self.1.clone())
    }

    pub fn f32(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("f32"), self.1.clone())
    }

    pub fn f64(&self) -> F64FieldRef {
        F64FieldRef::new(self.0.push("f64"), self.1.clone())
    }

    pub fn i32(&self) -> I32FieldRef {
        I32FieldRef::new(self.0.push("i32"), self.1.clone())
    }

    pub fn i64(&self) -> I64FieldRef {
        I64FieldRef::new(self.0.push("i64"), self.1.clone())
    }

    pub fn map_i32_vec3(&self) -> MapFieldRef::<I32FieldRef, Vec3Ref> {
        MapFieldRef::<I32FieldRef, Vec3Ref>::new(self.0.push("map_i32_vec3"), self.1.clone())
    }

    pub fn map_test_enum_all_fields(&self) -> MapFieldRef::<EnumFieldRef::<TestEnumEnum>, AllFieldsRef> {
        MapFieldRef::<EnumFieldRef::<TestEnumEnum>, AllFieldsRef>::new(self.0.push("map_test_enum_all_fields"), self.1.clone())
    }

    pub fn nullable_bool(&self) -> NullableFieldRef::<BooleanFieldRef> {
        NullableFieldRef::<BooleanFieldRef>::new(self.0.push("nullable_bool"), self.1.clone())
    }

    pub fn nullable_recursive(&self) -> NullableFieldRef::<AllFieldsRef> {
        NullableFieldRef::<AllFieldsRef>::new(self.0.push("nullable_recursive"), self.1.clone())
    }

    pub fn nullable_vec3(&self) -> NullableFieldRef::<Vec3Ref> {
        NullableFieldRef::<Vec3Ref>::new(self.0.push("nullable_vec3"), self.1.clone())
    }

    pub fn record_recursive(&self) -> AllFieldsRef {
        AllFieldsRef::new(self.0.push("record_recursive"), self.1.clone())
    }

    pub fn reference(&self) -> AssetRefFieldRef {
        AssetRefFieldRef::new(self.0.push("reference"), self.1.clone())
    }

    pub fn static_array(&self) -> StaticArrayFieldRef::<Vec3Ref> {
        StaticArrayFieldRef::<Vec3Ref>::new(self.0.push("static_array"), self.1.clone())
    }

    pub fn static_array_i32(&self) -> StaticArrayFieldRef::<I32FieldRef> {
        StaticArrayFieldRef::<I32FieldRef>::new(self.0.push("static_array_i32"), self.1.clone())
    }

    pub fn static_array_recursive(&self) -> StaticArrayFieldRef::<AllFieldsRef> {
        StaticArrayFieldRef::<AllFieldsRef>::new(self.0.push("static_array_recursive"), self.1.clone())
    }

    pub fn string(&self) -> StringFieldRef {
        StringFieldRef::new(self.0.push("string"), self.1.clone())
    }

    pub fn u32(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("u32"), self.1.clone())
    }

    pub fn u64(&self) -> U64FieldRef {
        U64FieldRef::new(self.0.push("u64"), self.1.clone())
    }

    pub fn v3(&self) -> Vec3Ref {
        Vec3Ref::new(self.0.push("v3"), self.1.clone())
    }

    pub fn v4(&self) -> Vec4Ref {
        Vec4Ref::new(self.0.push("v4"), self.1.clone())
    }
}
pub struct AllFieldsRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for AllFieldsRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        AllFieldsRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for AllFieldsRefMut<'a> {
    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl<'a> AllFieldsRefMut<'a> {
    pub fn boolean(self: &'a Self) -> BooleanFieldRefMut {
        BooleanFieldRefMut::new(self.0.push("boolean"), &self.1)
    }

    pub fn color(self: &'a Self) -> ColorRgbaU8RefMut {
        ColorRgbaU8RefMut::new(self.0.push("color"), &self.1)
    }

    pub fn dynamic_array_i32(self: &'a Self) -> DynamicArrayFieldRefMut::<I32FieldRefMut> {
        DynamicArrayFieldRefMut::<I32FieldRefMut>::new(self.0.push("dynamic_array_i32"), &self.1)
    }

    pub fn dynamic_array_recursive(self: &'a Self) -> DynamicArrayFieldRefMut::<AllFieldsRefMut> {
        DynamicArrayFieldRefMut::<AllFieldsRefMut>::new(self.0.push("dynamic_array_recursive"), &self.1)
    }

    pub fn dynamic_array_vec3(self: &'a Self) -> DynamicArrayFieldRefMut::<Vec3RefMut> {
        DynamicArrayFieldRefMut::<Vec3RefMut>::new(self.0.push("dynamic_array_vec3"), &self.1)
    }

    pub fn enum_field(self: &'a Self) -> EnumFieldRefMut::<TestEnumEnum> {
        EnumFieldRefMut::<TestEnumEnum>::new(self.0.push("enum_field"), &self.1)
    }

    pub fn f32(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("f32"), &self.1)
    }

    pub fn f64(self: &'a Self) -> F64FieldRefMut {
        F64FieldRefMut::new(self.0.push("f64"), &self.1)
    }

    pub fn i32(self: &'a Self) -> I32FieldRefMut {
        I32FieldRefMut::new(self.0.push("i32"), &self.1)
    }

    pub fn i64(self: &'a Self) -> I64FieldRefMut {
        I64FieldRefMut::new(self.0.push("i64"), &self.1)
    }

    pub fn map_i32_vec3(self: &'a Self) -> MapFieldRefMut::<I32FieldRefMut, Vec3RefMut> {
        MapFieldRefMut::<I32FieldRefMut, Vec3RefMut>::new(self.0.push("map_i32_vec3"), &self.1)
    }

    pub fn map_test_enum_all_fields(self: &'a Self) -> MapFieldRefMut::<EnumFieldRefMut::<TestEnumEnum>, AllFieldsRefMut> {
        MapFieldRefMut::<EnumFieldRefMut::<TestEnumEnum>, AllFieldsRefMut>::new(self.0.push("map_test_enum_all_fields"), &self.1)
    }

    pub fn nullable_bool(self: &'a Self) -> NullableFieldRefMut::<BooleanFieldRefMut> {
        NullableFieldRefMut::<BooleanFieldRefMut>::new(self.0.push("nullable_bool"), &self.1)
    }

    pub fn nullable_recursive(self: &'a Self) -> NullableFieldRefMut::<AllFieldsRefMut> {
        NullableFieldRefMut::<AllFieldsRefMut>::new(self.0.push("nullable_recursive"), &self.1)
    }

    pub fn nullable_vec3(self: &'a Self) -> NullableFieldRefMut::<Vec3RefMut> {
        NullableFieldRefMut::<Vec3RefMut>::new(self.0.push("nullable_vec3"), &self.1)
    }

    pub fn record_recursive(self: &'a Self) -> AllFieldsRefMut {
        AllFieldsRefMut::new(self.0.push("record_recursive"), &self.1)
    }

    pub fn reference(self: &'a Self) -> AssetRefFieldRefMut {
        AssetRefFieldRefMut::new(self.0.push("reference"), &self.1)
    }

    pub fn static_array(self: &'a Self) -> StaticArrayFieldRefMut::<Vec3RefMut> {
        StaticArrayFieldRefMut::<Vec3RefMut>::new(self.0.push("static_array"), &self.1)
    }

    pub fn static_array_i32(self: &'a Self) -> StaticArrayFieldRefMut::<I32FieldRefMut> {
        StaticArrayFieldRefMut::<I32FieldRefMut>::new(self.0.push("static_array_i32"), &self.1)
    }

    pub fn static_array_recursive(self: &'a Self) -> StaticArrayFieldRefMut::<AllFieldsRefMut> {
        StaticArrayFieldRefMut::<AllFieldsRefMut>::new(self.0.push("static_array_recursive"), &self.1)
    }

    pub fn string(self: &'a Self) -> StringFieldRefMut {
        StringFieldRefMut::new(self.0.push("string"), &self.1)
    }

    pub fn u32(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("u32"), &self.1)
    }

    pub fn u64(self: &'a Self) -> U64FieldRefMut {
        U64FieldRefMut::new(self.0.push("u64"), &self.1)
    }

    pub fn v3(self: &'a Self) -> Vec3RefMut {
        Vec3RefMut::new(self.0.push("v3"), &self.1)
    }

    pub fn v4(self: &'a Self) -> Vec4RefMut {
        Vec4RefMut::new(self.0.push("v4"), &self.1)
    }
}
pub struct AllFieldsRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for AllFieldsRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        AllFieldsRecord(property_path, data_container.clone())
    }
}

impl Record for AllFieldsRecord {
    type Reader<'a> = AllFieldsRef<'a>;
    type Writer<'a> = AllFieldsRefMut<'a>;
    type Accessor = AllFieldsAccessor;

    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl AllFieldsRecord {
    pub fn boolean(self: &Self) -> BooleanField {
        BooleanField::new(self.0.push("boolean"), &self.1)
    }

    pub fn color(self: &Self) -> ColorRgbaU8Record {
        ColorRgbaU8Record::new(self.0.push("color"), &self.1)
    }

    pub fn dynamic_array_i32(self: &Self) -> DynamicArrayField::<I32Field> {
        DynamicArrayField::<I32Field>::new(self.0.push("dynamic_array_i32"), &self.1)
    }

    pub fn dynamic_array_recursive(self: &Self) -> DynamicArrayField::<AllFieldsRecord> {
        DynamicArrayField::<AllFieldsRecord>::new(self.0.push("dynamic_array_recursive"), &self.1)
    }

    pub fn dynamic_array_vec3(self: &Self) -> DynamicArrayField::<Vec3Record> {
        DynamicArrayField::<Vec3Record>::new(self.0.push("dynamic_array_vec3"), &self.1)
    }

    pub fn enum_field(self: &Self) -> EnumField::<TestEnumEnum> {
        EnumField::<TestEnumEnum>::new(self.0.push("enum_field"), &self.1)
    }

    pub fn f32(self: &Self) -> F32Field {
        F32Field::new(self.0.push("f32"), &self.1)
    }

    pub fn f64(self: &Self) -> F64Field {
        F64Field::new(self.0.push("f64"), &self.1)
    }

    pub fn i32(self: &Self) -> I32Field {
        I32Field::new(self.0.push("i32"), &self.1)
    }

    pub fn i64(self: &Self) -> I64Field {
        I64Field::new(self.0.push("i64"), &self.1)
    }

    pub fn map_i32_vec3(self: &Self) -> MapField::<I32Field, Vec3Record> {
        MapField::<I32Field, Vec3Record>::new(self.0.push("map_i32_vec3"), &self.1)
    }

    pub fn map_test_enum_all_fields(self: &Self) -> MapField::<EnumField::<TestEnumEnum>, AllFieldsRecord> {
        MapField::<EnumField::<TestEnumEnum>, AllFieldsRecord>::new(self.0.push("map_test_enum_all_fields"), &self.1)
    }

    pub fn nullable_bool(self: &Self) -> NullableField::<BooleanField> {
        NullableField::<BooleanField>::new(self.0.push("nullable_bool"), &self.1)
    }

    pub fn nullable_recursive(self: &Self) -> NullableField::<AllFieldsRecord> {
        NullableField::<AllFieldsRecord>::new(self.0.push("nullable_recursive"), &self.1)
    }

    pub fn nullable_vec3(self: &Self) -> NullableField::<Vec3Record> {
        NullableField::<Vec3Record>::new(self.0.push("nullable_vec3"), &self.1)
    }

    pub fn record_recursive(self: &Self) -> AllFieldsRecord {
        AllFieldsRecord::new(self.0.push("record_recursive"), &self.1)
    }

    pub fn reference(self: &Self) -> AssetRefField {
        AssetRefField::new(self.0.push("reference"), &self.1)
    }

    pub fn static_array(self: &Self) -> StaticArrayField::<Vec3Record> {
        StaticArrayField::<Vec3Record>::new(self.0.push("static_array"), &self.1)
    }

    pub fn static_array_i32(self: &Self) -> StaticArrayField::<I32Field> {
        StaticArrayField::<I32Field>::new(self.0.push("static_array_i32"), &self.1)
    }

    pub fn static_array_recursive(self: &Self) -> StaticArrayField::<AllFieldsRecord> {
        StaticArrayField::<AllFieldsRecord>::new(self.0.push("static_array_recursive"), &self.1)
    }

    pub fn string(self: &Self) -> StringField {
        StringField::new(self.0.push("string"), &self.1)
    }

    pub fn u32(self: &Self) -> U32Field {
        U32Field::new(self.0.push("u32"), &self.1)
    }

    pub fn u64(self: &Self) -> U64Field {
        U64Field::new(self.0.push("u64"), &self.1)
    }

    pub fn v3(self: &Self) -> Vec3Record {
        Vec3Record::new(self.0.push("v3"), &self.1)
    }

    pub fn v4(self: &Self) -> Vec4Record {
        Vec4Record::new(self.0.push("v4"), &self.1)
    }
}
#[derive(Default)]
pub struct ColorRgbaU8Accessor(PropertyPath);

impl FieldAccessor for ColorRgbaU8Accessor {
    fn new(property_path: PropertyPath) -> Self {
        ColorRgbaU8Accessor(property_path)
    }
}

impl RecordAccessor for ColorRgbaU8Accessor {
    fn schema_name() -> &'static str {
        "ColorRgbaU8"
    }
}

impl ColorRgbaU8Accessor {
    pub fn a(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("a"))
    }

    pub fn b(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("b"))
    }

    pub fn g(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("g"))
    }

    pub fn r(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("r"))
    }
}
pub struct ColorRgbaU8Ref<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for ColorRgbaU8Ref<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        ColorRgbaU8Ref(property_path, data_container)
    }
}

impl<'a> RecordRef for ColorRgbaU8Ref<'a> {
    fn schema_name() -> &'static str {
        "ColorRgbaU8"
    }
}

impl<'a> ColorRgbaU8Ref<'a> {
    pub fn a(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("a"), self.1.clone())
    }

    pub fn b(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("b"), self.1.clone())
    }

    pub fn g(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("g"), self.1.clone())
    }

    pub fn r(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("r"), self.1.clone())
    }
}
pub struct ColorRgbaU8RefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for ColorRgbaU8RefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        ColorRgbaU8RefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for ColorRgbaU8RefMut<'a> {
    fn schema_name() -> &'static str {
        "ColorRgbaU8"
    }
}

impl<'a> ColorRgbaU8RefMut<'a> {
    pub fn a(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("a"), &self.1)
    }

    pub fn b(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("b"), &self.1)
    }

    pub fn g(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("g"), &self.1)
    }

    pub fn r(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("r"), &self.1)
    }
}
pub struct ColorRgbaU8Record(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for ColorRgbaU8Record {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        ColorRgbaU8Record(property_path, data_container.clone())
    }
}

impl Record for ColorRgbaU8Record {
    type Reader<'a> = ColorRgbaU8Ref<'a>;
    type Writer<'a> = ColorRgbaU8RefMut<'a>;
    type Accessor = ColorRgbaU8Accessor;

    fn schema_name() -> &'static str {
        "ColorRgbaU8"
    }
}

impl ColorRgbaU8Record {
    pub fn a(self: &Self) -> U32Field {
        U32Field::new(self.0.push("a"), &self.1)
    }

    pub fn b(self: &Self) -> U32Field {
        U32Field::new(self.0.push("b"), &self.1)
    }

    pub fn g(self: &Self) -> U32Field {
        U32Field::new(self.0.push("g"), &self.1)
    }

    pub fn r(self: &Self) -> U32Field {
        U32Field::new(self.0.push("r"), &self.1)
    }
}
#[derive(Default)]
pub struct GlslBuildTargetAssetAccessor(PropertyPath);

impl FieldAccessor for GlslBuildTargetAssetAccessor {
    fn new(property_path: PropertyPath) -> Self {
        GlslBuildTargetAssetAccessor(property_path)
    }
}

impl RecordAccessor for GlslBuildTargetAssetAccessor {
    fn schema_name() -> &'static str {
        "GlslBuildTargetAsset"
    }
}

impl GlslBuildTargetAssetAccessor {
    pub fn entry_point(&self) -> StringFieldAccessor {
        StringFieldAccessor::new(self.0.push("entry_point"))
    }

    pub fn source_file(&self) -> AssetRefFieldAccessor {
        AssetRefFieldAccessor::new(self.0.push("source_file"))
    }
}
pub struct GlslBuildTargetAssetRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for GlslBuildTargetAssetRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        GlslBuildTargetAssetRef(property_path, data_container)
    }
}

impl<'a> RecordRef for GlslBuildTargetAssetRef<'a> {
    fn schema_name() -> &'static str {
        "GlslBuildTargetAsset"
    }
}

impl<'a> GlslBuildTargetAssetRef<'a> {
    pub fn entry_point(&self) -> StringFieldRef {
        StringFieldRef::new(self.0.push("entry_point"), self.1.clone())
    }

    pub fn source_file(&self) -> AssetRefFieldRef {
        AssetRefFieldRef::new(self.0.push("source_file"), self.1.clone())
    }
}
pub struct GlslBuildTargetAssetRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for GlslBuildTargetAssetRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        GlslBuildTargetAssetRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for GlslBuildTargetAssetRefMut<'a> {
    fn schema_name() -> &'static str {
        "GlslBuildTargetAsset"
    }
}

impl<'a> GlslBuildTargetAssetRefMut<'a> {
    pub fn entry_point(self: &'a Self) -> StringFieldRefMut {
        StringFieldRefMut::new(self.0.push("entry_point"), &self.1)
    }

    pub fn source_file(self: &'a Self) -> AssetRefFieldRefMut {
        AssetRefFieldRefMut::new(self.0.push("source_file"), &self.1)
    }
}
pub struct GlslBuildTargetAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GlslBuildTargetAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GlslBuildTargetAssetRecord(property_path, data_container.clone())
    }
}

impl Record for GlslBuildTargetAssetRecord {
    type Reader<'a> = GlslBuildTargetAssetRef<'a>;
    type Writer<'a> = GlslBuildTargetAssetRefMut<'a>;
    type Accessor = GlslBuildTargetAssetAccessor;

    fn schema_name() -> &'static str {
        "GlslBuildTargetAsset"
    }
}

impl GlslBuildTargetAssetRecord {
    pub fn entry_point(self: &Self) -> StringField {
        StringField::new(self.0.push("entry_point"), &self.1)
    }

    pub fn source_file(self: &Self) -> AssetRefField {
        AssetRefField::new(self.0.push("source_file"), &self.1)
    }
}
#[derive(Default)]
pub struct GlslSourceFileAssetAccessor(PropertyPath);

impl FieldAccessor for GlslSourceFileAssetAccessor {
    fn new(property_path: PropertyPath) -> Self {
        GlslSourceFileAssetAccessor(property_path)
    }
}

impl RecordAccessor for GlslSourceFileAssetAccessor {
    fn schema_name() -> &'static str {
        "GlslSourceFileAsset"
    }
}

impl GlslSourceFileAssetAccessor {
}
pub struct GlslSourceFileAssetRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for GlslSourceFileAssetRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        GlslSourceFileAssetRef(property_path, data_container)
    }
}

impl<'a> RecordRef for GlslSourceFileAssetRef<'a> {
    fn schema_name() -> &'static str {
        "GlslSourceFileAsset"
    }
}

impl<'a> GlslSourceFileAssetRef<'a> {
}
pub struct GlslSourceFileAssetRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for GlslSourceFileAssetRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        GlslSourceFileAssetRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for GlslSourceFileAssetRefMut<'a> {
    fn schema_name() -> &'static str {
        "GlslSourceFileAsset"
    }
}

impl<'a> GlslSourceFileAssetRefMut<'a> {
}
pub struct GlslSourceFileAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GlslSourceFileAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GlslSourceFileAssetRecord(property_path, data_container.clone())
    }
}

impl Record for GlslSourceFileAssetRecord {
    type Reader<'a> = GlslSourceFileAssetRef<'a>;
    type Writer<'a> = GlslSourceFileAssetRefMut<'a>;
    type Accessor = GlslSourceFileAssetAccessor;

    fn schema_name() -> &'static str {
        "GlslSourceFileAsset"
    }
}

impl GlslSourceFileAssetRecord {
}
#[derive(Default)]
pub struct GlslSourceFileImportedDataAccessor(PropertyPath);

impl FieldAccessor for GlslSourceFileImportedDataAccessor {
    fn new(property_path: PropertyPath) -> Self {
        GlslSourceFileImportedDataAccessor(property_path)
    }
}

impl RecordAccessor for GlslSourceFileImportedDataAccessor {
    fn schema_name() -> &'static str {
        "GlslSourceFileImportedData"
    }
}

impl GlslSourceFileImportedDataAccessor {
    pub fn code(&self) -> StringFieldAccessor {
        StringFieldAccessor::new(self.0.push("code"))
    }
}
pub struct GlslSourceFileImportedDataRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for GlslSourceFileImportedDataRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        GlslSourceFileImportedDataRef(property_path, data_container)
    }
}

impl<'a> RecordRef for GlslSourceFileImportedDataRef<'a> {
    fn schema_name() -> &'static str {
        "GlslSourceFileImportedData"
    }
}

impl<'a> GlslSourceFileImportedDataRef<'a> {
    pub fn code(&self) -> StringFieldRef {
        StringFieldRef::new(self.0.push("code"), self.1.clone())
    }
}
pub struct GlslSourceFileImportedDataRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for GlslSourceFileImportedDataRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        GlslSourceFileImportedDataRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for GlslSourceFileImportedDataRefMut<'a> {
    fn schema_name() -> &'static str {
        "GlslSourceFileImportedData"
    }
}

impl<'a> GlslSourceFileImportedDataRefMut<'a> {
    pub fn code(self: &'a Self) -> StringFieldRefMut {
        StringFieldRefMut::new(self.0.push("code"), &self.1)
    }
}
pub struct GlslSourceFileImportedDataRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GlslSourceFileImportedDataRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GlslSourceFileImportedDataRecord(property_path, data_container.clone())
    }
}

impl Record for GlslSourceFileImportedDataRecord {
    type Reader<'a> = GlslSourceFileImportedDataRef<'a>;
    type Writer<'a> = GlslSourceFileImportedDataRefMut<'a>;
    type Accessor = GlslSourceFileImportedDataAccessor;

    fn schema_name() -> &'static str {
        "GlslSourceFileImportedData"
    }
}

impl GlslSourceFileImportedDataRecord {
    pub fn code(self: &Self) -> StringField {
        StringField::new(self.0.push("code"), &self.1)
    }
}
#[derive(Default)]
pub struct GpuBufferAssetAccessor(PropertyPath);

impl FieldAccessor for GpuBufferAssetAccessor {
    fn new(property_path: PropertyPath) -> Self {
        GpuBufferAssetAccessor(property_path)
    }
}

impl RecordAccessor for GpuBufferAssetAccessor {
    fn schema_name() -> &'static str {
        "GpuBufferAsset"
    }
}

impl GpuBufferAssetAccessor {
}
pub struct GpuBufferAssetRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for GpuBufferAssetRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        GpuBufferAssetRef(property_path, data_container)
    }
}

impl<'a> RecordRef for GpuBufferAssetRef<'a> {
    fn schema_name() -> &'static str {
        "GpuBufferAsset"
    }
}

impl<'a> GpuBufferAssetRef<'a> {
}
pub struct GpuBufferAssetRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for GpuBufferAssetRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        GpuBufferAssetRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for GpuBufferAssetRefMut<'a> {
    fn schema_name() -> &'static str {
        "GpuBufferAsset"
    }
}

impl<'a> GpuBufferAssetRefMut<'a> {
}
pub struct GpuBufferAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GpuBufferAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GpuBufferAssetRecord(property_path, data_container.clone())
    }
}

impl Record for GpuBufferAssetRecord {
    type Reader<'a> = GpuBufferAssetRef<'a>;
    type Writer<'a> = GpuBufferAssetRefMut<'a>;
    type Accessor = GpuBufferAssetAccessor;

    fn schema_name() -> &'static str {
        "GpuBufferAsset"
    }
}

impl GpuBufferAssetRecord {
}
#[derive(Default)]
pub struct GpuBufferImportedDataAccessor(PropertyPath);

impl FieldAccessor for GpuBufferImportedDataAccessor {
    fn new(property_path: PropertyPath) -> Self {
        GpuBufferImportedDataAccessor(property_path)
    }
}

impl RecordAccessor for GpuBufferImportedDataAccessor {
    fn schema_name() -> &'static str {
        "GpuBufferImportedData"
    }
}

impl GpuBufferImportedDataAccessor {
    pub fn alignment(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("alignment"))
    }

    pub fn data(&self) -> BytesFieldAccessor {
        BytesFieldAccessor::new(self.0.push("data"))
    }

    pub fn resource_type(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("resource_type"))
    }
}
pub struct GpuBufferImportedDataRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for GpuBufferImportedDataRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        GpuBufferImportedDataRef(property_path, data_container)
    }
}

impl<'a> RecordRef for GpuBufferImportedDataRef<'a> {
    fn schema_name() -> &'static str {
        "GpuBufferImportedData"
    }
}

impl<'a> GpuBufferImportedDataRef<'a> {
    pub fn alignment(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("alignment"), self.1.clone())
    }

    pub fn data(&self) -> BytesFieldRef {
        BytesFieldRef::new(self.0.push("data"), self.1.clone())
    }

    pub fn resource_type(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("resource_type"), self.1.clone())
    }
}
pub struct GpuBufferImportedDataRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for GpuBufferImportedDataRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        GpuBufferImportedDataRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for GpuBufferImportedDataRefMut<'a> {
    fn schema_name() -> &'static str {
        "GpuBufferImportedData"
    }
}

impl<'a> GpuBufferImportedDataRefMut<'a> {
    pub fn alignment(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("alignment"), &self.1)
    }

    pub fn data(self: &'a Self) -> BytesFieldRefMut {
        BytesFieldRefMut::new(self.0.push("data"), &self.1)
    }

    pub fn resource_type(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("resource_type"), &self.1)
    }
}
pub struct GpuBufferImportedDataRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GpuBufferImportedDataRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GpuBufferImportedDataRecord(property_path, data_container.clone())
    }
}

impl Record for GpuBufferImportedDataRecord {
    type Reader<'a> = GpuBufferImportedDataRef<'a>;
    type Writer<'a> = GpuBufferImportedDataRefMut<'a>;
    type Accessor = GpuBufferImportedDataAccessor;

    fn schema_name() -> &'static str {
        "GpuBufferImportedData"
    }
}

impl GpuBufferImportedDataRecord {
    pub fn alignment(self: &Self) -> U32Field {
        U32Field::new(self.0.push("alignment"), &self.1)
    }

    pub fn data(self: &Self) -> BytesField {
        BytesField::new(self.0.push("data"), &self.1)
    }

    pub fn resource_type(self: &Self) -> U32Field {
        U32Field::new(self.0.push("resource_type"), &self.1)
    }
}
#[derive(Default)]
pub struct GpuImageAssetAccessor(PropertyPath);

impl FieldAccessor for GpuImageAssetAccessor {
    fn new(property_path: PropertyPath) -> Self {
        GpuImageAssetAccessor(property_path)
    }
}

impl RecordAccessor for GpuImageAssetAccessor {
    fn schema_name() -> &'static str {
        "GpuImageAsset"
    }
}

impl GpuImageAssetAccessor {
    pub fn compress(&self) -> BooleanFieldAccessor {
        BooleanFieldAccessor::new(self.0.push("compress"))
    }
}
pub struct GpuImageAssetRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for GpuImageAssetRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        GpuImageAssetRef(property_path, data_container)
    }
}

impl<'a> RecordRef for GpuImageAssetRef<'a> {
    fn schema_name() -> &'static str {
        "GpuImageAsset"
    }
}

impl<'a> GpuImageAssetRef<'a> {
    pub fn compress(&self) -> BooleanFieldRef {
        BooleanFieldRef::new(self.0.push("compress"), self.1.clone())
    }
}
pub struct GpuImageAssetRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for GpuImageAssetRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        GpuImageAssetRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for GpuImageAssetRefMut<'a> {
    fn schema_name() -> &'static str {
        "GpuImageAsset"
    }
}

impl<'a> GpuImageAssetRefMut<'a> {
    pub fn compress(self: &'a Self) -> BooleanFieldRefMut {
        BooleanFieldRefMut::new(self.0.push("compress"), &self.1)
    }
}
pub struct GpuImageAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GpuImageAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GpuImageAssetRecord(property_path, data_container.clone())
    }
}

impl Record for GpuImageAssetRecord {
    type Reader<'a> = GpuImageAssetRef<'a>;
    type Writer<'a> = GpuImageAssetRefMut<'a>;
    type Accessor = GpuImageAssetAccessor;

    fn schema_name() -> &'static str {
        "GpuImageAsset"
    }
}

impl GpuImageAssetRecord {
    pub fn compress(self: &Self) -> BooleanField {
        BooleanField::new(self.0.push("compress"), &self.1)
    }
}
#[derive(Default)]
pub struct GpuImageImportedDataAccessor(PropertyPath);

impl FieldAccessor for GpuImageImportedDataAccessor {
    fn new(property_path: PropertyPath) -> Self {
        GpuImageImportedDataAccessor(property_path)
    }
}

impl RecordAccessor for GpuImageImportedDataAccessor {
    fn schema_name() -> &'static str {
        "GpuImageImportedData"
    }
}

impl GpuImageImportedDataAccessor {
    pub fn height(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("height"))
    }

    pub fn image_bytes(&self) -> BytesFieldAccessor {
        BytesFieldAccessor::new(self.0.push("image_bytes"))
    }

    pub fn width(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("width"))
    }
}
pub struct GpuImageImportedDataRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for GpuImageImportedDataRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        GpuImageImportedDataRef(property_path, data_container)
    }
}

impl<'a> RecordRef for GpuImageImportedDataRef<'a> {
    fn schema_name() -> &'static str {
        "GpuImageImportedData"
    }
}

impl<'a> GpuImageImportedDataRef<'a> {
    pub fn height(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("height"), self.1.clone())
    }

    pub fn image_bytes(&self) -> BytesFieldRef {
        BytesFieldRef::new(self.0.push("image_bytes"), self.1.clone())
    }

    pub fn width(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("width"), self.1.clone())
    }
}
pub struct GpuImageImportedDataRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for GpuImageImportedDataRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        GpuImageImportedDataRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for GpuImageImportedDataRefMut<'a> {
    fn schema_name() -> &'static str {
        "GpuImageImportedData"
    }
}

impl<'a> GpuImageImportedDataRefMut<'a> {
    pub fn height(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("height"), &self.1)
    }

    pub fn image_bytes(self: &'a Self) -> BytesFieldRefMut {
        BytesFieldRefMut::new(self.0.push("image_bytes"), &self.1)
    }

    pub fn width(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("width"), &self.1)
    }
}
pub struct GpuImageImportedDataRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GpuImageImportedDataRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GpuImageImportedDataRecord(property_path, data_container.clone())
    }
}

impl Record for GpuImageImportedDataRecord {
    type Reader<'a> = GpuImageImportedDataRef<'a>;
    type Writer<'a> = GpuImageImportedDataRefMut<'a>;
    type Accessor = GpuImageImportedDataAccessor;

    fn schema_name() -> &'static str {
        "GpuImageImportedData"
    }
}

impl GpuImageImportedDataRecord {
    pub fn height(self: &Self) -> U32Field {
        U32Field::new(self.0.push("height"), &self.1)
    }

    pub fn image_bytes(self: &Self) -> BytesField {
        BytesField::new(self.0.push("image_bytes"), &self.1)
    }

    pub fn width(self: &Self) -> U32Field {
        U32Field::new(self.0.push("width"), &self.1)
    }
}
#[derive(Copy, Clone)]
pub enum MeshAdvBlendMethodEnum {
    Opaque,
    AlphaClip,
    AlphaBlend,
}

impl Enum for MeshAdvBlendMethodEnum {
    fn to_symbol_name(&self) -> &'static str {
        match self {
            MeshAdvBlendMethodEnum::Opaque => "Opaque",
            MeshAdvBlendMethodEnum::AlphaClip => "AlphaClip",
            MeshAdvBlendMethodEnum::AlphaBlend => "AlphaBlend",
        }
    }

    fn from_symbol_name(str: &str) -> Option<MeshAdvBlendMethodEnum> {
        match str {
            "Opaque" => Some(MeshAdvBlendMethodEnum::Opaque),
            "OPAQUE" => Some(MeshAdvBlendMethodEnum::Opaque),
            "AlphaClip" => Some(MeshAdvBlendMethodEnum::AlphaClip),
            "ALPHA_CLIP" => Some(MeshAdvBlendMethodEnum::AlphaClip),
            "AlphaBlend" => Some(MeshAdvBlendMethodEnum::AlphaBlend),
            "ALPHA_BLEND" => Some(MeshAdvBlendMethodEnum::AlphaBlend),
            "BLEND" => Some(MeshAdvBlendMethodEnum::AlphaBlend),
            _ => None,
        }
    }
}

impl MeshAdvBlendMethodEnum {
    pub fn schema_name() -> &'static str {
        "MeshAdvBlendMethod"
    }
}
#[derive(Copy, Clone)]
pub enum MeshAdvIndexTypeEnum {
    Uint16,
    Uint32,
}

impl Enum for MeshAdvIndexTypeEnum {
    fn to_symbol_name(&self) -> &'static str {
        match self {
            MeshAdvIndexTypeEnum::Uint16 => "Uint16",
            MeshAdvIndexTypeEnum::Uint32 => "Uint32",
        }
    }

    fn from_symbol_name(str: &str) -> Option<MeshAdvIndexTypeEnum> {
        match str {
            "Uint16" => Some(MeshAdvIndexTypeEnum::Uint16),
            "Uint32" => Some(MeshAdvIndexTypeEnum::Uint32),
            _ => None,
        }
    }
}

impl MeshAdvIndexTypeEnum {
    pub fn schema_name() -> &'static str {
        "MeshAdvIndexType"
    }
}
#[derive(Default)]
pub struct MeshAdvMaterialAssetAccessor(PropertyPath);

impl FieldAccessor for MeshAdvMaterialAssetAccessor {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMaterialAssetAccessor(property_path)
    }
}

impl RecordAccessor for MeshAdvMaterialAssetAccessor {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialAsset"
    }
}

impl MeshAdvMaterialAssetAccessor {
    pub fn alpha_threshold(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("alpha_threshold"))
    }

    pub fn backface_culling(&self) -> BooleanFieldAccessor {
        BooleanFieldAccessor::new(self.0.push("backface_culling"))
    }

    pub fn base_color_factor(&self) -> Vec4Accessor {
        Vec4Accessor::new(self.0.push("base_color_factor"))
    }

    pub fn blend_method(&self) -> EnumFieldAccessor::<MeshAdvBlendMethodEnum> {
        EnumFieldAccessor::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"))
    }

    pub fn color_texture(&self) -> AssetRefFieldAccessor {
        AssetRefFieldAccessor::new(self.0.push("color_texture"))
    }

    pub fn color_texture_has_alpha_channel(&self) -> BooleanFieldAccessor {
        BooleanFieldAccessor::new(self.0.push("color_texture_has_alpha_channel"))
    }

    pub fn emissive_factor(&self) -> Vec3Accessor {
        Vec3Accessor::new(self.0.push("emissive_factor"))
    }

    pub fn emissive_texture(&self) -> AssetRefFieldAccessor {
        AssetRefFieldAccessor::new(self.0.push("emissive_texture"))
    }

    pub fn metallic_factor(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("metallic_factor"))
    }

    pub fn metallic_roughness_texture(&self) -> AssetRefFieldAccessor {
        AssetRefFieldAccessor::new(self.0.push("metallic_roughness_texture"))
    }

    pub fn normal_texture(&self) -> AssetRefFieldAccessor {
        AssetRefFieldAccessor::new(self.0.push("normal_texture"))
    }

    pub fn normal_texture_scale(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("normal_texture_scale"))
    }

    pub fn roughness_factor(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("roughness_factor"))
    }

    pub fn shadow_method(&self) -> EnumFieldAccessor::<MeshAdvShadowMethodEnum> {
        EnumFieldAccessor::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"))
    }
}
pub struct MeshAdvMaterialAssetRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for MeshAdvMaterialAssetRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        MeshAdvMaterialAssetRef(property_path, data_container)
    }
}

impl<'a> RecordRef for MeshAdvMaterialAssetRef<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialAsset"
    }
}

impl<'a> MeshAdvMaterialAssetRef<'a> {
    pub fn alpha_threshold(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("alpha_threshold"), self.1.clone())
    }

    pub fn backface_culling(&self) -> BooleanFieldRef {
        BooleanFieldRef::new(self.0.push("backface_culling"), self.1.clone())
    }

    pub fn base_color_factor(&self) -> Vec4Ref {
        Vec4Ref::new(self.0.push("base_color_factor"), self.1.clone())
    }

    pub fn blend_method(&self) -> EnumFieldRef::<MeshAdvBlendMethodEnum> {
        EnumFieldRef::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"), self.1.clone())
    }

    pub fn color_texture(&self) -> AssetRefFieldRef {
        AssetRefFieldRef::new(self.0.push("color_texture"), self.1.clone())
    }

    pub fn color_texture_has_alpha_channel(&self) -> BooleanFieldRef {
        BooleanFieldRef::new(self.0.push("color_texture_has_alpha_channel"), self.1.clone())
    }

    pub fn emissive_factor(&self) -> Vec3Ref {
        Vec3Ref::new(self.0.push("emissive_factor"), self.1.clone())
    }

    pub fn emissive_texture(&self) -> AssetRefFieldRef {
        AssetRefFieldRef::new(self.0.push("emissive_texture"), self.1.clone())
    }

    pub fn metallic_factor(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("metallic_factor"), self.1.clone())
    }

    pub fn metallic_roughness_texture(&self) -> AssetRefFieldRef {
        AssetRefFieldRef::new(self.0.push("metallic_roughness_texture"), self.1.clone())
    }

    pub fn normal_texture(&self) -> AssetRefFieldRef {
        AssetRefFieldRef::new(self.0.push("normal_texture"), self.1.clone())
    }

    pub fn normal_texture_scale(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("normal_texture_scale"), self.1.clone())
    }

    pub fn roughness_factor(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("roughness_factor"), self.1.clone())
    }

    pub fn shadow_method(&self) -> EnumFieldRef::<MeshAdvShadowMethodEnum> {
        EnumFieldRef::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"), self.1.clone())
    }
}
pub struct MeshAdvMaterialAssetRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for MeshAdvMaterialAssetRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        MeshAdvMaterialAssetRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for MeshAdvMaterialAssetRefMut<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialAsset"
    }
}

impl<'a> MeshAdvMaterialAssetRefMut<'a> {
    pub fn alpha_threshold(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("alpha_threshold"), &self.1)
    }

    pub fn backface_culling(self: &'a Self) -> BooleanFieldRefMut {
        BooleanFieldRefMut::new(self.0.push("backface_culling"), &self.1)
    }

    pub fn base_color_factor(self: &'a Self) -> Vec4RefMut {
        Vec4RefMut::new(self.0.push("base_color_factor"), &self.1)
    }

    pub fn blend_method(self: &'a Self) -> EnumFieldRefMut::<MeshAdvBlendMethodEnum> {
        EnumFieldRefMut::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"), &self.1)
    }

    pub fn color_texture(self: &'a Self) -> AssetRefFieldRefMut {
        AssetRefFieldRefMut::new(self.0.push("color_texture"), &self.1)
    }

    pub fn color_texture_has_alpha_channel(self: &'a Self) -> BooleanFieldRefMut {
        BooleanFieldRefMut::new(self.0.push("color_texture_has_alpha_channel"), &self.1)
    }

    pub fn emissive_factor(self: &'a Self) -> Vec3RefMut {
        Vec3RefMut::new(self.0.push("emissive_factor"), &self.1)
    }

    pub fn emissive_texture(self: &'a Self) -> AssetRefFieldRefMut {
        AssetRefFieldRefMut::new(self.0.push("emissive_texture"), &self.1)
    }

    pub fn metallic_factor(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("metallic_factor"), &self.1)
    }

    pub fn metallic_roughness_texture(self: &'a Self) -> AssetRefFieldRefMut {
        AssetRefFieldRefMut::new(self.0.push("metallic_roughness_texture"), &self.1)
    }

    pub fn normal_texture(self: &'a Self) -> AssetRefFieldRefMut {
        AssetRefFieldRefMut::new(self.0.push("normal_texture"), &self.1)
    }

    pub fn normal_texture_scale(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("normal_texture_scale"), &self.1)
    }

    pub fn roughness_factor(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("roughness_factor"), &self.1)
    }

    pub fn shadow_method(self: &'a Self) -> EnumFieldRefMut::<MeshAdvShadowMethodEnum> {
        EnumFieldRefMut::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"), &self.1)
    }
}
pub struct MeshAdvMaterialAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for MeshAdvMaterialAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        MeshAdvMaterialAssetRecord(property_path, data_container.clone())
    }
}

impl Record for MeshAdvMaterialAssetRecord {
    type Reader<'a> = MeshAdvMaterialAssetRef<'a>;
    type Writer<'a> = MeshAdvMaterialAssetRefMut<'a>;
    type Accessor = MeshAdvMaterialAssetAccessor;

    fn schema_name() -> &'static str {
        "MeshAdvMaterialAsset"
    }
}

impl MeshAdvMaterialAssetRecord {
    pub fn alpha_threshold(self: &Self) -> F32Field {
        F32Field::new(self.0.push("alpha_threshold"), &self.1)
    }

    pub fn backface_culling(self: &Self) -> BooleanField {
        BooleanField::new(self.0.push("backface_culling"), &self.1)
    }

    pub fn base_color_factor(self: &Self) -> Vec4Record {
        Vec4Record::new(self.0.push("base_color_factor"), &self.1)
    }

    pub fn blend_method(self: &Self) -> EnumField::<MeshAdvBlendMethodEnum> {
        EnumField::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"), &self.1)
    }

    pub fn color_texture(self: &Self) -> AssetRefField {
        AssetRefField::new(self.0.push("color_texture"), &self.1)
    }

    pub fn color_texture_has_alpha_channel(self: &Self) -> BooleanField {
        BooleanField::new(self.0.push("color_texture_has_alpha_channel"), &self.1)
    }

    pub fn emissive_factor(self: &Self) -> Vec3Record {
        Vec3Record::new(self.0.push("emissive_factor"), &self.1)
    }

    pub fn emissive_texture(self: &Self) -> AssetRefField {
        AssetRefField::new(self.0.push("emissive_texture"), &self.1)
    }

    pub fn metallic_factor(self: &Self) -> F32Field {
        F32Field::new(self.0.push("metallic_factor"), &self.1)
    }

    pub fn metallic_roughness_texture(self: &Self) -> AssetRefField {
        AssetRefField::new(self.0.push("metallic_roughness_texture"), &self.1)
    }

    pub fn normal_texture(self: &Self) -> AssetRefField {
        AssetRefField::new(self.0.push("normal_texture"), &self.1)
    }

    pub fn normal_texture_scale(self: &Self) -> F32Field {
        F32Field::new(self.0.push("normal_texture_scale"), &self.1)
    }

    pub fn roughness_factor(self: &Self) -> F32Field {
        F32Field::new(self.0.push("roughness_factor"), &self.1)
    }

    pub fn shadow_method(self: &Self) -> EnumField::<MeshAdvShadowMethodEnum> {
        EnumField::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"), &self.1)
    }
}
#[derive(Default)]
pub struct MeshAdvMeshAssetAccessor(PropertyPath);

impl FieldAccessor for MeshAdvMeshAssetAccessor {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMeshAssetAccessor(property_path)
    }
}

impl RecordAccessor for MeshAdvMeshAssetAccessor {
    fn schema_name() -> &'static str {
        "MeshAdvMeshAsset"
    }
}

impl MeshAdvMeshAssetAccessor {
    pub fn material_slots(&self) -> DynamicArrayFieldAccessor::<AssetRefFieldAccessor> {
        DynamicArrayFieldAccessor::<AssetRefFieldAccessor>::new(self.0.push("material_slots"))
    }
}
pub struct MeshAdvMeshAssetRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for MeshAdvMeshAssetRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        MeshAdvMeshAssetRef(property_path, data_container)
    }
}

impl<'a> RecordRef for MeshAdvMeshAssetRef<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshAsset"
    }
}

impl<'a> MeshAdvMeshAssetRef<'a> {
    pub fn material_slots(&self) -> DynamicArrayFieldRef::<AssetRefFieldRef> {
        DynamicArrayFieldRef::<AssetRefFieldRef>::new(self.0.push("material_slots"), self.1.clone())
    }
}
pub struct MeshAdvMeshAssetRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for MeshAdvMeshAssetRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        MeshAdvMeshAssetRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for MeshAdvMeshAssetRefMut<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshAsset"
    }
}

impl<'a> MeshAdvMeshAssetRefMut<'a> {
    pub fn material_slots(self: &'a Self) -> DynamicArrayFieldRefMut::<AssetRefFieldRefMut> {
        DynamicArrayFieldRefMut::<AssetRefFieldRefMut>::new(self.0.push("material_slots"), &self.1)
    }
}
pub struct MeshAdvMeshAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for MeshAdvMeshAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        MeshAdvMeshAssetRecord(property_path, data_container.clone())
    }
}

impl Record for MeshAdvMeshAssetRecord {
    type Reader<'a> = MeshAdvMeshAssetRef<'a>;
    type Writer<'a> = MeshAdvMeshAssetRefMut<'a>;
    type Accessor = MeshAdvMeshAssetAccessor;

    fn schema_name() -> &'static str {
        "MeshAdvMeshAsset"
    }
}

impl MeshAdvMeshAssetRecord {
    pub fn material_slots(self: &Self) -> DynamicArrayField::<AssetRefField> {
        DynamicArrayField::<AssetRefField>::new(self.0.push("material_slots"), &self.1)
    }
}
#[derive(Default)]
pub struct MeshAdvMeshImportedDataAccessor(PropertyPath);

impl FieldAccessor for MeshAdvMeshImportedDataAccessor {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMeshImportedDataAccessor(property_path)
    }
}

impl RecordAccessor for MeshAdvMeshImportedDataAccessor {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedData"
    }
}

impl MeshAdvMeshImportedDataAccessor {
    pub fn mesh_parts(&self) -> DynamicArrayFieldAccessor::<MeshAdvMeshImportedDataMeshPartAccessor> {
        DynamicArrayFieldAccessor::<MeshAdvMeshImportedDataMeshPartAccessor>::new(self.0.push("mesh_parts"))
    }
}
pub struct MeshAdvMeshImportedDataRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for MeshAdvMeshImportedDataRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        MeshAdvMeshImportedDataRef(property_path, data_container)
    }
}

impl<'a> RecordRef for MeshAdvMeshImportedDataRef<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedData"
    }
}

impl<'a> MeshAdvMeshImportedDataRef<'a> {
    pub fn mesh_parts(&self) -> DynamicArrayFieldRef::<MeshAdvMeshImportedDataMeshPartRef> {
        DynamicArrayFieldRef::<MeshAdvMeshImportedDataMeshPartRef>::new(self.0.push("mesh_parts"), self.1.clone())
    }
}
pub struct MeshAdvMeshImportedDataRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for MeshAdvMeshImportedDataRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        MeshAdvMeshImportedDataRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for MeshAdvMeshImportedDataRefMut<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedData"
    }
}

impl<'a> MeshAdvMeshImportedDataRefMut<'a> {
    pub fn mesh_parts(self: &'a Self) -> DynamicArrayFieldRefMut::<MeshAdvMeshImportedDataMeshPartRefMut> {
        DynamicArrayFieldRefMut::<MeshAdvMeshImportedDataMeshPartRefMut>::new(self.0.push("mesh_parts"), &self.1)
    }
}
pub struct MeshAdvMeshImportedDataRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for MeshAdvMeshImportedDataRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        MeshAdvMeshImportedDataRecord(property_path, data_container.clone())
    }
}

impl Record for MeshAdvMeshImportedDataRecord {
    type Reader<'a> = MeshAdvMeshImportedDataRef<'a>;
    type Writer<'a> = MeshAdvMeshImportedDataRefMut<'a>;
    type Accessor = MeshAdvMeshImportedDataAccessor;

    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedData"
    }
}

impl MeshAdvMeshImportedDataRecord {
    pub fn mesh_parts(self: &Self) -> DynamicArrayField::<MeshAdvMeshImportedDataMeshPartRecord> {
        DynamicArrayField::<MeshAdvMeshImportedDataMeshPartRecord>::new(self.0.push("mesh_parts"), &self.1)
    }
}
#[derive(Default)]
pub struct MeshAdvMeshImportedDataMeshPartAccessor(PropertyPath);

impl FieldAccessor for MeshAdvMeshImportedDataMeshPartAccessor {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMeshImportedDataMeshPartAccessor(property_path)
    }
}

impl RecordAccessor for MeshAdvMeshImportedDataMeshPartAccessor {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedDataMeshPart"
    }
}

impl MeshAdvMeshImportedDataMeshPartAccessor {
    pub fn indices(&self) -> BytesFieldAccessor {
        BytesFieldAccessor::new(self.0.push("indices"))
    }

    pub fn material_index(&self) -> U32FieldAccessor {
        U32FieldAccessor::new(self.0.push("material_index"))
    }

    pub fn normals(&self) -> BytesFieldAccessor {
        BytesFieldAccessor::new(self.0.push("normals"))
    }

    pub fn positions(&self) -> BytesFieldAccessor {
        BytesFieldAccessor::new(self.0.push("positions"))
    }

    pub fn texture_coordinates(&self) -> BytesFieldAccessor {
        BytesFieldAccessor::new(self.0.push("texture_coordinates"))
    }
}
pub struct MeshAdvMeshImportedDataMeshPartRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for MeshAdvMeshImportedDataMeshPartRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        MeshAdvMeshImportedDataMeshPartRef(property_path, data_container)
    }
}

impl<'a> RecordRef for MeshAdvMeshImportedDataMeshPartRef<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedDataMeshPart"
    }
}

impl<'a> MeshAdvMeshImportedDataMeshPartRef<'a> {
    pub fn indices(&self) -> BytesFieldRef {
        BytesFieldRef::new(self.0.push("indices"), self.1.clone())
    }

    pub fn material_index(&self) -> U32FieldRef {
        U32FieldRef::new(self.0.push("material_index"), self.1.clone())
    }

    pub fn normals(&self) -> BytesFieldRef {
        BytesFieldRef::new(self.0.push("normals"), self.1.clone())
    }

    pub fn positions(&self) -> BytesFieldRef {
        BytesFieldRef::new(self.0.push("positions"), self.1.clone())
    }

    pub fn texture_coordinates(&self) -> BytesFieldRef {
        BytesFieldRef::new(self.0.push("texture_coordinates"), self.1.clone())
    }
}
pub struct MeshAdvMeshImportedDataMeshPartRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for MeshAdvMeshImportedDataMeshPartRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        MeshAdvMeshImportedDataMeshPartRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for MeshAdvMeshImportedDataMeshPartRefMut<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedDataMeshPart"
    }
}

impl<'a> MeshAdvMeshImportedDataMeshPartRefMut<'a> {
    pub fn indices(self: &'a Self) -> BytesFieldRefMut {
        BytesFieldRefMut::new(self.0.push("indices"), &self.1)
    }

    pub fn material_index(self: &'a Self) -> U32FieldRefMut {
        U32FieldRefMut::new(self.0.push("material_index"), &self.1)
    }

    pub fn normals(self: &'a Self) -> BytesFieldRefMut {
        BytesFieldRefMut::new(self.0.push("normals"), &self.1)
    }

    pub fn positions(self: &'a Self) -> BytesFieldRefMut {
        BytesFieldRefMut::new(self.0.push("positions"), &self.1)
    }

    pub fn texture_coordinates(self: &'a Self) -> BytesFieldRefMut {
        BytesFieldRefMut::new(self.0.push("texture_coordinates"), &self.1)
    }
}
pub struct MeshAdvMeshImportedDataMeshPartRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for MeshAdvMeshImportedDataMeshPartRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        MeshAdvMeshImportedDataMeshPartRecord(property_path, data_container.clone())
    }
}

impl Record for MeshAdvMeshImportedDataMeshPartRecord {
    type Reader<'a> = MeshAdvMeshImportedDataMeshPartRef<'a>;
    type Writer<'a> = MeshAdvMeshImportedDataMeshPartRefMut<'a>;
    type Accessor = MeshAdvMeshImportedDataMeshPartAccessor;

    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedDataMeshPart"
    }
}

impl MeshAdvMeshImportedDataMeshPartRecord {
    pub fn indices(self: &Self) -> BytesField {
        BytesField::new(self.0.push("indices"), &self.1)
    }

    pub fn material_index(self: &Self) -> U32Field {
        U32Field::new(self.0.push("material_index"), &self.1)
    }

    pub fn normals(self: &Self) -> BytesField {
        BytesField::new(self.0.push("normals"), &self.1)
    }

    pub fn positions(self: &Self) -> BytesField {
        BytesField::new(self.0.push("positions"), &self.1)
    }

    pub fn texture_coordinates(self: &Self) -> BytesField {
        BytesField::new(self.0.push("texture_coordinates"), &self.1)
    }
}
#[derive(Copy, Clone)]
pub enum MeshAdvShadowMethodEnum {
    None,
    Opaque,
}

impl Enum for MeshAdvShadowMethodEnum {
    fn to_symbol_name(&self) -> &'static str {
        match self {
            MeshAdvShadowMethodEnum::None => "None",
            MeshAdvShadowMethodEnum::Opaque => "Opaque",
        }
    }

    fn from_symbol_name(str: &str) -> Option<MeshAdvShadowMethodEnum> {
        match str {
            "None" => Some(MeshAdvShadowMethodEnum::None),
            "NONE" => Some(MeshAdvShadowMethodEnum::None),
            "Opaque" => Some(MeshAdvShadowMethodEnum::Opaque),
            "OPAQUE" => Some(MeshAdvShadowMethodEnum::Opaque),
            _ => None,
        }
    }
}

impl MeshAdvShadowMethodEnum {
    pub fn schema_name() -> &'static str {
        "MeshAdvShadowMethod"
    }
}
#[derive(Copy, Clone)]
pub enum TestEnumEnum {
    None,
    Opaque,
}

impl Enum for TestEnumEnum {
    fn to_symbol_name(&self) -> &'static str {
        match self {
            TestEnumEnum::None => "None",
            TestEnumEnum::Opaque => "Opaque",
        }
    }

    fn from_symbol_name(str: &str) -> Option<TestEnumEnum> {
        match str {
            "None" => Some(TestEnumEnum::None),
            "NONE" => Some(TestEnumEnum::None),
            "Opaque" => Some(TestEnumEnum::Opaque),
            "OPAQUE" => Some(TestEnumEnum::Opaque),
            _ => None,
        }
    }
}

impl TestEnumEnum {
    pub fn schema_name() -> &'static str {
        "TestEnum"
    }
}
#[derive(Default)]
pub struct TransformAccessor(PropertyPath);

impl FieldAccessor for TransformAccessor {
    fn new(property_path: PropertyPath) -> Self {
        TransformAccessor(property_path)
    }
}

impl RecordAccessor for TransformAccessor {
    fn schema_name() -> &'static str {
        "Transform"
    }
}

impl TransformAccessor {
    pub fn all_fields(&self) -> AllFieldsAccessor {
        AllFieldsAccessor::new(self.0.push("all_fields"))
    }

    pub fn position(&self) -> Vec3Accessor {
        Vec3Accessor::new(self.0.push("position"))
    }

    pub fn rotation(&self) -> Vec4Accessor {
        Vec4Accessor::new(self.0.push("rotation"))
    }

    pub fn scale(&self) -> Vec3Accessor {
        Vec3Accessor::new(self.0.push("scale"))
    }
}
pub struct TransformRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for TransformRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        TransformRef(property_path, data_container)
    }
}

impl<'a> RecordRef for TransformRef<'a> {
    fn schema_name() -> &'static str {
        "Transform"
    }
}

impl<'a> TransformRef<'a> {
    pub fn all_fields(&self) -> AllFieldsRef {
        AllFieldsRef::new(self.0.push("all_fields"), self.1.clone())
    }

    pub fn position(&self) -> Vec3Ref {
        Vec3Ref::new(self.0.push("position"), self.1.clone())
    }

    pub fn rotation(&self) -> Vec4Ref {
        Vec4Ref::new(self.0.push("rotation"), self.1.clone())
    }

    pub fn scale(&self) -> Vec3Ref {
        Vec3Ref::new(self.0.push("scale"), self.1.clone())
    }
}
pub struct TransformRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for TransformRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        TransformRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for TransformRefMut<'a> {
    fn schema_name() -> &'static str {
        "Transform"
    }
}

impl<'a> TransformRefMut<'a> {
    pub fn all_fields(self: &'a Self) -> AllFieldsRefMut {
        AllFieldsRefMut::new(self.0.push("all_fields"), &self.1)
    }

    pub fn position(self: &'a Self) -> Vec3RefMut {
        Vec3RefMut::new(self.0.push("position"), &self.1)
    }

    pub fn rotation(self: &'a Self) -> Vec4RefMut {
        Vec4RefMut::new(self.0.push("rotation"), &self.1)
    }

    pub fn scale(self: &'a Self) -> Vec3RefMut {
        Vec3RefMut::new(self.0.push("scale"), &self.1)
    }
}
pub struct TransformRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for TransformRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        TransformRecord(property_path, data_container.clone())
    }
}

impl Record for TransformRecord {
    type Reader<'a> = TransformRef<'a>;
    type Writer<'a> = TransformRefMut<'a>;
    type Accessor = TransformAccessor;

    fn schema_name() -> &'static str {
        "Transform"
    }
}

impl TransformRecord {
    pub fn all_fields(self: &Self) -> AllFieldsRecord {
        AllFieldsRecord::new(self.0.push("all_fields"), &self.1)
    }

    pub fn position(self: &Self) -> Vec3Record {
        Vec3Record::new(self.0.push("position"), &self.1)
    }

    pub fn rotation(self: &Self) -> Vec4Record {
        Vec4Record::new(self.0.push("rotation"), &self.1)
    }

    pub fn scale(self: &Self) -> Vec3Record {
        Vec3Record::new(self.0.push("scale"), &self.1)
    }
}
#[derive(Default)]
pub struct TransformRefAccessor(PropertyPath);

impl FieldAccessor for TransformRefAccessor {
    fn new(property_path: PropertyPath) -> Self {
        TransformRefAccessor(property_path)
    }
}

impl RecordAccessor for TransformRefAccessor {
    fn schema_name() -> &'static str {
        "TransformRef"
    }
}

impl TransformRefAccessor {
    pub fn transform(&self) -> AssetRefFieldAccessor {
        AssetRefFieldAccessor::new(self.0.push("transform"))
    }
}
pub struct TransformRefRef<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for TransformRefRef<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        TransformRefRef(property_path, data_container)
    }
}

impl<'a> RecordRef for TransformRefRef<'a> {
    fn schema_name() -> &'static str {
        "TransformRef"
    }
}

impl<'a> TransformRefRef<'a> {
    pub fn transform(&self) -> AssetRefFieldRef {
        AssetRefFieldRef::new(self.0.push("transform"), self.1.clone())
    }
}
pub struct TransformRefRefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for TransformRefRefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        TransformRefRefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for TransformRefRefMut<'a> {
    fn schema_name() -> &'static str {
        "TransformRef"
    }
}

impl<'a> TransformRefRefMut<'a> {
    pub fn transform(self: &'a Self) -> AssetRefFieldRefMut {
        AssetRefFieldRefMut::new(self.0.push("transform"), &self.1)
    }
}
pub struct TransformRefRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for TransformRefRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        TransformRefRecord(property_path, data_container.clone())
    }
}

impl Record for TransformRefRecord {
    type Reader<'a> = TransformRefRef<'a>;
    type Writer<'a> = TransformRefRefMut<'a>;
    type Accessor = TransformRefAccessor;

    fn schema_name() -> &'static str {
        "TransformRef"
    }
}

impl TransformRefRecord {
    pub fn transform(self: &Self) -> AssetRefField {
        AssetRefField::new(self.0.push("transform"), &self.1)
    }
}
#[derive(Default)]
pub struct Vec3Accessor(PropertyPath);

impl FieldAccessor for Vec3Accessor {
    fn new(property_path: PropertyPath) -> Self {
        Vec3Accessor(property_path)
    }
}

impl RecordAccessor for Vec3Accessor {
    fn schema_name() -> &'static str {
        "Vec3"
    }
}

impl Vec3Accessor {
    pub fn x(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("x"))
    }

    pub fn y(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("y"))
    }

    pub fn z(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("z"))
    }
}
pub struct Vec3Ref<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for Vec3Ref<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        Vec3Ref(property_path, data_container)
    }
}

impl<'a> RecordRef for Vec3Ref<'a> {
    fn schema_name() -> &'static str {
        "Vec3"
    }
}

impl<'a> Vec3Ref<'a> {
    pub fn x(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("x"), self.1.clone())
    }

    pub fn y(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("y"), self.1.clone())
    }

    pub fn z(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("z"), self.1.clone())
    }
}
pub struct Vec3RefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for Vec3RefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        Vec3RefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for Vec3RefMut<'a> {
    fn schema_name() -> &'static str {
        "Vec3"
    }
}

impl<'a> Vec3RefMut<'a> {
    pub fn x(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("x"), &self.1)
    }

    pub fn y(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("y"), &self.1)
    }

    pub fn z(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("z"), &self.1)
    }
}
pub struct Vec3Record(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for Vec3Record {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        Vec3Record(property_path, data_container.clone())
    }
}

impl Record for Vec3Record {
    type Reader<'a> = Vec3Ref<'a>;
    type Writer<'a> = Vec3RefMut<'a>;
    type Accessor = Vec3Accessor;

    fn schema_name() -> &'static str {
        "Vec3"
    }
}

impl Vec3Record {
    pub fn x(self: &Self) -> F32Field {
        F32Field::new(self.0.push("x"), &self.1)
    }

    pub fn y(self: &Self) -> F32Field {
        F32Field::new(self.0.push("y"), &self.1)
    }

    pub fn z(self: &Self) -> F32Field {
        F32Field::new(self.0.push("z"), &self.1)
    }
}
#[derive(Default)]
pub struct Vec4Accessor(PropertyPath);

impl FieldAccessor for Vec4Accessor {
    fn new(property_path: PropertyPath) -> Self {
        Vec4Accessor(property_path)
    }
}

impl RecordAccessor for Vec4Accessor {
    fn schema_name() -> &'static str {
        "Vec4"
    }
}

impl Vec4Accessor {
    pub fn w(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("w"))
    }

    pub fn x(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("x"))
    }

    pub fn y(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("y"))
    }

    pub fn z(&self) -> F32FieldAccessor {
        F32FieldAccessor::new(self.0.push("z"))
    }
}
pub struct Vec4Ref<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldRef<'a> for Vec4Ref<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
        Vec4Ref(property_path, data_container)
    }
}

impl<'a> RecordRef for Vec4Ref<'a> {
    fn schema_name() -> &'static str {
        "Vec4"
    }
}

impl<'a> Vec4Ref<'a> {
    pub fn w(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("w"), self.1.clone())
    }

    pub fn x(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("x"), self.1.clone())
    }

    pub fn y(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("y"), self.1.clone())
    }

    pub fn z(&self) -> F32FieldRef {
        F32FieldRef::new(self.0.push("z"), self.1.clone())
    }
}
pub struct Vec4RefMut<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldRefMut<'a> for Vec4RefMut<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
        Vec4RefMut(property_path, data_container.clone())
    }
}

impl<'a> RecordRefMut for Vec4RefMut<'a> {
    fn schema_name() -> &'static str {
        "Vec4"
    }
}

impl<'a> Vec4RefMut<'a> {
    pub fn w(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("w"), &self.1)
    }

    pub fn x(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("x"), &self.1)
    }

    pub fn y(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("y"), &self.1)
    }

    pub fn z(self: &'a Self) -> F32FieldRefMut {
        F32FieldRefMut::new(self.0.push("z"), &self.1)
    }
}
pub struct Vec4Record(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for Vec4Record {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        Vec4Record(property_path, data_container.clone())
    }
}

impl Record for Vec4Record {
    type Reader<'a> = Vec4Ref<'a>;
    type Writer<'a> = Vec4RefMut<'a>;
    type Accessor = Vec4Accessor;

    fn schema_name() -> &'static str {
        "Vec4"
    }
}

impl Vec4Record {
    pub fn w(self: &Self) -> F32Field {
        F32Field::new(self.0.push("w"), &self.1)
    }

    pub fn x(self: &Self) -> F32Field {
        F32Field::new(self.0.push("x"), &self.1)
    }

    pub fn y(self: &Self) -> F32Field {
        F32Field::new(self.0.push("y"), &self.1)
    }

    pub fn z(self: &Self) -> F32Field {
        F32Field::new(self.0.push("z"), &self.1)
    }
}
