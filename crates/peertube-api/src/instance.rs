use std::convert::TryFrom;
use std::rc::Rc;

use reqwest::Client;

use peertube_ser::channels::Channels;
use peertube_ser::video::{Description, File, Video as FullVideo};
use peertube_ser::{Comments, Videos};

use crate::channels::Channel;
use crate::comments::Comment;
use crate::error;
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
        offset: usize,
    ) -> error::Result<(Vec<Video>, usize)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/search/videos");

        let mut query = self.client.get(&url).query(&[
            ("search", query),
            ("count", &nb.to_string()),
            ("start", &offset.to_string()),
            ("nsfw", self.include_nsfw),
        ]);

        if self.local {
            query = query.query(&[("filter", "local")]);
        }

        let mut search_res: Videos = serde_json::from_str(&query.send().await?.text().await?)?;
        let mut res = Vec::new();

        for video in search_res.data.drain(..) {
            res.push(Video::from_search(self, video));
        }

        Ok((res, search_res.total))
    }

    pub async fn channel_videos(
        self: &Rc<Instance>,
        handle: &str,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Video>, usize)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/video-channels/");
        url.push_str(handle);
        url.push_str("/videos");

        let mut query = self.client.get(&url).query(&[
            ("nsfw", self.include_nsfw),
            ("count", &nb.to_string()),
            ("start", &offset.to_string()),
        ]);

        if self.local {
            query = query.query(&[("filter", "local")]);
        }

        let mut video_res: Videos = serde_json::from_str(&query.send().await?.text().await?)?;
        let mut res = Vec::new();
        for video in video_res.data.drain(..) {
            res.push(Video::from_search(self, video));
        }

        Ok((res, video_res.total))
    }

    pub async fn comments(
        self: &Rc<Instance>,
        video_uuid: &str,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Comment>, usize)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(video_uuid);
        url.push_str("/comment-threads");

        let query = self
            .client
            .get(&url)
            .query(&[("count", &nb.to_string()), ("start", &offset.to_string())]);

        let mut comment_res: Comments = serde_json::from_str(&query.send().await?.text().await?)?;
        let mut res = Vec::new();
        for comment in comment_res.data.drain(..) {
            if let Ok(c) = Comment::try_from(comment) {
                res.push(c);
            }
        }

        Ok((res, comment_res.total))
    }

    /// Returns the trending videos of an instance
    pub async fn trending_videos(
        self: &Rc<Instance>,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Video>, usize)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos");

        let mut query = self.client.get(&url).query(&[
            ("sort", "-trending"),
            ("count", &nb.to_string()),
            ("start", &offset.to_string()),
            ("nsfw", self.include_nsfw),
        ]);

        if self.local {
            query = query.query(&[("filter", "local")]);
        }

        let mut search_res: Videos = serde_json::from_str(&query.send().await?.text().await?)?;
        let mut res = Vec::new();
        for video in search_res.data.drain(..) {
            res.push(Video::from_search(self, video));
        }

        Ok((res, search_res.total))
    }

    /// Perform a search for the given query
    pub async fn search_channels(
        self: &Rc<Instance>,
        query: &str,
        nb: usize,
        offset: usize,
    ) -> error::Result<(Vec<Channel>, usize)> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/search/video-channels");

        let mut query = self.client.get(&url).query(&[
            ("search", query),
            ("count", &nb.to_string()),
            ("start", &offset.to_string()),
        ]);

        if self.local {
            query = query.query(&[("filter", "local")]);
        }

        let mut search_res: Channels = serde_json::from_str(&query.send().await?.text().await?)?;
        let mut res = Vec::new();
        for video in search_res.data.drain(..) {
            if let Some(v) = Channel::maybe_from(video, self.host.clone()) {
                res.push(v);
            }
        }

        Ok((res, search_res.total))
    }

    /// Load a single video from its uuid
    pub async fn single_video(self: &Rc<Instance>, uuid: &str) -> error::Result<Video> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/videos/");
        url.push_str(uuid);
        Ok(Video::from_full(
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
