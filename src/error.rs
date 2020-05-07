use crate::cli::ConfigLoadError;
use peertube_api::error::Error as ApiError;
use std::{error, fmt};

#[derive(Debug)]
pub enum Error {
    Api(ApiError),
    Config(ConfigLoadError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Api(err) => write!(f, "Error connecting to the API: {}", err),
            Error::Config(err) => write!(f, "Config error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Api(err) => Some(err),
            Error::Config(err) => Some(err),
        }
    }
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Error::Api(err)
    }
}

impl From<ConfigLoadError> for Error {
    fn from(err: ConfigLoadError) -> Self {
        Error::Config(err)
    }
}
