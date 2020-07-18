use crate::common::Channel;
use crate::search::{IdentifiedLabel, Language};
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
    pub createdAt: String,
    pub publishedAt: String,
    pub updatedAt: String,
    pub description: Option<String>,
    pub duration: u64,
    pub isLocal: bool,
    pub name: String,
    pub thumbnailPath: String,
    pub previewPath: String,
    pub embedPath: String,
    pub views: u64,
    pub likes: u64,
    pub dislikes: u64,
    pub nsfw: bool,
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
    pub id: u64,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct File {
    pub magnetUri: String,
    pub resolution: Resolution,
    pub size: u64,
    pub torrentUrl: String,
    pub torrentDownloadUrl: String,
    pub fileUrl: String,
    pub fileDownloadUrl: String,
}
