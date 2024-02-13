#[cfg(debug_assertions)]
use std::sync::Arc;

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

//impl ErrorSupportingBacktrace for DataSetError {}

#[derive(Clone)]
pub struct DataSetErrorWithBacktrace {
    pub error: DataSetError,
    #[cfg(debug_assertions)]
    pub backtrace: Arc<backtrace::Backtrace>,
}

impl From<DataSetError> for DataSetErrorWithBacktrace {
    fn from(error: DataSetError) -> Self {
        DataSetErrorWithBacktrace {
            error,
            #[cfg(debug_assertions)]
            backtrace: Arc::new(backtrace::Backtrace::new()),
        }
    }
}

impl std::fmt::Debug for DataSetErrorWithBacktrace {
    #[cfg(not(debug_assertions))]
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{:?}", self.error)
    }

    #[cfg(debug_assertions)]
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{:?}:\n{:?}", self.error, *self.backtrace)
    }
}

pub type DataSetResult<T> = Result<T, DataSetErrorWithBacktrace>;
