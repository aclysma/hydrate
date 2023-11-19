#[derive(Debug, Copy, Clone)]
pub enum DataSetError {
    ValueDoesNotMatchSchema,
    PathParentIsNull,
    PathDynamicArrayEntryDoesNotExist,
    UnexpectedEnumSymbol,
    DuplicateAssetId,
    AssetNotFound,
    ImportDataNotFound,
    SingleObjectDoesNotMatchSchema,
    LocationCycleDetected,
    LocationParentNotFound,
    SchemaNotFound,
    InvalidSchema,
    UuidParseError,
    // the data was in a container, but moved out of it (i.e. Option::take())
    DataTaken,
}

pub type DataSetResult<T> = Result<T, DataSetError>;
