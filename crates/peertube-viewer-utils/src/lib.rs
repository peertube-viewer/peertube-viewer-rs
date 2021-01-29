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
