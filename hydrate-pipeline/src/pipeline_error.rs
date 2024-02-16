use hydrate_data::DataSetError;
use hydrate_schema::DataSetErrorWithBacktrace;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum PipelineError {
    StringError(String),
    DataSetError(DataSetError),
    DataSetErrorWithBacktrace(DataSetErrorWithBacktrace),
    IoError(Arc<std::io::Error>),
    BincodeError(Arc<bincode::Error>),
    JsonError(Arc<serde_json::Error>),
    UuidError(uuid::Error),
    ThumbnailUnavailable,
}

impl std::error::Error for PipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            PipelineError::StringError(_) => None,
            PipelineError::DataSetError(_) => None,
            PipelineError::DataSetErrorWithBacktrace(_) => None,
            PipelineError::IoError(ref e) => Some(&**e),
            PipelineError::BincodeError(ref e) => Some(&**e),
            PipelineError::JsonError(ref e) => Some(&**e),
            PipelineError::UuidError(ref e) => Some(e),
            PipelineError::ThumbnailUnavailable => None,
        }
    }
}

impl core::fmt::Display for PipelineError {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        match *self {
            PipelineError::StringError(ref e) => e.fmt(fmt),
            PipelineError::DataSetError(ref e) => {
                use std::fmt::Debug;
                e.fmt(fmt)
            }
            PipelineError::DataSetErrorWithBacktrace(ref e) => {
                use std::fmt::Debug;
                e.fmt(fmt)
            }
            PipelineError::IoError(ref e) => e.fmt(fmt),
            PipelineError::BincodeError(ref e) => e.fmt(fmt),
            PipelineError::JsonError(ref e) => e.fmt(fmt),
            PipelineError::UuidError(ref e) => e.fmt(fmt),
            PipelineError::ThumbnailUnavailable => "ThumbnailUnavailable".fmt(fmt),
        }
    }
}

impl From<&str> for PipelineError {
    fn from(str: &str) -> Self {
        PipelineError::StringError(str.to_string())
    }
}

impl From<String> for PipelineError {
    fn from(string: String) -> Self {
        PipelineError::StringError(string)
    }
}

impl From<DataSetError> for PipelineError {
    fn from(error: DataSetError) -> Self {
        PipelineError::DataSetError(error)
    }
}

impl From<DataSetErrorWithBacktrace> for PipelineError {
    fn from(error: DataSetErrorWithBacktrace) -> Self {
        PipelineError::DataSetErrorWithBacktrace(error)
    }
}

impl From<std::io::Error> for PipelineError {
    fn from(error: std::io::Error) -> Self {
        PipelineError::IoError(Arc::new(error))
    }
}

impl From<bincode::Error> for PipelineError {
    fn from(error: bincode::Error) -> Self {
        PipelineError::BincodeError(Arc::new(error))
    }
}

impl From<serde_json::Error> for PipelineError {
    fn from(error: serde_json::Error) -> Self {
        PipelineError::JsonError(Arc::new(error))
    }
}

impl From<uuid::Error> for PipelineError {
    fn from(error: uuid::Error) -> Self {
        PipelineError::UuidError(error)
    }
}

#[derive(Clone)]
pub struct PipelineErrorWithBacktrace {
    pub error: PipelineError,
    #[cfg(all(backtrace, debug_assertions))]
    backtrace: Arc<backtrace::Backtrace>,
}

impl std::error::Error for PipelineErrorWithBacktrace {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.error.source()
    }
}

impl core::fmt::Display for PipelineErrorWithBacktrace {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        self.error.fmt(fmt)
    }
}

impl<T: Into<PipelineError>> From<T> for PipelineErrorWithBacktrace {
    fn from(error: T) -> Self {
        PipelineErrorWithBacktrace {
            error: error.into(),
            #[cfg(all(backtrace, debug_assertions))]
            backtrace: Arc::new(backtrace::Backtrace::new()),
        }
    }
}

impl std::fmt::Debug for PipelineErrorWithBacktrace {
    #[cfg(not(all(backtrace, debug_assertions)))]
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{:?}", self.error)
    }

    #[cfg(all(backtrace, debug_assertions))]
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let backtrace = match &self.error {
            PipelineError::DataSetErrorWithBacktrace(e) => &e.backtrace,
            _ => &*self.backtrace,
        };
        write!(f, "{:?}:\n{:?}", self.error, backtrace)
    }
}

pub type PipelineResult<T> = Result<T, PipelineErrorWithBacktrace>;
