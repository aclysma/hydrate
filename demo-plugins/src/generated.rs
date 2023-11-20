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

    pub fn dynamic_array_i32(&self) -> DynamicArrayFieldAccessor::<I32FieldAccessor> {
        DynamicArrayFieldAccessor::<I32FieldAccessor>::new(self.0.push("dynamic_array_i32"))
    }

    pub fn dynamic_array_vec3(&self) -> DynamicArrayFieldAccessor::<Vec3Accessor> {
        DynamicArrayFieldAccessor::<Vec3Accessor>::new(self.0.push("dynamic_array_vec3"))
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

    pub fn nullable_bool(&self) -> NullableFieldAccessor::<BooleanFieldAccessor> {
        NullableFieldAccessor::<BooleanFieldAccessor>::new(self.0.push("nullable_bool"))
    }

    pub fn nullable_vec3(&self) -> NullableFieldAccessor::<Vec3Accessor> {
        NullableFieldAccessor::<Vec3Accessor>::new(self.0.push("nullable_vec3"))
    }

    pub fn reference(&self) -> AssetRefFieldAccessor {
        AssetRefFieldAccessor::new(self.0.push("reference"))
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
}
pub struct AllFieldsReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for AllFieldsReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct AllFieldsWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for AllFieldsWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct AllFieldsRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for AllFieldsRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        AllFieldsRecord(property_path, data_container.clone())
    }
}

impl Record for AllFieldsRecord {
    type Reader<'a> = AllFieldsReader<'a>;

    fn schema_name() -> &'static str {
        "AllFields"
    }
}

impl AllFieldsRecord {
    pub fn boolean(self: &Self) -> BooleanField {
        BooleanField::new(self.0.push("boolean"), &self.1)
    }

    pub fn dynamic_array_i32(self: &Self) -> DynamicArrayField::<I32Field> {
        DynamicArrayField::<I32Field>::new(self.0.push("dynamic_array_i32"), &self.1)
    }

    pub fn dynamic_array_vec3(self: &Self) -> DynamicArrayField::<Vec3Record> {
        DynamicArrayField::<Vec3Record>::new(self.0.push("dynamic_array_vec3"), &self.1)
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

    pub fn nullable_bool(self: &Self) -> NullableField::<BooleanField> {
        NullableField::<BooleanField>::new(self.0.push("nullable_bool"), &self.1)
    }

    pub fn nullable_vec3(self: &Self) -> NullableField::<Vec3Record> {
        NullableField::<Vec3Record>::new(self.0.push("nullable_vec3"), &self.1)
    }

    pub fn reference(self: &Self) -> AssetRefField {
        AssetRefField::new(self.0.push("reference"), &self.1)
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
pub struct GlslBuildTargetAssetReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for GlslBuildTargetAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct GlslBuildTargetAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for GlslBuildTargetAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct GlslBuildTargetAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GlslBuildTargetAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GlslBuildTargetAssetRecord(property_path, data_container.clone())
    }
}

impl Record for GlslBuildTargetAssetRecord {
    type Reader<'a> = GlslBuildTargetAssetReader<'a>;

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
pub struct GlslSourceFileAssetReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for GlslSourceFileAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct GlslSourceFileAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for GlslSourceFileAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct GlslSourceFileAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GlslSourceFileAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GlslSourceFileAssetRecord(property_path, data_container.clone())
    }
}

impl Record for GlslSourceFileAssetRecord {
    type Reader<'a> = GlslSourceFileAssetReader<'a>;

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
pub struct GlslSourceFileImportedDataReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for GlslSourceFileImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct GlslSourceFileImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for GlslSourceFileImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct GlslSourceFileImportedDataRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GlslSourceFileImportedDataRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GlslSourceFileImportedDataRecord(property_path, data_container.clone())
    }
}

impl Record for GlslSourceFileImportedDataRecord {
    type Reader<'a> = GlslSourceFileImportedDataReader<'a>;

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
pub struct GpuBufferAssetReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for GpuBufferAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct GpuBufferAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for GpuBufferAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct GpuBufferAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GpuBufferAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GpuBufferAssetRecord(property_path, data_container.clone())
    }
}

impl Record for GpuBufferAssetRecord {
    type Reader<'a> = GpuBufferAssetReader<'a>;

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
pub struct GpuBufferImportedDataReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for GpuBufferImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct GpuBufferImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for GpuBufferImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct GpuBufferImportedDataRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GpuBufferImportedDataRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GpuBufferImportedDataRecord(property_path, data_container.clone())
    }
}

impl Record for GpuBufferImportedDataRecord {
    type Reader<'a> = GpuBufferImportedDataReader<'a>;

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
pub struct GpuImageAssetReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for GpuImageAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct GpuImageAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for GpuImageAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct GpuImageAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GpuImageAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GpuImageAssetRecord(property_path, data_container.clone())
    }
}

impl Record for GpuImageAssetRecord {
    type Reader<'a> = GpuImageAssetReader<'a>;

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
pub struct GpuImageImportedDataReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for GpuImageImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct GpuImageImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for GpuImageImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct GpuImageImportedDataRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for GpuImageImportedDataRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        GpuImageImportedDataRecord(property_path, data_container.clone())
    }
}

