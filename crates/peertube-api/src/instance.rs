use std::convert::TryFrom;
use std::sync::Arc;

use nanoserde::DeJson;

use peertube_ser::channels::Channels;
use peertube_ser::video::{Description, File, StreamingPlaylist, Video as FullVideo};
use peertube_ser::{Comments, Videos};

use crate::channels::Channel;
use crate::comments::Comment;
use crate::error;
use crate::video::Video;

/// Connexion to an instance
/// Video that this instance returns through `search_videos` will all use the instance which
/// created them. This avoids connecting to many distinct instances.
pub struct Instance {
    host: String,
    user_agent: Option<String>,
    include_nsfw: &'static str,
    local: bool,
}

fn status_or_error(res: ureq::Response) -> error::Result<ureq::Response> {
    if res.error() {
        Err(error::Error::Status(res.status()))
    } else {
        Ok(res)
    }
}

impl Instance {
    pub fn new(
        host: String,
        include_nsfw: bool,
        local: bool,
        user_agent: Option<String>,
    ) -> Arc<Instance> {
        Arc::new(Instance {
            host,
            user_agent,
            include_nsfw: nsfw_string(include_nsfw),
            local,
        })
    }

    /// Adds the user agent if there is one
    fn add_user_agent<'r>(&self, req: &'r mut ureq::Request) -> &'r mut ureq::Request {
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

        let mut req = ureq::get(&url);
        self.add_user_agent(&mut req)
            .query("search", query)
            .query("count", &nb.to_string())
            .query("start", &offset.to_string())
            .query("nsfw", self.include_nsfw);

        if self.local {
            req.query("filter", "local");
        }

        let search_res: Videos =
            DeJson::deserialize_json(&status_or_error(req.call())?.into_string()?)?;
        let mut res = Vec::new();

        for video in search_res.data {
            res.push(Video::from_search(self, video));
        }

        Ok((res, search_res.total))
    }

    pub fn channel_videos(
        self: &Arc<Instance>,
        handle: &str,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Video>, usize)> {
        let url = format!("{}/api/v1/video-channels/{}/videos", self.host, handle);

        let mut req = ureq::get(&url);
        self.add_user_agent(&mut req)
            .query("nsfw", self.include_nsfw)
            .query("count", &nb.to_string())
            .query("start", &offset.to_string())
            .set("User-Agent", concat!(env!("CARGO_PKG_NAME")));

        if self.local {
            req.query("filter", "local");
        }

        let video_res: Videos =
            DeJson::deserialize_json(&status_or_error(req.call())?.into_string()?)?;
        let mut res = Vec::new();
        for video in video_res.data {
            res.push(Video::from_search(self, video));
        }

        Ok((res, video_res.total))
    }

    pub fn comments(
        self: &Arc<Instance>,
        video_uuid: &str,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Comment>, usize)> {
        let url = format!("{}/api/v1/videos/{}/comment-threads", self.host, video_uuid);

        let mut req = ureq::get(&url);
        self.add_user_agent(&mut req)
            .query("count", &nb.to_string())
            .query("start", &offset.to_string());

        let comment_res: Comments =
            DeJson::deserialize_json(&status_or_error(req.call())?.into_string()?)?;
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

        let mut req = ureq::get(&url);
        self.add_user_agent(&mut req)
            .query("sort", "-trending")
            .query("count", &nb.to_string())
            .query("start", &offset.to_string())
            .query("nsfw", self.include_nsfw);

        if self.local {
            req.query("filter", "local");
        }

        let search_res: Videos =
            DeJson::deserialize_json(&status_or_error(req.call())?.into_string()?)?;
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

        let mut req = ureq::get(&url);
        self.add_user_agent(&mut req)
            .query("search", query)
            .query("count", &nb.to_string())
            .query("start", &offset.to_string());

        if self.local {
            req.query("filter", "local");
        }

        let search_res: Channels =
            DeJson::deserialize_json(&status_or_error(req.call())?.into_string()?)?;
        let mut res = Vec::new();
        for video in search_res.data {
            if let Some(v) = Channel::maybe_from(video, self.host.clone()) {
                res.push(v);
            }
        }

        Ok((res, search_res.total))
    }

    /// Load a single video from its uuid
    pub fn single_video(self: &Arc<Instance>, uuid: &str) -> error::Result<Video> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(uuid);

        let mut req = ureq::get(&url);
        self.add_user_agent(&mut req);
        Ok(Video::from_full(
            self,
            DeJson::deserialize_json(&status_or_error(req.call())?.into_string()?)?,
        ))
    }

    /// Fetch a video description
    pub fn video_description(self: &Arc<Instance>, uuid: &str) -> error::Result<Option<String>> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(uuid);
        url.push_str("/description");

        let mut req = ureq::get(&url);
        let desc: Description =
            DeJson::deserialize_json(&status_or_error(req.call())?.into_string()?)?;
        Ok(desc.description)
    }

    /// Fetch the files for a given video uuid
    pub fn video_complete(
        self: &Arc<Instance>,
        uuid: &str,
    ) -> error::Result<(Vec<File>, Vec<StreamingPlaylist>)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(uuid);

        let mut req = ureq::get(&url);
        self.add_user_agent(&mut req);
        let video: FullVideo =
            DeJson::deserialize_json(&status_or_error(req.call())?.into_string()?)?;
        Ok((video.files, video.streamingPlaylists))
    }

    pub fn channel_url(&self, channel: &Channel) -> String {
        let mut channel_url = self.host.clone();
        channel_url.push_str("/video-channels/");
        channel_url.push_str(&channel.handle());
        channel_url.push_str("/videos/");
        channel_url
    }

    pub fn host(&self) -> &String {
        &self.host
    }
}

fn nsfw_string(include_nsfw: bool) -> &'static str {
    if include_nsfw {
        "both"
    } else {
        "false"
    }
}
