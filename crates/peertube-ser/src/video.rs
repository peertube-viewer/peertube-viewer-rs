use crate::search::{Channel, IdentifiedLabel, Language};
use serde::{Deserialize, Serialize};

/// Structure used to deserialize the json output from fetching a video description
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Description {
    pub description: Option<String>,
}

/// Structure used to deserialize the json output from fetching video data
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Video {
    pub uuid: String,
    pub createdAt: Option<String>,
    pub publishedAt: Option<String>,
    pub updatedAt: Option<String>,
    pub originallyPublishedAt: Option<String>,
    pub description: Option<String>,
    pub duration: Option<i64>,
    pub isLocal: Option<bool>,
    pub name: String,
    pub thumbnailPath: Option<String>,
    pub previewPath: Option<String>,
    pub embedPath: Option<String>,
    pub views: Option<i64>,
    pub likes: Option<i64>,
    pub dislikes: Option<i64>,
    pub nsfw: Option<bool>,
    pub account: Channel,
    pub channel: Channel,
    pub category: IdentifiedLabel,
    pub licence: IdentifiedLabel,
    pub language: Language,
    pub privacy: IdentifiedLabel,

    /// The list of files for the video
    /// Each file corresponds to an available resolution
    pub files: Vec<File>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Resolution {
    pub id: i64,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct File {
    pub magnetUri: String,
    pub resolution: Resolution,
    pub size: i64,
    pub torrentUrl: String,
    pub torrentDownloadUrl: String,
    pub fileUrl: String,
    pub fileDownloadUrl: String,
}
