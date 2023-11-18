// This file generated automatically by hydrate-codegen. Do not make manual edits. Use include!() to place these types in the intended location.
#[derive(Default)]
pub struct AllFieldsRecord(PropertyPath);

impl Field for AllFieldsRecord {
    fn new(property_path: PropertyPath) -> Self {
        AllFieldsRecord(property_path)
    }
}

impl Record for AllFieldsRecord {
    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl AllFieldsRecord {
    pub fn boolean(&self) -> BooleanField {
        BooleanField::new(self.0.push("boolean"))
    }

    pub fn dynamic_array_i32(&self) -> DynamicArrayField::<I32Field> {
        DynamicArrayField::<I32Field>::new(self.0.push("dynamic_array_i32"))
    }

    pub fn dynamic_array_vec3(&self) -> DynamicArrayField::<Vec3Record> {
        DynamicArrayField::<Vec3Record>::new(self.0.push("dynamic_array_vec3"))
    }

    pub fn f32(&self) -> F32Field {
        F32Field::new(self.0.push("f32"))
    }

    pub fn f64(&self) -> F64Field {
        F64Field::new(self.0.push("f64"))
    }

    pub fn i32(&self) -> I32Field {
        I32Field::new(self.0.push("i32"))
    }

    pub fn i64(&self) -> I64Field {
        I64Field::new(self.0.push("i64"))
    }

    pub fn nullable_bool(&self) -> NullableField::<BooleanField> {
        NullableField::<BooleanField>::new(self.0.push("nullable_bool"))
    }

    pub fn nullable_vec3(&self) -> NullableField::<Vec3Record> {
        NullableField::<Vec3Record>::new(self.0.push("nullable_vec3"))
    }

    pub fn reference(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("reference"))
    }

    pub fn string(&self) -> StringField {
        StringField::new(self.0.push("string"))
    }

    pub fn u32(&self) -> U32Field {
        U32Field::new(self.0.push("u32"))
    }

    pub fn u64(&self) -> U64Field {
        U64Field::new(self.0.push("u64"))
    }
}
pub struct AllFieldsReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for AllFieldsReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        AllFieldsReader(property_path, data_container)
    }
}

impl<'a> RecordReader for AllFieldsReader<'a> {
    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl<'a> AllFieldsReader<'a> {
    pub fn boolean(&self) -> BooleanFieldReader {
        BooleanFieldReader::new(self.0.push("boolean"), self.1)
    }

    pub fn dynamic_array_i32(&self) -> DynamicArrayFieldReader::<I32FieldReader> {
        DynamicArrayFieldReader::<I32FieldReader>::new(self.0.push("dynamic_array_i32"), self.1)
    }

    pub fn dynamic_array_vec3(&self) -> DynamicArrayFieldReader::<Vec3Reader> {
        DynamicArrayFieldReader::<Vec3Reader>::new(self.0.push("dynamic_array_vec3"), self.1)
    }

    pub fn f32(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("f32"), self.1)
    }

    pub fn f64(&self) -> F64FieldReader {
        F64FieldReader::new(self.0.push("f64"), self.1)
    }

    pub fn i32(&self) -> I32FieldReader {
        I32FieldReader::new(self.0.push("i32"), self.1)
    }

    pub fn i64(&self) -> I64FieldReader {
        I64FieldReader::new(self.0.push("i64"), self.1)
    }

    pub fn nullable_bool(&self) -> NullableFieldReader::<BooleanFieldReader> {
        NullableFieldReader::<BooleanFieldReader>::new(self.0.push("nullable_bool"), self.1)
    }

    pub fn nullable_vec3(&self) -> NullableFieldReader::<Vec3Reader> {
        NullableFieldReader::<Vec3Reader>::new(self.0.push("nullable_vec3"), self.1)
    }

    pub fn reference(&self) -> AssetRefFieldReader {
        AssetRefFieldReader::new(self.0.push("reference"), self.1)
    }

    pub fn string(&self) -> StringFieldReader {
        StringFieldReader::new(self.0.push("string"), self.1)
    }

    pub fn u32(&self) -> U32FieldReader {
        U32FieldReader::new(self.0.push("u32"), self.1)
    }

    pub fn u64(&self) -> U64FieldReader {
        U64FieldReader::new(self.0.push("u64"), self.1)
    }
}
pub struct AllFieldsWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for AllFieldsWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        AllFieldsWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for AllFieldsWriter<'a> {
    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl<'a> AllFieldsWriter<'a> {
    pub fn boolean(self: &'a Self) -> BooleanFieldWriter {
        BooleanFieldWriter::new(self.0.push("boolean"), &self.1)
    }

    pub fn dynamic_array_i32(self: &'a Self) -> DynamicArrayFieldWriter::<I32FieldWriter> {
        DynamicArrayFieldWriter::<I32FieldWriter>::new(self.0.push("dynamic_array_i32"), &self.1)
    }

    pub fn dynamic_array_vec3(self: &'a Self) -> DynamicArrayFieldWriter::<Vec3Writer> {
        DynamicArrayFieldWriter::<Vec3Writer>::new(self.0.push("dynamic_array_vec3"), &self.1)
    }

    pub fn f32(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("f32"), &self.1)
    }

    pub fn f64(self: &'a Self) -> F64FieldWriter {
        F64FieldWriter::new(self.0.push("f64"), &self.1)
    }

    pub fn i32(self: &'a Self) -> I32FieldWriter {
        I32FieldWriter::new(self.0.push("i32"), &self.1)
    }

    pub fn i64(self: &'a Self) -> I64FieldWriter {
        I64FieldWriter::new(self.0.push("i64"), &self.1)
    }

    pub fn nullable_bool(self: &'a Self) -> NullableFieldWriter::<BooleanFieldWriter> {
        NullableFieldWriter::<BooleanFieldWriter>::new(self.0.push("nullable_bool"), &self.1)
    }

    pub fn nullable_vec3(self: &'a Self) -> NullableFieldWriter::<Vec3Writer> {
        NullableFieldWriter::<Vec3Writer>::new(self.0.push("nullable_vec3"), &self.1)
    }

    pub fn reference(self: &'a Self) -> AssetRefFieldWriter {
        AssetRefFieldWriter::new(self.0.push("reference"), &self.1)
    }

    pub fn string(self: &'a Self) -> StringFieldWriter {
        StringFieldWriter::new(self.0.push("string"), &self.1)
    }

    pub fn u32(self: &'a Self) -> U32FieldWriter {
        U32FieldWriter::new(self.0.push("u32"), &self.1)
    }

    pub fn u64(self: &'a Self) -> U64FieldWriter {
        U64FieldWriter::new(self.0.push("u64"), &self.1)
    }
}
pub struct AllFieldsOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for AllFieldsOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        AllFieldsOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for AllFieldsOwned {
    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl AllFieldsOwned {
    pub fn boolean(self: &Self) -> BooleanFieldOwned {
        BooleanFieldOwned::new(self.0.push("boolean"), &self.1)
    }

    pub fn dynamic_array_i32(self: &Self) -> DynamicArrayFieldOwned::<I32FieldOwned> {
        DynamicArrayFieldOwned::<I32FieldOwned>::new(self.0.push("dynamic_array_i32"), &self.1)
    }

    pub fn dynamic_array_vec3(self: &Self) -> DynamicArrayFieldOwned::<Vec3Owned> {
        DynamicArrayFieldOwned::<Vec3Owned>::new(self.0.push("dynamic_array_vec3"), &self.1)
    }

    pub fn f32(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("f32"), &self.1)
    }

