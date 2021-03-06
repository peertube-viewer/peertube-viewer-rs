// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

use crate::common::{dates_deser, Channel, VideoState};
use crate::search::{IdentifiedLabel, Language};
use serde::Deserialize;

use time::OffsetDateTime;

/// Structure used to deserialize the json output from fetching a video description
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Description {
    pub description: Option<String>,
}

/// Structure used to deserialize the json output from fetching video data
#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Video {
    pub uuid: String,
    #[serde(with = "dates_deser")]
    pub createdAt: OffsetDateTime,
    #[serde(with = "dates_deser")]
    pub publishedAt: OffsetDateTime,
    pub description: Option<String>,
    pub duration: u64,
    #[serde(default)]
    pub isLocal: bool,
    pub name: String,
    pub thumbnailPath: String,
    pub previewPath: String,
    pub embedPath: String,
    pub views: u64,
    pub likes: u64,
    pub dislikes: u64,
    pub nsfw: bool,

    #[serde(default)]
    pub isLive: bool,
    #[serde(default)]
    pub state: VideoState,
    pub account: Channel,
    pub channel: Channel,
    pub category: IdentifiedLabel,
    pub licence: IdentifiedLabel,
    pub language: Language,
    pub privacy: IdentifiedLabel,

    /// The list of files for the video
    /// Each file corresponds to an available resolution
    #[serde(default)]
    pub files: Vec<File>,
    #[serde(default)]
    pub streamingPlaylists: Vec<StreamingPlaylist>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Resolution {
    pub id: u64,
    pub label: String,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct StreamingPlaylist {
    pub id: u64,
    pub playlistUrl: String,
}
