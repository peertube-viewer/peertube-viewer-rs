use std::rc::Rc;

use reqwest::Client;

use peertube_ser::channels::Channels;
use peertube_ser::video::{Description, File, Video as FullVideo};
use peertube_ser::Videos;

use crate::channels::Channel;
use crate::error;
use crate::preloadable::{ChannelSearch, VideoList};
use crate::video::Video;

/// Connexion to an instance
/// Video that this instance returns through `search_videos` will all use the instance which
/// created them. This avoids connecting to many distinct instances.
pub struct Instance {
    client: Client,
    host: String,
    include_nsfw: &'static str,
    local: bool,
}

impl Instance {
    pub fn new(host: String, include_nsfw: bool, local: bool) -> Rc<Instance> {
        Rc::new(Instance {
            client: Client::new(),
            host,
            include_nsfw: nsfw_string(include_nsfw),
            local,
        })
    }

    /// Perform a search for the given query
    pub async fn search_videos(
        self: &Rc<Instance>,
        query: &str,
        nb: usize,
        skip: usize,
    ) -> error::Result<(Vec<Video>, Option<usize>)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/search/videos");

        let mut query = self.client.get(&url).query(&[
            ("search", query),
            ("count", &nb.to_string()),
            ("start", &skip.to_string()),
            ("nsfw", self.include_nsfw),
        ]);

        if self.local {
            query = query.query(&[("filter", "local")]);
        }

        let mut search_res: Videos = serde_json::from_str(&query.send().await?.text().await?)?;
        let mut res = Vec::new();
        for video in search_res.data.drain(..) {
            if let Some(v) = Video::maybe_from(self, video) {
                res.push(v);
            }
        }

        let total = match search_res.total {
            Some(c) if c > 0 => Some(c as usize),
            _ => None,
        };

        Ok((res, total))
    }

    pub async fn channel_videos(
        self: &Rc<Instance>,
        handle: &str,
        nb: usize,
        skip: usize,
    ) -> error::Result<(Vec<Video>, Option<usize>)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/video-channels/");
        url.push_str(handle);
        url.push_str("/videos");

        let mut query = self.client.get(&url).query(&[
            ("nsfw", self.include_nsfw),
            ("count", &nb.to_string()),
            ("start", &skip.to_string()),
        ]);

        if self.local {
            query = query.query(&[("filter", "local")]);
        }

        let mut video_res: Videos = serde_json::from_str(&query.send().await?.text().await?)?;
        let mut res = Vec::new();
        for video in video_res.data.drain(..) {
            if let Some(v) = Video::maybe_from(self, video) {
                res.push(v);
            }
        }

        let total = match video_res.total {
            Some(c) if c > 0 => Some(c as usize),
            _ => None,
        };

        Ok((res, total))
    }

    pub fn search(self: &Rc<Instance>, query: &str, skip: usize) -> VideoList {
        VideoList::new_search(self.clone(), query, skip)
    }

    pub fn channel(self: &Rc<Instance>, handle: &str, skip: usize) -> VideoList {
        VideoList::new_channel(self.clone(), handle, skip)
    }

    /// Perform a search for the given query
    pub async fn trending_videos(
        self: &Rc<Instance>,
        nb: usize,
        skip: usize,
    ) -> error::Result<(Vec<Video>, Option<usize>)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos");

        let mut query = self.client.get(&url).query(&[
            ("sort", "-trending"),
            ("count", &nb.to_string()),
            ("start", &skip.to_string()),
            ("nsfw", self.include_nsfw),
        ]);

        if self.local {
            query = query.query(&[("filter", "local")]);
        }

        let mut search_res: Videos = serde_json::from_str(&query.send().await?.text().await?)?;
        let mut res = Vec::new();
        for video in search_res.data.drain(..) {
            if let Some(v) = Video::maybe_from(self, video) {
                res.push(v);
            }
        }

        let total = match search_res.total {
            Some(c) if c > 0 => Some(c as usize),
            _ => None,
        };

        Ok((res, total))
    }
    /// Perform a search for the given query
    pub async fn search_channels(
        self: &Rc<Instance>,
        query: &str,
        nb: usize,
        skip: usize,
    ) -> error::Result<(Vec<Channel>, Option<usize>)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/search/video-channels");

        let mut query = self.client.get(&url).query(&[
            ("search", query),
            ("count", &nb.to_string()),
            ("start", &skip.to_string()),
        ]);

        if self.local {
            query = query.query(&[("filter", "local")]);
        }

        let mut search_res: Channels = serde_json::from_str(&query.send().await?.text().await?)?;
        let mut res = Vec::new();
        for video in search_res.data.drain(..) {
            if let Some(v) = Channel::maybe_from(video) {
                res.push(v);
            }
        }

        let total = match search_res.total {
            Some(c) if c > 0 => Some(c as usize),
            _ => None,
        };

        Ok((res, total))
    }

    pub fn channels(self: &Rc<Instance>, query: &str, skip: usize) -> ChannelSearch {
        ChannelSearch::new(self.clone(), query, skip)
    }

    pub fn trending(self: &Rc<Instance>, skip: usize) -> VideoList {
        VideoList::new_trending(self.clone(), skip)
    }

    /// Load a single video from its uuid
    pub async fn single_video(self: &Rc<Instance>, uuid: &str) -> error::Result<Video> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(uuid);
        Ok(Video::from(
            self,
            serde_json::from_str::<FullVideo>(&*self.client.get(&url).send().await?.text().await?)?,
        ))
    }

    /// Fetch a video description
    pub async fn video_description(
        self: &Rc<Instance>,
        uuid: &str,
    ) -> error::Result<Option<String>> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(uuid);
        url.push_str("/description");

        let desc: Description =
            serde_json::from_str(&*self.client.get(&url).send().await?.text().await?)?;
        Ok(desc.description)
    }

    /// Fetch the files for a given video uuid
    pub async fn video_complete(self: &Rc<Instance>, uuid: &str) -> error::Result<Vec<File>> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(uuid);

        let video: FullVideo =
            serde_json::from_str(&*self.client.get(&url).send().await?.text().await?)?;
        Ok(video.files)
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