    pub fn f64(self: &Self) -> F64FieldOwned {
        F64FieldOwned::new(self.0.push("f64"), &self.1)
    }

    pub fn i32(self: &Self) -> I32FieldOwned {
        I32FieldOwned::new(self.0.push("i32"), &self.1)
    }

    pub fn i64(self: &Self) -> I64FieldOwned {
        I64FieldOwned::new(self.0.push("i64"), &self.1)
    }

    pub fn nullable_bool(self: &Self) -> NullableFieldOwned::<BooleanFieldOwned> {
        NullableFieldOwned::<BooleanFieldOwned>::new(self.0.push("nullable_bool"), &self.1)
    }

    pub fn nullable_vec3(self: &Self) -> NullableFieldOwned::<Vec3Owned> {
        NullableFieldOwned::<Vec3Owned>::new(self.0.push("nullable_vec3"), &self.1)
    }

    pub fn reference(self: &Self) -> AssetRefFieldOwned {
        AssetRefFieldOwned::new(self.0.push("reference"), &self.1)
    }

    pub fn string(self: &Self) -> StringFieldOwned {
        StringFieldOwned::new(self.0.push("string"), &self.1)
    }

    pub fn u32(self: &Self) -> U32FieldOwned {
        U32FieldOwned::new(self.0.push("u32"), &self.1)
    }

    pub fn u64(self: &Self) -> U64FieldOwned {
        U64FieldOwned::new(self.0.push("u64"), &self.1)
    }
}
#[derive(Default)]
pub struct GlslBuildTargetAssetRecord(PropertyPath);

impl Field for GlslBuildTargetAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        GlslBuildTargetAssetRecord(property_path)
    }
}

impl Record for GlslBuildTargetAssetRecord {
    fn schema_name() -> &'static str {
        "GlslBuildTargetAsset"
    }
}

impl GlslBuildTargetAssetRecord {
    pub fn entry_point(&self) -> StringField {
        StringField::new(self.0.push("entry_point"))
    }

    pub fn source_file(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("source_file"))
    }
}
pub struct GlslBuildTargetAssetReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for GlslBuildTargetAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        GlslBuildTargetAssetReader(property_path, data_container)
    }
}

impl<'a> RecordReader for GlslBuildTargetAssetReader<'a> {
    fn schema_name() -> &'static str {
        "GlslBuildTargetAsset"
    }
}

impl<'a> GlslBuildTargetAssetReader<'a> {
    pub fn entry_point(&self) -> StringFieldReader {
        StringFieldReader::new(self.0.push("entry_point"), self.1)
    }

    pub fn source_file(&self) -> AssetRefFieldReader {
        AssetRefFieldReader::new(self.0.push("source_file"), self.1)
    }
}
pub struct GlslBuildTargetAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for GlslBuildTargetAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        GlslBuildTargetAssetWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for GlslBuildTargetAssetWriter<'a> {
    fn schema_name() -> &'static str {
        "GlslBuildTargetAsset"
    }
}

impl<'a> GlslBuildTargetAssetWriter<'a> {
    pub fn entry_point(self: &'a Self) -> StringFieldWriter {
        StringFieldWriter::new(self.0.push("entry_point"), &self.1)
    }

    pub fn source_file(self: &'a Self) -> AssetRefFieldWriter {
        AssetRefFieldWriter::new(self.0.push("source_file"), &self.1)
    }
}
pub struct GlslBuildTargetAssetOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for GlslBuildTargetAssetOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        GlslBuildTargetAssetOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for GlslBuildTargetAssetOwned {
    fn schema_name() -> &'static str {
        "GlslBuildTargetAsset"
    }
}

impl GlslBuildTargetAssetOwned {
    pub fn entry_point(self: &Self) -> StringFieldOwned {
        StringFieldOwned::new(self.0.push("entry_point"), &self.1)
    }

    pub fn source_file(self: &Self) -> AssetRefFieldOwned {
        AssetRefFieldOwned::new(self.0.push("source_file"), &self.1)
    }
}
#[derive(Default)]
pub struct GlslSourceFileAssetRecord(PropertyPath);

impl Field for GlslSourceFileAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        GlslSourceFileAssetRecord(property_path)
    }
}

impl Record for GlslSourceFileAssetRecord {
    fn schema_name() -> &'static str {
        "GlslSourceFileAsset"
    }
}

impl GlslSourceFileAssetRecord {
}
pub struct GlslSourceFileAssetReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for GlslSourceFileAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        GlslSourceFileAssetReader(property_path, data_container)
    }
}

impl<'a> RecordReader for GlslSourceFileAssetReader<'a> {
    fn schema_name() -> &'static str {
        "GlslSourceFileAsset"
    }
}

impl<'a> GlslSourceFileAssetReader<'a> {
}
pub struct GlslSourceFileAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for GlslSourceFileAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        GlslSourceFileAssetWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for GlslSourceFileAssetWriter<'a> {
    fn schema_name() -> &'static str {
        "GlslSourceFileAsset"
    }
}

impl<'a> GlslSourceFileAssetWriter<'a> {
}
pub struct GlslSourceFileAssetOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for GlslSourceFileAssetOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        GlslSourceFileAssetOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for GlslSourceFileAssetOwned {
    fn schema_name() -> &'static str {
        "GlslSourceFileAsset"
    }
}

impl GlslSourceFileAssetOwned {
}
#[derive(Default)]
pub struct GlslSourceFileImportedDataRecord(PropertyPath);

impl Field for GlslSourceFileImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        GlslSourceFileImportedDataRecord(property_path)
    }
}

