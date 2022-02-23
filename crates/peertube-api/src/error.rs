// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

use std::{error, fmt, io, sync::Arc};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Ureq(Arc<ureq::Error>),
    NoContent,

    /// contains the length of the array
    OutOfBound(usize),
    Io(Arc<io::Error>),
    Serde(Arc<serde_json::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Ureq(err) => write!(f, "Connection error: {}", err),
            Error::Io(err) => write!(f, "Connection error: {}", err),
            Error::NoContent => write!(f, "No content"),
            Error::OutOfBound(len) => write!(f, "Out of bound access, the array is of len {}", len),
            Error::Serde(err) => write!(f, "Deserialisation error: {}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Ureq(err) => Some(&**err),
            Error::Io(err) => Some(&**err),
            Error::Serde(err) => Some(&**err),
            Error::NoContent | Error::OutOfBound(_) => None,
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(err: ureq::Error) -> Self {
        Error::Ureq(Arc::new(err))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(Arc::new(err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serde(Arc::new(err))
    }
}
