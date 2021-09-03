
/// Generic error that contains all the different kinds of errors that may occur when using the API
#[derive(Debug, Clone)]
pub enum ObjectDbError {
    TypeError,
    StringError(String)
}

impl std::error::Error for ObjectDbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ObjectDbError::TypeError => None,
            ObjectDbError::StringError(_) => None,
        }
    }
}

impl core::fmt::Display for ObjectDbError {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        match *self {
            ObjectDbError::TypeError => "TypeError".fmt(fmt),
            ObjectDbError::StringError(ref s) => s.fmt(fmt),
        }
    }
}

impl From<&str> for ObjectDbError {
    fn from(str: &str) -> Self {
        ObjectDbError::StringError(str.to_string())
    }
}

impl From<String> for ObjectDbError {
    fn from(string: String) -> Self {
        ObjectDbError::StringError(string)
    }
}

pub type ObjectDbResult<T> = Result<T, ObjectDbError>;