impl Record for GlslSourceFileImportedDataRecord {
    fn schema_name() -> &'static str {
        "GlslSourceFileImportedData"
    }
}

impl GlslSourceFileImportedDataRecord {
    pub fn code(&self) -> StringField {
        StringField::new(self.0.push("code"))
    }
}
pub struct GlslSourceFileImportedDataReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for GlslSourceFileImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        GlslSourceFileImportedDataReader(property_path, data_container)
    }
}

impl<'a> RecordReader for GlslSourceFileImportedDataReader<'a> {
    fn schema_name() -> &'static str {
        "GlslSourceFileImportedData"
    }
}

impl<'a> GlslSourceFileImportedDataReader<'a> {
    pub fn code(&self) -> StringFieldReader {
        StringFieldReader::new(self.0.push("code"), self.1)
    }
}
pub struct GlslSourceFileImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for GlslSourceFileImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        GlslSourceFileImportedDataWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for GlslSourceFileImportedDataWriter<'a> {
    fn schema_name() -> &'static str {
        "GlslSourceFileImportedData"
    }
}

impl<'a> GlslSourceFileImportedDataWriter<'a> {
    pub fn code(self: &'a Self) -> StringFieldWriter {
        StringFieldWriter::new(self.0.push("code"), &self.1)
    }
}
pub struct GlslSourceFileImportedDataOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for GlslSourceFileImportedDataOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        GlslSourceFileImportedDataOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for GlslSourceFileImportedDataOwned {
    fn schema_name() -> &'static str {
        "GlslSourceFileImportedData"
    }
}

impl GlslSourceFileImportedDataOwned {
    pub fn code(self: &Self) -> StringFieldOwned {
        StringFieldOwned::new(self.0.push("code"), &self.1)
    }
}
#[derive(Default)]
pub struct GpuBufferAssetRecord(PropertyPath);

impl Field for GpuBufferAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        GpuBufferAssetRecord(property_path)
    }
}

impl Record for GpuBufferAssetRecord {
    fn schema_name() -> &'static str {
        "GpuBufferAsset"
    }
}

impl GpuBufferAssetRecord {
}
pub struct GpuBufferAssetReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for GpuBufferAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        GpuBufferAssetReader(property_path, data_container)
    }
}

impl<'a> RecordReader for GpuBufferAssetReader<'a> {
    fn schema_name() -> &'static str {
        "GpuBufferAsset"
    }
}

impl<'a> GpuBufferAssetReader<'a> {
}
pub struct GpuBufferAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for GpuBufferAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        GpuBufferAssetWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for GpuBufferAssetWriter<'a> {
    fn schema_name() -> &'static str {
        "GpuBufferAsset"
    }
}

impl<'a> GpuBufferAssetWriter<'a> {
}
pub struct GpuBufferAssetOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for GpuBufferAssetOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        GpuBufferAssetOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for GpuBufferAssetOwned {
    fn schema_name() -> &'static str {
        "GpuBufferAsset"
    }
}

impl GpuBufferAssetOwned {
}
#[derive(Default)]
pub struct GpuBufferImportedDataRecord(PropertyPath);

impl Field for GpuBufferImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        GpuBufferImportedDataRecord(property_path)
    }
}

impl Record for GpuBufferImportedDataRecord {
    fn schema_name() -> &'static str {
        "GpuBufferImportedData"
    }
}

impl GpuBufferImportedDataRecord {
    pub fn alignment(&self) -> U32Field {
        U32Field::new(self.0.push("alignment"))
    }

    pub fn data(&self) -> BytesField {
        BytesField::new(self.0.push("data"))
    }

    pub fn resource_type(&self) -> U32Field {
        U32Field::new(self.0.push("resource_type"))
    }
}
pub struct GpuBufferImportedDataReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for GpuBufferImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        GpuBufferImportedDataReader(property_path, data_container)
    }
}

impl<'a> RecordReader for GpuBufferImportedDataReader<'a> {
    fn schema_name() -> &'static str {
        "GpuBufferImportedData"
    }
}

impl<'a> GpuBufferImportedDataReader<'a> {
    pub fn alignment(&self) -> U32FieldReader {
        U32FieldReader::new(self.0.push("alignment"), self.1)
    }

    pub fn data(&self) -> BytesFieldReader {
        BytesFieldReader::new(self.0.push("data"), self.1)
    }

    pub fn resource_type(&self) -> U32FieldReader {
        U32FieldReader::new(self.0.push("resource_type"), self.1)
    }
}
pub struct GpuBufferImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for GpuBufferImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        GpuBufferImportedDataWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for GpuBufferImportedDataWriter<'a> {
    fn schema_name() -> &'static str {
        "GpuBufferImportedData"
    }
}

impl<'a> GpuBufferImportedDataWriter<'a> {
    pub fn alignment(self: &'a Self) -> U32FieldWriter {
        U32FieldWriter::new(self.0.push("alignment"), &self.1)
    }

    pub fn data(self: &'a Self) -> BytesFieldWriter {
        BytesFieldWriter::new(self.0.push("data"), &self.1)
    }

    pub fn resource_type(self: &'a Self) -> U32FieldWriter {
        U32FieldWriter::new(self.0.push("resource_type"), &self.1)
    }
}
pub struct GpuBufferImportedDataOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for GpuBufferImportedDataOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        GpuBufferImportedDataOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for GpuBufferImportedDataOwned {
    fn schema_name() -> &'static str {
        "GpuBufferImportedData"
    }
}

impl GpuBufferImportedDataOwned {
    pub fn alignment(self: &Self) -> U32FieldOwned {
        U32FieldOwned::new(self.0.push("alignment"), &self.1)
    }

    pub fn data(self: &Self) -> BytesFieldOwned {
        BytesFieldOwned::new(self.0.push("data"), &self.1)
    }

    pub fn resource_type(self: &Self) -> U32FieldOwned {
        U32FieldOwned::new(self.0.push("resource_type"), &self.1)
    }
}
#[derive(Default)]
pub struct GpuImageAssetRecord(PropertyPath);

impl Field for GpuImageAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        GpuImageAssetRecord(property_path)
    }
}

impl Record for GpuImageAssetRecord {
    fn schema_name() -> &'static str {
        "GpuImageAsset"
    }
}

impl GpuImageAssetRecord {
    pub fn compress(&self) -> BooleanField {
        BooleanField::new(self.0.push("compress"))
    }
}
pub struct GpuImageAssetReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for GpuImageAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        GpuImageAssetReader(property_path, data_container)
    }
}

