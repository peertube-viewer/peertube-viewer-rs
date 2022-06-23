// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

use std::borrow::Cow;

#[derive(Debug, PartialEq)]
pub struct FromHandleError {}

pub fn to_https(mut s: &str) -> Cow<'_, str> {
    s = s.strip_suffix('/').unwrap_or(s);
    if s.starts_with("https://") {
        Cow::Borrowed(s)
    } else if let Some(stripped) = s.strip_prefix("http://") {
        Cow::Owned(format!("https://{}", stripped))
    } else {
        Cow::Owned(format!("https://{}", s))
    }
}

pub fn host_from_handle(s: &str) -> Result<String, FromHandleError> {
    let mut it = s.split('@');
    it.next().ok_or(FromHandleError {})?;
    it.next()
        .map(|i| format!("https://{}", i))
        .ok_or(FromHandleError {})
}

#[cfg(test)]
mod helpers {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn https_helper() {
        assert_eq!(
            to_https("http://foo.bar/"),
            Cow::<'_, str>::Owned("https://foo.bar".to_owned())
        );
        assert_eq!(
            to_https("foo.bar"),
            Cow::<'_, str>::Owned("https://foo.bar".to_owned())
        );
        assert_eq!(
            to_https("foo.bar/"),
            Cow::<'_, str>::Owned("https://foo.bar".to_owned())
        );
        assert_eq!(
            to_https("https://foo.bar/"),
            Cow::<'_, str>::Owned("https://foo.bar".to_owned())
        );
        assert_eq!(
            to_https("https://foo.bar"),
            Cow::Borrowed("https://foo.bar")
        );
    }

    #[test]
    fn handle_helper() {
        assert_eq!(
            host_from_handle("channel@instance.org").unwrap(),
            "https://instance.org".to_owned()
        );
        assert!(host_from_handle("no ").is_err());
    }
}
