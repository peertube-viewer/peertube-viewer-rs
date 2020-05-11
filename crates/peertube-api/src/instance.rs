#[cfg(not(feature = "send"))]
use std::rc::Rc as FeaturedRc;
#[cfg(feature = "send")]
use std::sync::Arc as FeaturedRc;

use reqwest::Client;

use peertube_ser::search::Search;
use peertube_ser::video::{Description, File, Video as FullVideo};

use crate::error;
use crate::video::Video;

/// Connexion to an instance
/// Video that this instance returns through `search_videos` will all use the instance which
/// created them. This avoids connecting to many distinct instances.
pub struct Instance {
    client: Client,
    host: String,
    include_nsfw: &'static str,
}

impl Instance {
    pub fn new(host: String, include_nsfw: bool) -> FeaturedRc<Instance> {
        FeaturedRc::new(Instance {
            client: Client::new(),
            host,
            include_nsfw: nsfw_string(include_nsfw),
        })
    }

    /// Perform a search for the given query
    pub async fn search_videos(
        self: &FeaturedRc<Instance>,
        query: &str,
        nb: usize,
        skip: usize,
    ) -> error::Result<Vec<Video>> {
        let mut url = self.host.clone();
        url.push_str("/api/v1/search/videos");
        let mut search_res: Search = serde_json::from_str(
            &*self
                .client
                .get(&url)
                .query(&[
                    ("search", query),
                    ("count", &nb.to_string()),
                    ("start", &skip.to_string()),
                    ("nsfw", self.include_nsfw),
                ])
                .send()
                .await?
                .text()
                .await?,
        )?;
        let mut res = Vec::new();
        for video in search_res.data.drain(..) {
            if let Some(v) = Video::maybe_from(self, video) {
                res.push(v);
            }
        }

        Ok(res)
    }

    /// Load a single video from its uuid
    pub async fn single_video(self: &FeaturedRc<Instance>, uuid: &str) -> error::Result<Video> {
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
        self: &FeaturedRc<Instance>,
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
    pub async fn video_complete(
        self: &FeaturedRc<Instance>,
        uuid: &str,
    ) -> error::Result<Vec<File>> {
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