impl<'a> RecordReader for GpuImageAssetReader<'a> {
    fn schema_name() -> &'static str {
        "GpuImageAsset"
    }
}

impl<'a> GpuImageAssetReader<'a> {
    pub fn compress(&self) -> BooleanFieldReader {
        BooleanFieldReader::new(self.0.push("compress"), self.1)
    }
}
pub struct GpuImageAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for GpuImageAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        GpuImageAssetWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for GpuImageAssetWriter<'a> {
    fn schema_name() -> &'static str {
        "GpuImageAsset"
    }
}

impl<'a> GpuImageAssetWriter<'a> {
    pub fn compress(self: &'a Self) -> BooleanFieldWriter {
        BooleanFieldWriter::new(self.0.push("compress"), &self.1)
    }
}
pub struct GpuImageAssetOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for GpuImageAssetOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        GpuImageAssetOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for GpuImageAssetOwned {
    fn schema_name() -> &'static str {
        "GpuImageAsset"
    }
}

impl GpuImageAssetOwned {
    pub fn compress(self: &Self) -> BooleanFieldOwned {
        BooleanFieldOwned::new(self.0.push("compress"), &self.1)
    }
}
#[derive(Default)]
pub struct GpuImageImportedDataRecord(PropertyPath);

impl Field for GpuImageImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        GpuImageImportedDataRecord(property_path)
    }
}

impl Record for GpuImageImportedDataRecord {
    fn schema_name() -> &'static str {
        "GpuImageImportedData"
    }
}

impl GpuImageImportedDataRecord {
    pub fn height(&self) -> U32Field {
        U32Field::new(self.0.push("height"))
    }

    pub fn image_bytes(&self) -> BytesField {
        BytesField::new(self.0.push("image_bytes"))
    }

    pub fn width(&self) -> U32Field {
        U32Field::new(self.0.push("width"))
    }
}
pub struct GpuImageImportedDataReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for GpuImageImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        GpuImageImportedDataReader(property_path, data_container)
    }
}

impl<'a> RecordReader for GpuImageImportedDataReader<'a> {
    fn schema_name() -> &'static str {
        "GpuImageImportedData"
    }
}

impl<'a> GpuImageImportedDataReader<'a> {
    pub fn height(&self) -> U32FieldReader {
        U32FieldReader::new(self.0.push("height"), self.1)
    }

    pub fn image_bytes(&self) -> BytesFieldReader {
        BytesFieldReader::new(self.0.push("image_bytes"), self.1)
    }

    pub fn width(&self) -> U32FieldReader {
        U32FieldReader::new(self.0.push("width"), self.1)
    }
}
pub struct GpuImageImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for GpuImageImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        GpuImageImportedDataWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for GpuImageImportedDataWriter<'a> {
    fn schema_name() -> &'static str {
        "GpuImageImportedData"
    }
}

impl<'a> GpuImageImportedDataWriter<'a> {
    pub fn height(self: &'a Self) -> U32FieldWriter {
        U32FieldWriter::new(self.0.push("height"), &self.1)
    }

    pub fn image_bytes(self: &'a Self) -> BytesFieldWriter {
        BytesFieldWriter::new(self.0.push("image_bytes"), &self.1)
    }

    pub fn width(self: &'a Self) -> U32FieldWriter {
        U32FieldWriter::new(self.0.push("width"), &self.1)
    }
}
pub struct GpuImageImportedDataOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for GpuImageImportedDataOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        GpuImageImportedDataOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for GpuImageImportedDataOwned {
    fn schema_name() -> &'static str {
        "GpuImageImportedData"
    }
}

impl GpuImageImportedDataOwned {
    pub fn height(self: &Self) -> U32FieldOwned {
        U32FieldOwned::new(self.0.push("height"), &self.1)
    }

    pub fn image_bytes(self: &Self) -> BytesFieldOwned {
        BytesFieldOwned::new(self.0.push("image_bytes"), &self.1)
    }

    pub fn width(self: &Self) -> U32FieldOwned {
        U32FieldOwned::new(self.0.push("width"), &self.1)
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
pub struct MeshAdvMaterialAssetRecord(PropertyPath);

impl Field for MeshAdvMaterialAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMaterialAssetRecord(property_path)
    }
}

impl Record for MeshAdvMaterialAssetRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialAsset"
    }
}

impl MeshAdvMaterialAssetRecord {
    pub fn alpha_threshold(&self) -> F32Field {
        F32Field::new(self.0.push("alpha_threshold"))
    }

    pub fn backface_culling(&self) -> BooleanField {
        BooleanField::new(self.0.push("backface_culling"))
    }

    pub fn base_color_factor(&self) -> Vec4Record {
        Vec4Record::new(self.0.push("base_color_factor"))
    }

    pub fn blend_method(&self) -> EnumField::<MeshAdvBlendMethodEnum> {
        EnumField::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"))
    }

    pub fn color_texture(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("color_texture"))
    }

    pub fn color_texture_has_alpha_channel(&self) -> BooleanField {
        BooleanField::new(self.0.push("color_texture_has_alpha_channel"))
    }

    pub fn emissive_factor(&self) -> Vec3Record {
        Vec3Record::new(self.0.push("emissive_factor"))
    }

    pub fn emissive_texture(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("emissive_texture"))
    }

    pub fn metallic_factor(&self) -> F32Field {
        F32Field::new(self.0.push("metallic_factor"))
    }

    pub fn metallic_roughness_texture(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("metallic_roughness_texture"))
    }

    pub fn normal_texture(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("normal_texture"))
    }

    pub fn normal_texture_scale(&self) -> F32Field {
        F32Field::new(self.0.push("normal_texture_scale"))
    }

    pub fn roughness_factor(&self) -> F32Field {
        F32Field::new(self.0.push("roughness_factor"))
    }

    pub fn shadow_method(&self) -> EnumField::<MeshAdvShadowMethodEnum> {
        EnumField::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"))
    }
}
pub struct MeshAdvMaterialAssetReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for MeshAdvMaterialAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        MeshAdvMaterialAssetReader(property_path, data_container)
    }
}

impl<'a> RecordReader for MeshAdvMaterialAssetReader<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialAsset"
    }
}

