#[derive(Debug, Copy, Clone)]
pub enum DataSetError {
    ValueDoesNotMatchSchema,
    PathParentIsNull,
    PathDynamicArrayEntryDoesNotExist,
    UnexpectedEnumSymbol,
    DuplicateAssetId,
    DuplicateEntryKey,
    EntryKeyNotFound,
    AssetNotFound,
    ImportDataNotFound,
    SingleObjectDoesNotMatchSchema,
    LocationCycleDetected,
    LocationParentNotFound,
    SchemaNotFound,
    InvalidSchema,
    UuidParseError,
    StorageFormatError,
    NewLocationIsChildOfCurrentAsset,
    UnknownPathNamespace,

    // the data was in a container, but moved out of it (i.e. Option::take())
    DataTaken,
}

pub type DataSetResult<T> = Result<T, DataSetError>;