impl Record for GpuImageImportedDataRecord {
    type Reader<'a> = GpuImageImportedDataReader<'a>;

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
pub struct MeshAdvMaterialAssetReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for MeshAdvMaterialAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct MeshAdvMaterialAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMaterialAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct MeshAdvMaterialAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for MeshAdvMaterialAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        MeshAdvMaterialAssetRecord(property_path, data_container.clone())
    }
}

impl Record for MeshAdvMaterialAssetRecord {
    type Reader<'a> = MeshAdvMaterialAssetReader<'a>;

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
pub struct MeshAdvMaterialImportedDataAccessor(PropertyPath);

impl FieldAccessor for MeshAdvMaterialImportedDataAccessor {
    fn new(property_path: PropertyPath) -> Self {
        MeshAdvMaterialImportedDataAccessor(property_path)
    }
}

impl RecordAccessor for MeshAdvMaterialImportedDataAccessor {
    fn schema_name() -> &'static str {
        "MeshAdvMaterialImportedData"
    }
}

impl MeshAdvMaterialImportedDataAccessor {
}
pub struct MeshAdvMaterialImportedDataReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for MeshAdvMaterialImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct MeshAdvMaterialImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMaterialImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct MeshAdvMaterialImportedDataRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for MeshAdvMaterialImportedDataRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        MeshAdvMaterialImportedDataRecord(property_path, data_container.clone())
    }
}

impl Record for MeshAdvMaterialImportedDataRecord {
    type Reader<'a> = MeshAdvMaterialImportedDataReader<'a>;

    fn schema_name() -> &'static str {
        "MeshAdvMaterialImportedData"
    }
}

impl MeshAdvMaterialImportedDataRecord {
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
pub struct MeshAdvMeshAssetReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for MeshAdvMeshAssetReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct MeshAdvMeshAssetWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMeshAssetWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct MeshAdvMeshAssetRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for MeshAdvMeshAssetRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        MeshAdvMeshAssetRecord(property_path, data_container.clone())
    }
}

impl Record for MeshAdvMeshAssetRecord {
    type Reader<'a> = MeshAdvMeshAssetReader<'a>;

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
pub struct MeshAdvMeshImportedDataReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for MeshAdvMeshImportedDataReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct MeshAdvMeshImportedDataWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMeshImportedDataWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct MeshAdvMeshImportedDataRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for MeshAdvMeshImportedDataRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        MeshAdvMeshImportedDataRecord(property_path, data_container.clone())
    }
}

impl Record for MeshAdvMeshImportedDataRecord {
    type Reader<'a> = MeshAdvMeshImportedDataReader<'a>;

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
pub struct MeshAdvMeshImportedDataMeshPartReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for MeshAdvMeshImportedDataMeshPartReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct MeshAdvMeshImportedDataMeshPartWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for MeshAdvMeshImportedDataMeshPartWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct MeshAdvMeshImportedDataMeshPartRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for MeshAdvMeshImportedDataMeshPartRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        MeshAdvMeshImportedDataMeshPartRecord(property_path, data_container.clone())
    }
}

impl Record for MeshAdvMeshImportedDataMeshPartRecord {
    type Reader<'a> = MeshAdvMeshImportedDataMeshPartReader<'a>;

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
pub struct TransformReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for TransformReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct TransformWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for TransformWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct TransformRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for TransformRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        TransformRecord(property_path, data_container.clone())
    }
}

impl Record for TransformRecord {
    type Reader<'a> = TransformReader<'a>;

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
pub struct TransformRefReader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for TransformRefReader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct TransformRefWriter<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for TransformRefWriter<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct TransformRefRecord(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for TransformRefRecord {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        TransformRefRecord(property_path, data_container.clone())
    }
}

impl Record for TransformRefRecord {
    type Reader<'a> = TransformRefReader<'a>;

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
pub struct Vec3Reader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for Vec3Reader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct Vec3Writer<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for Vec3Writer<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct Vec3Record(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for Vec3Record {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        Vec3Record(property_path, data_container.clone())
    }
}

impl Record for Vec3Record {
    type Reader<'a> = Vec3Reader<'a>;

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
pub struct Vec4Reader<'a>(PropertyPath, DataContainerRef<'a>);

impl<'a> FieldReader<'a> for Vec4Reader<'a> {
    fn new(property_path: PropertyPath, data_container: DataContainerRef<'a>) -> Self {
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
pub struct Vec4Writer<'a>(PropertyPath, Rc<RefCell<DataContainerRefMut<'a>>>);

impl<'a> FieldWriter<'a> for Vec4Writer<'a> {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<DataContainerRefMut<'a>>>) -> Self {
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
pub struct Vec4Record(PropertyPath, Rc<RefCell<Option<DataContainer>>>);

impl Field for Vec4Record {
    fn new(property_path: PropertyPath, data_container: &Rc<RefCell<Option<DataContainer>>>) -> Self {
        Vec4Record(property_path, data_container.clone())
    }
}

impl Record for Vec4Record {
    type Reader<'a> = Vec4Reader<'a>;

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