impl<'a> MeshAdvMaterialAssetReader<'a> {
    pub fn alpha_threshold(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("alpha_threshold"), self.1)
    }

    pub fn backface_culling(&self) -> BooleanFieldReader {
        BooleanFieldReader::new(self.0.push("backface_culling"), self.1)
    }

    pub fn base_color_factor(&self) -> Vec4Reader {
        Vec4Reader::new(self.0.push("base_color_factor"), self.1)
    }

    pub fn blend_method(&self) -> EnumFieldReader::<MeshAdvBlendMethodEnum> {
        EnumFieldReader::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"), self.1)
    }

    pub fn color_texture(&self) -> AssetRefFieldReader {
        AssetRefFieldReader::new(self.0.push("color_texture"), self.1)
    }

    pub fn color_texture_has_alpha_channel(&self) -> BooleanFieldReader {
        BooleanFieldReader::new(self.0.push("color_texture_has_alpha_channel"), self.1)
    }

    pub fn emissive_factor(&self) -> Vec3Reader {
        Vec3Reader::new(self.0.push("emissive_factor"), self.1)
    }

    pub fn emissive_texture(&self) -> AssetRefFieldReader {
        AssetRefFieldReader::new(self.0.push("emissive_texture"), self.1)
    }

    pub fn metallic_factor(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("metallic_factor"), self.1)
    }

    pub fn metallic_roughness_texture(&self) -> AssetRefFieldReader {
        AssetRefFieldReader::new(self.0.push("metallic_roughness_texture"), self.1)
    }

    pub fn normal_texture(&self) -> AssetRefFieldReader {
        AssetRefFieldReader::new(self.0.push("normal_texture"), self.1)
    }

    pub fn normal_texture_scale(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("normal_texture_scale"), self.1)
    }

    pub fn roughness_factor(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("roughness_factor"), self.1)
    }

    pub fn shadow_method(&self) -> EnumFieldReader::<MeshAdvShadowMethodEnum> {
        EnumFieldReader::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"), self.1)
    }
}
pub struct MeshAdvMaterialAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMaterialAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        MeshAdvMaterialAssetWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for MeshAdvMaterialAssetWriter<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialAsset"
    }
}

impl<'a> MeshAdvMaterialAssetWriter<'a> {
    pub fn alpha_threshold(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("alpha_threshold"), &self.1)
    }

    pub fn backface_culling(self: &'a Self) -> BooleanFieldWriter {
        BooleanFieldWriter::new(self.0.push("backface_culling"), &self.1)
    }

    pub fn base_color_factor(self: &'a Self) -> Vec4Writer {
        Vec4Writer::new(self.0.push("base_color_factor"), &self.1)
    }

    pub fn blend_method(self: &'a Self) -> EnumFieldWriter::<MeshAdvBlendMethodEnum> {
        EnumFieldWriter::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"), &self.1)
    }

    pub fn color_texture(self: &'a Self) -> AssetRefFieldWriter {
        AssetRefFieldWriter::new(self.0.push("color_texture"), &self.1)
    }

    pub fn color_texture_has_alpha_channel(self: &'a Self) -> BooleanFieldWriter {
        BooleanFieldWriter::new(self.0.push("color_texture_has_alpha_channel"), &self.1)
    }

    pub fn emissive_factor(self: &'a Self) -> Vec3Writer {
        Vec3Writer::new(self.0.push("emissive_factor"), &self.1)
    }

    pub fn emissive_texture(self: &'a Self) -> AssetRefFieldWriter {
        AssetRefFieldWriter::new(self.0.push("emissive_texture"), &self.1)
    }

    pub fn metallic_factor(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("metallic_factor"), &self.1)
    }

    pub fn metallic_roughness_texture(self: &'a Self) -> AssetRefFieldWriter {
        AssetRefFieldWriter::new(self.0.push("metallic_roughness_texture"), &self.1)
    }

    pub fn normal_texture(self: &'a Self) -> AssetRefFieldWriter {
        AssetRefFieldWriter::new(self.0.push("normal_texture"), &self.1)
    }

    pub fn normal_texture_scale(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("normal_texture_scale"), &self.1)
    }

    pub fn roughness_factor(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("roughness_factor"), &self.1)
    }

    pub fn shadow_method(self: &'a Self) -> EnumFieldWriter::<MeshAdvShadowMethodEnum> {
        EnumFieldWriter::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"), &self.1)
    }
}
pub struct MeshAdvMaterialAssetOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for MeshAdvMaterialAssetOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        MeshAdvMaterialAssetOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for MeshAdvMaterialAssetOwned {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialAsset"
    }
}

impl MeshAdvMaterialAssetOwned {
    pub fn alpha_threshold(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("alpha_threshold"), &self.1)
    }

    pub fn backface_culling(self: &Self) -> BooleanFieldOwned {
        BooleanFieldOwned::new(self.0.push("backface_culling"), &self.1)
    }

    pub fn base_color_factor(self: &Self) -> Vec4Owned {
        Vec4Owned::new(self.0.push("base_color_factor"), &self.1)
    }

    pub fn blend_method(self: &Self) -> EnumFieldOwned::<MeshAdvBlendMethodEnum> {
        EnumFieldOwned::<MeshAdvBlendMethodEnum>::new(self.0.push("blend_method"), &self.1)
    }

    pub fn color_texture(self: &Self) -> AssetRefFieldOwned {
        AssetRefFieldOwned::new(self.0.push("color_texture"), &self.1)
    }

    pub fn color_texture_has_alpha_channel(self: &Self) -> BooleanFieldOwned {
        BooleanFieldOwned::new(self.0.push("color_texture_has_alpha_channel"), &self.1)
    }

    pub fn emissive_factor(self: &Self) -> Vec3Owned {
        Vec3Owned::new(self.0.push("emissive_factor"), &self.1)
    }

    pub fn emissive_texture(self: &Self) -> AssetRefFieldOwned {
        AssetRefFieldOwned::new(self.0.push("emissive_texture"), &self.1)
    }

    pub fn metallic_factor(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("metallic_factor"), &self.1)
    }

    pub fn metallic_roughness_texture(self: &Self) -> AssetRefFieldOwned {
        AssetRefFieldOwned::new(self.0.push("metallic_roughness_texture"), &self.1)
    }

    pub fn normal_texture(self: &Self) -> AssetRefFieldOwned {
        AssetRefFieldOwned::new(self.0.push("normal_texture"), &self.1)
    }

    pub fn normal_texture_scale(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("normal_texture_scale"), &self.1)
    }

    pub fn roughness_factor(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("roughness_factor"), &self.1)
    }

    pub fn shadow_method(self: &Self) -> EnumFieldOwned::<MeshAdvShadowMethodEnum> {
        EnumFieldOwned::<MeshAdvShadowMethodEnum>::new(self.0.push("shadow_method"), &self.1)
    }
}
#[derive(Default)]
pub struct MeshAdvMaterialImportedDataRecord(PropertyPath);

