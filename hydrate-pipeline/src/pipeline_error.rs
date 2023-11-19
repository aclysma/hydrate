use hydrate_data::DataSetError;
use std::sync::Arc;

pub type PipelineResult<T> = Result<T, PipelineError>;

#[derive(Debug, Clone)]
pub enum PipelineError {
    StringError(String),
    DataSetError(DataSetError),
    IoError(Arc<std::io::Error>),
    BincodeError(Arc<bincode::Error>),
    JsonError(Arc<serde_json::Error>),
}

impl std::error::Error for PipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            PipelineError::StringError(_) => None,
            PipelineError::DataSetError(_) => None,
            PipelineError::IoError(ref e) => Some(&**e),
            PipelineError::BincodeError(ref e) => Some(&**e),
            PipelineError::JsonError(ref e) => Some(&**e),
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
            PipelineError::IoError(ref e) => e.fmt(fmt),
            PipelineError::BincodeError(ref e) => e.fmt(fmt),
            PipelineError::JsonError(ref e) => e.fmt(fmt),
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
