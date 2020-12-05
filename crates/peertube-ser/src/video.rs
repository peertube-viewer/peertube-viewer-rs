#![allow(clippy::redundant_pattern_matching)] // Suppress warnings caused by nanoserde

use crate::common::Channel;
use crate::search::{IdentifiedLabel, Language};
use nanoserde::DeJson;

/// Structure used to deserialize the json output from fetching a video description
#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Description {
    pub description: Option<String>,
}

/// Structure used to deserialize the json output from fetching video data
#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Video {
    pub uuid: String,
    pub createdAt: String,
    pub publishedAt: String,
    pub updatedAt: String,
    pub description: Option<String>,
    pub duration: u64,
    #[nserde(default)]
    pub isLocal: bool,
    pub name: String,
    pub thumbnailPath: String,
    pub previewPath: String,
    pub embedPath: String,
    pub views: u64,
    pub likes: u64,
    pub dislikes: u64,
    pub nsfw: bool,

    #[nserde(default)]
    pub isLive: bool,
    pub account: Channel,
    pub channel: Channel,
    pub category: IdentifiedLabel,
    pub licence: IdentifiedLabel,
    pub language: Language,
    pub privacy: IdentifiedLabel,

    /// The list of files for the video
    /// Each file corresponds to an available resolution
    #[nserde(default)]
    pub files: Vec<File>,
    #[nserde(default)]
    pub streamingPlaylists: Vec<StreamingPlaylist>,
}

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct Resolution {
    pub id: u64,
    pub label: String,
}

#[derive(DeJson, Debug)]
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

#[derive(DeJson, Debug)]
#[allow(non_snake_case)]
pub struct StreamingPlaylist {
    pub id: u64,
    pub playlistUrl: String,
}