impl Field for MeshAdvMaterialImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMaterialImportedDataRecord(property_path)
    }
}

impl Record for MeshAdvMaterialImportedDataRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialImportedData"
    }
}

impl MeshAdvMaterialImportedDataRecord {
}
pub struct MeshAdvMaterialImportedDataReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for MeshAdvMaterialImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        MeshAdvMaterialImportedDataReader(property_path, data_container)
    }
}

impl<'a> RecordReader for MeshAdvMaterialImportedDataReader<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialImportedData"
    }
}

impl<'a> MeshAdvMaterialImportedDataReader<'a> {
}
pub struct MeshAdvMaterialImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMaterialImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        MeshAdvMaterialImportedDataWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for MeshAdvMaterialImportedDataWriter<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialImportedData"
    }
}

impl<'a> MeshAdvMaterialImportedDataWriter<'a> {
}
pub struct MeshAdvMaterialImportedDataOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for MeshAdvMaterialImportedDataOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        MeshAdvMaterialImportedDataOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for MeshAdvMaterialImportedDataOwned {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialImportedData"
    }
}

impl MeshAdvMaterialImportedDataOwned {
}
#[derive(Default)]
pub struct MeshAdvMeshAssetRecord(PropertyPath);

impl Field for MeshAdvMeshAssetRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMeshAssetRecord(property_path)
    }
}

impl Record for MeshAdvMeshAssetRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMeshAsset"
    }
}

impl MeshAdvMeshAssetRecord {
    pub fn material_slots(&self) -> DynamicArrayField::<AssetRefField> {
        DynamicArrayField::<AssetRefField>::new(self.0.push("material_slots"))
    }
}
pub struct MeshAdvMeshAssetReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for MeshAdvMeshAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        MeshAdvMeshAssetReader(property_path, data_container)
    }
}

impl<'a> RecordReader for MeshAdvMeshAssetReader<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshAsset"
    }
}

impl<'a> MeshAdvMeshAssetReader<'a> {
    pub fn material_slots(&self) -> DynamicArrayFieldReader::<AssetRefFieldReader> {
        DynamicArrayFieldReader::<AssetRefFieldReader>::new(self.0.push("material_slots"), self.1)
    }
}
pub struct MeshAdvMeshAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMeshAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        MeshAdvMeshAssetWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for MeshAdvMeshAssetWriter<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshAsset"
    }
}

impl<'a> MeshAdvMeshAssetWriter<'a> {
    pub fn material_slots(self: &'a Self) -> DynamicArrayFieldWriter::<AssetRefFieldWriter> {
        DynamicArrayFieldWriter::<AssetRefFieldWriter>::new(self.0.push("material_slots"), &self.1)
    }
}
pub struct MeshAdvMeshAssetOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for MeshAdvMeshAssetOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        MeshAdvMeshAssetOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for MeshAdvMeshAssetOwned {
    fn schema_name() -> &'static str {
        "MeshAdvMeshAsset"
    }
}

impl MeshAdvMeshAssetOwned {
    pub fn material_slots(self: &Self) -> DynamicArrayFieldOwned::<AssetRefFieldOwned> {
        DynamicArrayFieldOwned::<AssetRefFieldOwned>::new(self.0.push("material_slots"), &self.1)
    }
}
#[derive(Default)]
pub struct MeshAdvMeshImportedDataRecord(PropertyPath);

impl Field for MeshAdvMeshImportedDataRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMeshImportedDataRecord(property_path)
    }
}

impl Record for MeshAdvMeshImportedDataRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedData"
    }
}

impl MeshAdvMeshImportedDataRecord {
    pub fn mesh_parts(&self) -> DynamicArrayField::<MeshAdvMeshImportedDataMeshPartRecord> {
        DynamicArrayField::<MeshAdvMeshImportedDataMeshPartRecord>::new(self.0.push("mesh_parts"))
    }
}
pub struct MeshAdvMeshImportedDataReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for MeshAdvMeshImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        MeshAdvMeshImportedDataReader(property_path, data_container)
    }
}

impl<'a> RecordReader for MeshAdvMeshImportedDataReader<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedData"
    }
}

impl<'a> MeshAdvMeshImportedDataReader<'a> {
    pub fn mesh_parts(&self) -> DynamicArrayFieldReader::<MeshAdvMeshImportedDataMeshPartReader> {
        DynamicArrayFieldReader::<MeshAdvMeshImportedDataMeshPartReader>::new(self.0.push("mesh_parts"), self.1)
    }
}
pub struct MeshAdvMeshImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMeshImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        MeshAdvMeshImportedDataWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for MeshAdvMeshImportedDataWriter<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedData"
    }
}

impl<'a> MeshAdvMeshImportedDataWriter<'a> {
    pub fn mesh_parts(self: &'a Self) -> DynamicArrayFieldWriter::<MeshAdvMeshImportedDataMeshPartWriter> {
        DynamicArrayFieldWriter::<MeshAdvMeshImportedDataMeshPartWriter>::new(self.0.push("mesh_parts"), &self.1)
    }
}
pub struct MeshAdvMeshImportedDataOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for MeshAdvMeshImportedDataOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        MeshAdvMeshImportedDataOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for MeshAdvMeshImportedDataOwned {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedData"
    }
}

impl MeshAdvMeshImportedDataOwned {
    pub fn mesh_parts(self: &Self) -> DynamicArrayFieldOwned::<MeshAdvMeshImportedDataMeshPartOwned> {
        DynamicArrayFieldOwned::<MeshAdvMeshImportedDataMeshPartOwned>::new(self.0.push("mesh_parts"), &self.1)
    }
}
#[derive(Default)]
pub struct MeshAdvMeshImportedDataMeshPartRecord(PropertyPath);

impl Field for MeshAdvMeshImportedDataMeshPartRecord {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMeshImportedDataMeshPartRecord(property_path)
    }
}

impl Record for MeshAdvMeshImportedDataMeshPartRecord {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedDataMeshPart"
    }
}

impl MeshAdvMeshImportedDataMeshPartRecord {
    pub fn indices(&self) -> BytesField {
        BytesField::new(self.0.push("indices"))
    }

    pub fn material_index(&self) -> U32Field {
        U32Field::new(self.0.push("material_index"))
    }

    pub fn normals(&self) -> BytesField {
        BytesField::new(self.0.push("normals"))
    }

    pub fn positions(&self) -> BytesField {
        BytesField::new(self.0.push("positions"))
    }

