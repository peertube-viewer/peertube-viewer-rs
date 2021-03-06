// This file is part of peertube-viewer-rs.
//
// peertube-viewer-rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.
//
// peertube-viewer-rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with peertube-viewer-rs. If not, see <https://www.gnu.org/licenses/>.

use std::borrow::Cow;
use std::convert::TryFrom;
use std::sync::Arc;

use peertube_ser::channels::Channels;
use peertube_ser::video::{Description, File, StreamingPlaylist, Video as FullVideo};
use peertube_ser::{Comments, Videos};
use peertube_viewer_utils::to_https;

use crate::channels::Channel;
use crate::comments::Comment;
use crate::error;
use crate::video::Video;

/// Connection to an instance
/// Video that this instance returns through `search_videos` will all use the instance which
/// created them. This avoids connecting to many distinct instances.
pub struct Instance {
    host: String,
    user_agent: Option<String>,
    include_nsfw: &'static str,
    local: bool,

    // If true (when connected to sepia_search for instance)
    // this means that additional info outside of search should
    // come from the hosts of videos instead of this instance
    // because the instance doesn't have them
    is_search: bool,
}

impl Instance {
    pub fn new(
        host: String,
        include_nsfw: bool,
        local: bool,
        user_agent: Option<String>,
        is_search: bool,
    ) -> Arc<Instance> {
        Arc::new(Instance {
            host,
            user_agent,
            include_nsfw: nsfw_string(include_nsfw),
            local,
            is_search,
        })
    }

    /// Adds the user agent if there is one
    fn add_user_agent(&self, req: ureq::Request) -> ureq::Request {
        if let Some(user_agent) = &self.user_agent {
            req.set("User-Agent", user_agent)
        } else {
            req
        }
    }

    /// Perform a search for the given query
    pub fn search_videos(
        self: &Arc<Instance>,
        query: &str,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Video>, usize)> {
        let url = format!("{}/api/v1/search/videos", self.host);

        let mut req = self
            .add_user_agent(ureq::get(&url))
            .query("search", query)
            .query("count", &nb.to_string())
            .query("start", &offset.to_string())
            .query("nsfw", self.include_nsfw);

        if self.local {
            req = req.query("filter", "local");
        }

        let search_res: Videos = serde_json::from_str(&req.call()?.into_string()?)?;
        let mut res = Vec::new();

        for video in search_res.data {
            res.push(Video::from_search(self, video));
        }

        Ok((res, search_res.total))
    }

    /// Get the videos from a given channel
    pub fn channel_videos(
        self: &Arc<Instance>,
        host: &str,
        handle: &str,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Video>, usize)> {
        let url = format!(
            "{}/api/v1/video-channels/{}/videos",
            self.api_host(host),
            handle
        );

        let mut req = self
            .add_user_agent(ureq::get(&url))
            .query("nsfw", self.include_nsfw)
            .query("count", &nb.to_string())
            .query("start", &offset.to_string())
            .set("User-Agent", concat!(env!("CARGO_PKG_NAME")));

        if self.local {
            req = req.query("filter", "local");
        }

        let video_res: Videos = serde_json::from_str(&req.call()?.into_string()?)?;
        let mut res = Vec::new();
        for video in video_res.data {
            res.push(Video::from_search(self, video));
        }

        Ok((res, video_res.total))
    }

    pub fn comments(
        self: &Arc<Instance>,
        host: &str,
        video_uuid: &str,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Comment>, usize)> {
        let url = format!(
            "{}/api/v1/videos/{}/comment-threads",
            self.api_host(host),
            video_uuid
        );

        let req = self
            .add_user_agent(ureq::get(&url))
            .query("count", &nb.to_string())
            .query("start", &offset.to_string());

        let comment_res: Comments = serde_json::from_str(&req.call()?.into_string()?)?;
        let mut res = Vec::new();
        for comment in comment_res.data {
            if let Ok(c) = Comment::try_from(comment) {
                res.push(c);
            }
        }

        Ok((res, comment_res.total))
    }

    /// Returns the trending videos of an instance
    pub fn trending_videos(
        self: &Arc<Instance>,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Video>, usize)> {
        let url = format!("{}/api/v1/videos", self.host);

        let mut req = self
            .add_user_agent(ureq::get(&url))
            .query("sort", "-trending")
            .query("count", &nb.to_string())
            .query("start", &offset.to_string())
            .query("nsfw", self.include_nsfw);

        if self.local {
            req = req.query("filter", "local");
        }

        let search_res: Videos = serde_json::from_str(&req.call()?.into_string()?)?;
        let mut res = Vec::new();
        for video in search_res.data {
            res.push(Video::from_search(self, video));
        }

        Ok((res, search_res.total))
    }

    /// Perform a search for the given query
    pub fn search_channels(
        self: &Arc<Instance>,
        query: &str,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Channel>, usize)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/search/video-channels");

        let mut req = self
            .add_user_agent(ureq::get(&url))
            .query("search", query)
            .query("count", &nb.to_string())
            .query("start", &offset.to_string());

        if self.local {
            req = req.query("filter", "local");
        }

        let search_res: Channels = serde_json::from_str(&req.call()?.into_string()?)?;
        let mut res = Vec::new();
        for video in search_res.data {
            if let Some(v) = Channel::maybe_from(video, self.host.clone()) {
                res.push(v);
            }
        }

        Ok((res, search_res.total))
    }

    /// Load a single video from its uuid
    pub fn single_video(self: &Arc<Instance>, host: &str, uuid: &str) -> error::Result<Video> {
        let url = format!("{}/api/v1/videos/{}", self.api_host(host), uuid);

        let req = self.add_user_agent(ureq::get(&url));
        Ok(Video::from_full(
            self,
            serde_json::from_str(&req.call()?.into_string()?)?,
        ))
    }

    /// Fetch a video description
    pub fn video_description(
        self: &Arc<Instance>,
        host: &str,
        uuid: &str,
    ) -> error::Result<Option<String>> {
        let url = format!("{}/api/v1/videos/{}/description", self.api_host(host), uuid);

        let req = ureq::get(&url);
        let desc: Description = serde_json::from_str(&req.call()?.into_string()?)?;
        Ok(desc.description)
    }

    /// Fetch the files for a given video uuid
    pub fn video_complete(
        self: &Arc<Instance>,
        host: &str,
        uuid: &str,
    ) -> error::Result<(Vec<File>, Vec<StreamingPlaylist>)> {
        let url = format!("{}/api/v1/videos/{}", self.api_host(host), uuid);

        let req = self.add_user_agent(ureq::get(&url));
        let video: FullVideo = serde_json::from_str(&req.call()?.into_string()?)?;
        Ok((video.files, video.streamingPlaylists))
    }

    pub fn channel_url(&self, host: &str, channel: &Channel) -> String {
        format!(
            "{}/video-channels/{}/videos",
            self.api_host(host),
            channel.handle()
        )
    }

    pub fn host(&self) -> &String {
        &self.host
    }

    /// Returns the host to be used for api requests outside of search
    fn api_host<'i>(&'i self, host: &'i str) -> Cow<'i, str> {
        if !self.is_search {
            Cow::Borrowed(&self.host)
        } else if !host.is_empty() {
            to_https(host)
        } else {
            to_https(&self.host)
        }
    }
}

fn nsfw_string(include_nsfw: bool) -> &'static str {
    if include_nsfw {
        "both"
    } else {
        "false"
    }
}
