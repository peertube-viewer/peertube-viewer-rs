// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

#[derive(Debug, Clone, PartialEq)]
pub enum UrlType {
    /// The url is a video with a UUID
    Video(String),
    /// The url is a channel with handle
    Channel(String),
    Search(String),
    LandingPage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUrl {
    pub instance: String,
    pub url_data: UrlType,
}

impl ParsedUrl {
    pub fn from_url(i: &str) -> Option<ParsedUrl> {
        if !i.starts_with("http://") && !i.starts_with("https://") {
            return None;
        }

        let parsed = url::Url::parse(i).ok()?;

        let mut path_iter = parsed.path_segments()?;

        let instance;
        match parsed.domain() {
            Some(domain) => {
                instance = format!("https://{}", domain.split(' ').next().expect("Unreachable"));
                match (path_iter.next(), path_iter.next(), path_iter.next()) {
                    (Some("videos"), Some("watch"), Some(uuid)) => Some(ParsedUrl {
                        instance,
                        url_data: UrlType::Video(uuid.to_string()),
                    }),
                    (Some("w"), Some(uuid), None) => Some(ParsedUrl {
                        instance,
                        url_data: UrlType::Video(uuid.to_string()),
                    }),
                    (Some("video-channels"), Some(handle), Some("videos") | None) => {
                        Some(ParsedUrl {
                            instance,
                            url_data: UrlType::Channel(handle.to_string()),
                        })
                    }
                    (Some("c"), Some(handle), Some("videos") | None) => Some(ParsedUrl {
                        instance,
                        url_data: UrlType::Channel(handle.to_string()),
                    }),
                    (Some("search"), _, _) => {
                        for (name, value) in parsed.query_pairs() {
                            if name == "search" {
                                return Some(ParsedUrl {
                                    instance,
                                    url_data: UrlType::Search(value.into_owned()),
                                });
                            }
                        }
                        Some(ParsedUrl {
                            instance,
                            url_data: UrlType::LandingPage,
                        })
                    }

                    (_, _, _) => Some(ParsedUrl {
                        instance,
                        url_data: UrlType::LandingPage,
                    }),
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod url_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn frontend_url_parser_test() {
        assert_eq!(
            ParsedUrl::from_url(
                "https://video.ploud.fr/videos/watch/9c9de5e8-0a1e-484a-b099-e80766180a6d"
            ),
            Some(ParsedUrl {
                instance: "https://video.ploud.fr".to_string(),
                url_data: UrlType::Video("9c9de5e8-0a1e-484a-b099-e80766180a6d".to_string())
            })
        );
        assert_eq!(
            ParsedUrl::from_url(
                "https://video.ploud.fr/video-channels/bf54d359-cfad-4935-9d45-9d6be93f63e8@framatube.org/videos"
            ),
            Some(ParsedUrl {
                instance: "https://video.ploud.fr".to_string(),
                url_data: UrlType::Channel("bf54d359-cfad-4935-9d45-9d6be93f63e8@framatube.org".to_string())
            })
        );
        assert_eq!(
            ParsedUrl::from_url(
                "https://video.ploud.fr/search?search=what%20is%20peertube&searchTarget=local"
            ),
            Some(ParsedUrl {
                instance: "https://video.ploud.fr".to_string(),
                url_data: UrlType::Search("what is peertube".to_string())
            })
        );
        assert_eq!(
            ParsedUrl::from_url("https://video.ploud.fr"),
            Some(ParsedUrl {
                instance: "https://video.ploud.fr".to_string(),
                url_data: UrlType::LandingPage
            })
        );
        assert_eq!(
            ParsedUrl::from_url("https://skeptikon.fr/c/le_reveilleur/videos"),
            Some(ParsedUrl {
                instance: "https://skeptikon.fr".to_string(),
                url_data: UrlType::Channel("le_reveilleur".to_string())
            })
        );
        assert_eq!(
            ParsedUrl::from_url("https://skeptikon.fr/w/qRR6xVFzEbaAC3uk62PGhP"),
            Some(ParsedUrl {
                instance: "https://skeptikon.fr".to_string(),
                url_data: UrlType::Video("qRR6xVFzEbaAC3uk62PGhP".to_string())
            })
        );
    }
}