    pub fn texture_coordinates(&self) -> BytesField {
        BytesField::new(self.0.push("texture_coordinates"))
    }
}
pub struct MeshAdvMeshImportedDataMeshPartReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for MeshAdvMeshImportedDataMeshPartReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        MeshAdvMeshImportedDataMeshPartReader(property_path, data_container)
    }
}

impl<'a> RecordReader for MeshAdvMeshImportedDataMeshPartReader<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedDataMeshPart"
    }
}

impl<'a> MeshAdvMeshImportedDataMeshPartReader<'a> {
    pub fn indices(&self) -> BytesFieldReader {
        BytesFieldReader::new(self.0.push("indices"), self.1)
    }

    pub fn material_index(&self) -> U32FieldReader {
        U32FieldReader::new(self.0.push("material_index"), self.1)
    }

    pub fn normals(&self) -> BytesFieldReader {
        BytesFieldReader::new(self.0.push("normals"), self.1)
    }

    pub fn positions(&self) -> BytesFieldReader {
        BytesFieldReader::new(self.0.push("positions"), self.1)
    }

    pub fn texture_coordinates(&self) -> BytesFieldReader {
        BytesFieldReader::new(self.0.push("texture_coordinates"), self.1)
    }
}
pub struct MeshAdvMeshImportedDataMeshPartWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMeshImportedDataMeshPartWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        MeshAdvMeshImportedDataMeshPartWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for MeshAdvMeshImportedDataMeshPartWriter<'a> {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedDataMeshPart"
    }
}

impl<'a> MeshAdvMeshImportedDataMeshPartWriter<'a> {
    pub fn indices(self: &'a Self) -> BytesFieldWriter {
        BytesFieldWriter::new(self.0.push("indices"), &self.1)
    }

    pub fn material_index(self: &'a Self) -> U32FieldWriter {
        U32FieldWriter::new(self.0.push("material_index"), &self.1)
    }

    pub fn normals(self: &'a Self) -> BytesFieldWriter {
        BytesFieldWriter::new(self.0.push("normals"), &self.1)
    }

    pub fn positions(self: &'a Self) -> BytesFieldWriter {
        BytesFieldWriter::new(self.0.push("positions"), &self.1)
    }

    pub fn texture_coordinates(self: &'a Self) -> BytesFieldWriter {
        BytesFieldWriter::new(self.0.push("texture_coordinates"), &self.1)
    }
}
pub struct MeshAdvMeshImportedDataMeshPartOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for MeshAdvMeshImportedDataMeshPartOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        MeshAdvMeshImportedDataMeshPartOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for MeshAdvMeshImportedDataMeshPartOwned {
    fn schema_name() -> &'static str {
        "MeshAdvMeshImportedDataMeshPart"
    }
}

impl MeshAdvMeshImportedDataMeshPartOwned {
    pub fn indices(self: &Self) -> BytesFieldOwned {
        BytesFieldOwned::new(self.0.push("indices"), &self.1)
    }

    pub fn material_index(self: &Self) -> U32FieldOwned {
        U32FieldOwned::new(self.0.push("material_index"), &self.1)
    }

    pub fn normals(self: &Self) -> BytesFieldOwned {
        BytesFieldOwned::new(self.0.push("normals"), &self.1)
    }

    pub fn positions(self: &Self) -> BytesFieldOwned {
        BytesFieldOwned::new(self.0.push("positions"), &self.1)
    }

    pub fn texture_coordinates(self: &Self) -> BytesFieldOwned {
        BytesFieldOwned::new(self.0.push("texture_coordinates"), &self.1)
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
#[derive(Default)]
pub struct TransformRecord(PropertyPath);

impl Field for TransformRecord {
    fn new(property_path: PropertyPath) -> Self {
        TransformRecord(property_path)
    }
}

impl Record for TransformRecord {
    fn schema_name() -> &'static str {
        "Transform"
    }
}

impl TransformRecord {
    pub fn all_fields(&self) -> AllFieldsRecord {
        AllFieldsRecord::new(self.0.push("all_fields"))
    }

    pub fn position(&self) -> Vec3Record {
        Vec3Record::new(self.0.push("position"))
    }

    pub fn rotation(&self) -> Vec4Record {
        Vec4Record::new(self.0.push("rotation"))
    }

    pub fn scale(&self) -> Vec3Record {
        Vec3Record::new(self.0.push("scale"))
    }
}
pub struct TransformReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for TransformReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        TransformReader(property_path, data_container)
    }
}

impl<'a> RecordReader for TransformReader<'a> {
    fn schema_name() -> &'static str {
        "Transform"
    }
}

impl<'a> TransformReader<'a> {
    pub fn all_fields(&self) -> AllFieldsReader {
        AllFieldsReader::new(self.0.push("all_fields"), self.1)
    }

    pub fn position(&self) -> Vec3Reader {
        Vec3Reader::new(self.0.push("position"), self.1)
    }

    pub fn rotation(&self) -> Vec4Reader {
        Vec4Reader::new(self.0.push("rotation"), self.1)
    }

    pub fn scale(&self) -> Vec3Reader {
        Vec3Reader::new(self.0.push("scale"), self.1)
    }
}
pub struct TransformWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for TransformWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        TransformWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for TransformWriter<'a> {
    fn schema_name() -> &'static str {
        "Transform"
    }
}

impl<'a> TransformWriter<'a> {
    pub fn all_fields(self: &'a Self) -> AllFieldsWriter {
        AllFieldsWriter::new(self.0.push("all_fields"), &self.1)
    }

    pub fn position(self: &'a Self) -> Vec3Writer {
        Vec3Writer::new(self.0.push("position"), &self.1)
    }

    pub fn rotation(self: &'a Self) -> Vec4Writer {
        Vec4Writer::new(self.0.push("rotation"), &self.1)
    }

    pub fn scale(self: &'a Self) -> Vec3Writer {
        Vec3Writer::new(self.0.push("scale"), &self.1)
    }
}
pub struct TransformOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for TransformOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        TransformOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for TransformOwned {
    fn schema_name() -> &'static str {
        "Transform"
    }
}

impl TransformOwned {
    pub fn all_fields(self: &Self) -> AllFieldsOwned {
        AllFieldsOwned::new(self.0.push("all_fields"), &self.1)
    }

    pub fn position(self: &Self) -> Vec3Owned {
        Vec3Owned::new(self.0.push("position"), &self.1)
    }

    pub fn rotation(self: &Self) -> Vec4Owned {
        Vec4Owned::new(self.0.push("rotation"), &self.1)
    }

    pub fn scale(self: &Self) -> Vec3Owned {
        Vec3Owned::new(self.0.push("scale"), &self.1)
    }
}
#[derive(Default)]
pub struct TransformRefRecord(PropertyPath);

