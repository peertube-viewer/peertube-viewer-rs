use crate::cli::ConfigLoadError;
use peertube_api::error::Error as ApiError;
use std::{error, fmt, io};

#[derive(Debug)]
pub enum Error {
    Api(ApiError),
    Config(ConfigLoadError),
    Readline(rustyline::error::ReadlineError),
    RuntimeInit(io::Error),
    VideoLaunch(io::Error),
    BlacklistedInstance(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Api(_) => write!(f, "Error connecting to the API"),
            Error::Config(_) => write!(f, "Config error"),
            Error::Readline(_) => write!(f, "Input error"),
            Error::RuntimeInit(_) => write!(f, "Unable to init the async runtime"),
            Error::VideoLaunch(_) => write!(f, "Unable to launch video"),
            Error::BlacklistedInstance(s) => {
                write!(f, "Connexion to a blacklisted instance: {}", s)
            }
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Api(err) => Some(err),
            Error::Config(err) => Some(err),
            Error::Readline(err) => Some(err),
            Error::RuntimeInit(err) => Some(err),
            Error::VideoLaunch(err) => Some(err),
            Error::BlacklistedInstance(_) => None,
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

impl From<rustyline::error::ReadlineError> for Error {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        Error::Readline(err)
    }
}