impl Field for TransformRefRecord {
    fn new(property_path: PropertyPath) -> Self {
        TransformRefRecord(property_path)
    }
}

impl Record for TransformRefRecord {
    fn schema_name() -> &'static str {
        "TransformRef"
    }
}

impl TransformRefRecord {
    pub fn transform(&self) -> AssetRefField {
        AssetRefField::new(self.0.push("transform"))
    }
}
pub struct TransformRefReader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for TransformRefReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        TransformRefReader(property_path, data_container)
    }
}

impl<'a> RecordReader for TransformRefReader<'a> {
    fn schema_name() -> &'static str {
        "TransformRef"
    }
}

impl<'a> TransformRefReader<'a> {
    pub fn transform(&self) -> AssetRefFieldReader {
        AssetRefFieldReader::new(self.0.push("transform"), self.1)
    }
}
pub struct TransformRefWriter<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for TransformRefWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        TransformRefWriter(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for TransformRefWriter<'a> {
    fn schema_name() -> &'static str {
        "TransformRef"
    }
}

impl<'a> TransformRefWriter<'a> {
    pub fn transform(self: &'a Self) -> AssetRefFieldWriter {
        AssetRefFieldWriter::new(self.0.push("transform"), &self.1)
    }
}
pub struct TransformRefOwned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for TransformRefOwned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        TransformRefOwned(property_path, data_container.clone())
    }
}

impl RecordOwned for TransformRefOwned {
    fn schema_name() -> &'static str {
        "TransformRef"
    }
}

impl TransformRefOwned {
    pub fn transform(self: &Self) -> AssetRefFieldOwned {
        AssetRefFieldOwned::new(self.0.push("transform"), &self.1)
    }
}
#[derive(Default)]
pub struct Vec3Record(PropertyPath);

impl Field for Vec3Record {
    fn new(property_path: PropertyPath) -> Self {
        Vec3Record(property_path)
    }
}

impl Record for Vec3Record {
    fn schema_name() -> &'static str {
        "Vec3"
    }
}

impl Vec3Record {
    pub fn x(&self) -> F32Field {
        F32Field::new(self.0.push("x"))
    }

    pub fn y(&self) -> F32Field {
        F32Field::new(self.0.push("y"))
    }

    pub fn z(&self) -> F32Field {
        F32Field::new(self.0.push("z"))
    }
}
pub struct Vec3Reader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for Vec3Reader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        Vec3Reader(property_path, data_container)
    }
}

impl<'a> RecordReader for Vec3Reader<'a> {
    fn schema_name() -> &'static str {
        "Vec3"
    }
}

impl<'a> Vec3Reader<'a> {
    pub fn x(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("x"), self.1)
    }

    pub fn y(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("y"), self.1)
    }

    pub fn z(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("z"), self.1)
    }
}
pub struct Vec3Writer<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for Vec3Writer<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        Vec3Writer(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for Vec3Writer<'a> {
    fn schema_name() -> &'static str {
        "Vec3"
    }
}

impl<'a> Vec3Writer<'a> {
    pub fn x(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("x"), &self.1)
    }

    pub fn y(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("y"), &self.1)
    }

    pub fn z(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("z"), &self.1)
    }
}
pub struct Vec3Owned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for Vec3Owned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        Vec3Owned(property_path, data_container.clone())
    }
}

impl RecordOwned for Vec3Owned {
    fn schema_name() -> &'static str {
        "Vec3"
    }
}

impl Vec3Owned {
    pub fn x(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("x"), &self.1)
    }

    pub fn y(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("y"), &self.1)
    }

    pub fn z(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("z"), &self.1)
    }
}
#[derive(Default)]
pub struct Vec4Record(PropertyPath);

impl Field for Vec4Record {
    fn new(property_path: PropertyPath) -> Self {
        Vec4Record(property_path)
    }
}

impl Record for Vec4Record {
    fn schema_name() -> &'static str {
        "Vec4"
    }
}

impl Vec4Record {
    pub fn w(&self) -> F32Field {
        F32Field::new(self.0.push("w"))
    }

    pub fn x(&self) -> F32Field {
        F32Field::new(self.0.push("x"))
    }

    pub fn y(&self) -> F32Field {
        F32Field::new(self.0.push("y"))
    }

    pub fn z(&self) -> F32Field {
        F32Field::new(self.0.push("z"))
    }
}
pub struct Vec4Reader<'a>(PropertyPath, DataContainer<'a>);

impl<'a> FieldReader<'a> for Vec4Reader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainer<'a>) -> Self {
        Vec4Reader(property_path, data_container)
    }
}

impl<'a> RecordReader for Vec4Reader<'a> {
    fn schema_name() -> &'static str {
        "Vec4"
    }
}

impl<'a> Vec4Reader<'a> {
    pub fn w(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("w"), self.1)
    }

    pub fn x(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("x"), self.1)
    }

    pub fn y(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("y"), self.1)
    }

    pub fn z(&self) -> F32FieldReader {
        F32FieldReader::new(self.0.push("z"), self.1)
    }
}
pub struct Vec4Writer<'a>(PropertyPath, Rc<RefCell<DataContainerMut<'a>>>);

impl<'a> FieldWriter<'a> for Vec4Writer<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerMut<'a>>>) -> Self {
        Vec4Writer(property_path, data_container.clone())
    }
}

impl<'a> RecordWriter for Vec4Writer<'a> {
    fn schema_name() -> &'static str {
        "Vec4"
    }
}

impl<'a> Vec4Writer<'a> {
    pub fn w(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("w"), &self.1)
    }

    pub fn x(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("x"), &self.1)
    }

    pub fn y(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("y"), &self.1)
    }

    pub fn z(self: &'a Self) -> F32FieldWriter {
        F32FieldWriter::new(self.0.push("z"), &self.1)
    }
}
pub struct Vec4Owned(PropertyPath, Rc<RefCell<Option<DataContainerOwned>>>);

impl FieldOwned for Vec4Owned {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainerOwned>>>) -> Self {
        Vec4Owned(property_path, data_container.clone())
    }
}

impl RecordOwned for Vec4Owned {
    fn schema_name() -> &'static str {
        "Vec4"
    }
}

impl Vec4Owned {
    pub fn w(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("w"), &self.1)
    }

    pub fn x(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("x"), &self.1)
    }

    pub fn y(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("y"), &self.1)
    }

    pub fn z(self: &Self) -> F32FieldOwned {
        F32FieldOwned::new(self.0.push("z"), &self.1)
    }
}
